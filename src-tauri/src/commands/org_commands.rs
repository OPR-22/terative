use serde::{Deserialize, Serialize};
use tauri::State;

use super::{AppState, OrgServices};
use crate::adapters::sqlite::connection::{
    change_org_db_key, open_org_db, probe_org_file, validate_org_key, OpenOrgError, OrgFileKind,
};
use crate::application::org_registry::OrgRegistry;
use crate::application::ports::OrgKeyStore;
use crate::application::{AppError, SecretKey};
use crate::domain::org::OrgCode;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OrgSummaryDto {
    /// User-supplied code — also the on-disk folder name and the picker
    /// label. Validated to `[a-z0-9_-]+` at creation.
    pub code: String,
    pub has_password: bool,
    pub last_modified_at: Option<String>,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OrgInfoDto {
    pub code: String,
}

#[tauri::command]
#[specta::specta]
pub fn org_list(state: State<'_, AppState>) -> Result<Vec<OrgSummaryDto>, AppError> {
    let mut summaries = Vec::new();
    for s in state.org_registry.list()? {
        summaries.push(OrgSummaryDto {
            code: s.code.as_str().to_string(),
            has_password: s.encrypted,
            last_modified_at: s.last_modified.map(|t| t.to_rfc3339()),
            file_size_bytes: s.file_size_bytes,
        });
    }
    Ok(summaries)
}

#[tauri::command]
#[specta::specta]
pub fn org_create(
    state: State<'_, AppState>,
    code: String,
    password: Option<String>,
) -> Result<OrgSummaryDto, AppError> {
    let parsed = OrgCode::parse(&code)?;
    let created = state.org_registry.create(parsed, password.as_deref())?;

    let path = state.org_registry.db_path(&created);
    let meta = std::fs::metadata(&path)?;
    let last_modified_at = meta.modified().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Utc> = t.into();
        dt.to_rfc3339()
    });

    Ok(OrgSummaryDto {
        code: created.as_str().to_string(),
        has_password: password.is_some(),
        last_modified_at,
        file_size_bytes: meta.len(),
    })
}

/// Open an org. For encrypted orgs the key is resolved as: `password`
/// argument → OS keyring entry → `OrgPasswordRequired` error.
///
/// `remember` (default `true`) only takes effect when the caller supplies
/// `password`: it stores the new key in the keyring (or, when `false`,
/// clears any existing entry — opt-out). Ignored for plaintext orgs.
#[tauri::command]
#[specta::specta]
pub fn org_open(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
    password: Option<String>,
    remember: Option<bool>,
) -> Result<OrgInfoDto, AppError> {
    let parsed = OrgCode::parse(&code)?;
    let db_path = state.org_registry.db_path(&parsed);

    // Bookmark webviews are bound to a per-(org, bookmark) data partition
    // at construction. Close any open ones so the next bookmark navigation
    // creates fresh webviews against the new org's storage.
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);

    let invoices_dir = state.org_registry.invoices_dir(&parsed);
    let user_backup_dir = state.org_registry.user_backup_dir(&parsed);
    let system_backup_dir = state.org_registry.system_backup_dir(&parsed);

    for d in [&invoices_dir, &user_backup_dir, &system_backup_dir] {
        std::fs::create_dir_all(d)?;
    }

    let kind = if db_path.exists() {
        probe_org_file(&db_path).unwrap_or(OrgFileKind::Empty)
    } else {
        OrgFileKind::Empty
    };

    let resolved = resolve_org_key(
        state.org_key_store.as_ref(),
        parsed.as_str(),
        password.as_deref(),
        kind,
    )?;

    // Verify the key unlocks the file *before* doing anything else.
    // Otherwise `snapshot_pre_migration_if_pending` is the first I/O on
    // the file and it surfaces SQLCipher's NotADatabase error as a
    // generic Internal — the frontend wants OrgWrongPassword so it can
    // re-prompt cleanly.
    match validate_org_key(&db_path, resolved.as_secret().map(|s| s.expose())) {
        Ok(()) => {}
        Err(OpenOrgError::WrongPassword) => {
            return Err(AppError::org_wrong_password(code.clone()));
        }
        Err(OpenOrgError::NotFound) => {} // fresh org, nothing to validate
        Err(OpenOrgError::ForeignFile) => {
            return Err(AppError::org_not_found(code.clone()))
        }
        Err(OpenOrgError::Other(err)) => return Err(AppError::internal(err.to_string())),
    }

    // Snapshot the on-disk file BEFORE `open_org_db` runs migrations — once
    // they've applied, we've lost the pre-migration state.
    match crate::adapters::sqlite::snapshot_pre_migration_if_pending(
        &db_path,
        &system_backup_dir,
        resolved.as_secret().map(|s| s.expose()),
    ) {
        Ok(Some(path)) => {
            eprintln!("pre-migration snapshot written to {}", path.display())
        }
        Ok(None) => {}
        Err(e) => {
            return Err(AppError::internal(format!(
                "pre-migration snapshot failed for org '{code}': {e}"
            )));
        }
    }

    let db = open_org_db(&db_path, resolved.as_secret().map(|s| s.expose())).map_err(|e| match e {
        OpenOrgError::NotFound => AppError::org_not_found(code.clone()),
        OpenOrgError::ForeignFile => AppError::org_not_found(code.clone()),
        OpenOrgError::WrongPassword => AppError::org_wrong_password(code.clone()),
        OpenOrgError::Other(err) => AppError::internal(err.to_string()),
    })?;

    // Sync the keyring with the caller's intent: only when the caller
    // explicitly supplied a password do we touch the entry — passwords
    // sourced from the keyring stay as-is.
    if let ResolvedKey::FromCaller(ref key) = resolved {
        let remember_pw = remember.unwrap_or(true);
        let result = if remember_pw {
            state.org_key_store.set(parsed.as_str(), key.expose())
        } else {
            state.org_key_store.delete(parsed.as_str())
        };
        if let Err(e) = result {
            eprintln!("keyring write failed for org '{code}': {e}");
        }
    }

    crate::seed_default_template_if_empty(&db);
    crate::seed_default_email_templates_if_empty(&db);

    let services = OrgServices::new(
        parsed.clone(),
        db,
        db_path,
        invoices_dir,
        user_backup_dir,
        system_backup_dir,
        resolved.into_secret(),
    );
    state.open_org(services);

    Ok(OrgInfoDto {
        code: parsed.as_str().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn org_close(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);
    state.close_org();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn org_delete(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
) -> Result<(), AppError> {
    let parsed = OrgCode::parse(&code)?;
    if state
        .active_code()
        .map(|c| c.as_str() == parsed.as_str())
        .unwrap_or(false)
    {
        state.close_org();
    }
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);
    state.org_registry.delete(&parsed)
}

#[tauri::command]
#[specta::specta]
pub fn org_get_active(state: State<'_, AppState>) -> Result<Option<OrgInfoDto>, AppError> {
    Ok(state.active_code().map(|code| OrgInfoDto {
        code: code.as_str().to_string(),
    }))
}

/// Set or change the password on an org's database.
///
/// Resolves the current key from the caller-supplied `current_password`
/// argument first, then the OS keyring entry. The new key is written via
/// `change_org_db_key`. On success, the keyring entry is updated to the
/// new password unless `remember == Some(false)`.
///
/// The org must be closed before rekeying — if `code` matches the active
/// org, this command closes it first.
#[tauri::command]
#[specta::specta]
pub fn org_set_password(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
    current_password: Option<String>,
    new_password: String,
    remember: Option<bool>,
) -> Result<(), AppError> {
    if new_password.is_empty() {
        return Err(AppError::invalid_org_code(
            "new_password must be non-empty; use org_remove_password to disable encryption",
        ));
    }
    close_if_active(&app, &state, &code)?;
    rekey_org(
        state.org_registry.as_ref(),
        state.org_key_store.as_ref(),
        &code,
        current_password.as_deref(),
        Some(new_password.as_str()),
        remember.unwrap_or(true),
    )
}

/// Remove the password from an org, leaving the database plaintext on
/// disk. Clears the keyring entry.
#[tauri::command]
#[specta::specta]
pub fn org_remove_password(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
    current_password: Option<String>,
) -> Result<(), AppError> {
    close_if_active(&app, &state, &code)?;
    rekey_org(
        state.org_registry.as_ref(),
        state.org_key_store.as_ref(),
        &code,
        current_password.as_deref(),
        None,
        false,
    )
}

fn close_if_active(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    code: &str,
) -> Result<(), AppError> {
    let parsed = OrgCode::parse(code)?;
    if state
        .active_code()
        .map(|c| c.as_str() == parsed.as_str())
        .unwrap_or(false)
    {
        crate::commands::bookmark_commands::close_all_bookmark_webviews(app);
        state.close_org();
    }
    Ok(())
}

/// Outcome of resolving the key needed to unlock an org.
///
/// Encrypted orgs require a key; plaintext/empty orgs don't. The variant
/// also remembers whether the key came from the caller (eligible for a
/// keyring write) or from the keyring (already persisted) — only used by
/// the open path.
#[derive(Debug)]
pub(crate) enum ResolvedKey {
    NotNeeded,
    FromCaller(SecretKey),
    FromKeyring(SecretKey),
}

impl ResolvedKey {
    pub(crate) fn as_secret(&self) -> Option<&SecretKey> {
        match self {
            ResolvedKey::FromCaller(s) | ResolvedKey::FromKeyring(s) => Some(s),
            ResolvedKey::NotNeeded => None,
        }
    }

    pub(crate) fn into_secret(self) -> Option<SecretKey> {
        match self {
            ResolvedKey::FromCaller(s) | ResolvedKey::FromKeyring(s) => Some(s),
            ResolvedKey::NotNeeded => None,
        }
    }
}

/// Resolve the key needed to unlock an org. Tries the caller-supplied
/// `caller_password` first, then falls back to the keyring. Returns
/// [`AppError::org_password_required`] if the file is encrypted and
/// neither source yields a key.
pub(crate) fn resolve_org_key(
    keystore: &dyn OrgKeyStore,
    code: &str,
    caller_password: Option<&str>,
    kind: OrgFileKind,
) -> Result<ResolvedKey, AppError> {
    match kind {
        OrgFileKind::Foreign => Err(AppError::org_not_found(code)),
        OrgFileKind::Plaintext | OrgFileKind::Empty => Ok(ResolvedKey::NotNeeded),
        OrgFileKind::Encrypted => match caller_password {
            Some(p) => Ok(ResolvedKey::FromCaller(SecretKey::new(p))),
            None => match keystore.get(code) {
                Ok(Some(p)) => Ok(ResolvedKey::FromKeyring(SecretKey::new(p))),
                Ok(None) => Err(AppError::org_password_required(code)),
                Err(e) => {
                    eprintln!("keyring read failed for org '{code}': {e}");
                    Err(AppError::org_password_required(code))
                }
            },
        },
    }
}

/// Change (set, rotate, or remove) the password on an org. Resolves the
/// *current* key the same way [`resolve_org_key`] does, then rewrites
/// the file via `change_org_db_key`. The keyring is synced afterwards:
/// when `new_password` is `Some` and `remember` is `true`, the new value
/// is stored; in every other case the entry is cleared.
///
/// Caller must close the active org first if `code` matches it.
pub(crate) fn rekey_org(
    registry: &OrgRegistry,
    keystore: &dyn OrgKeyStore,
    code: &str,
    current_password: Option<&str>,
    new_password: Option<&str>,
    remember: bool,
) -> Result<(), AppError> {
    let parsed = OrgCode::parse(code)?;
    let db_path = registry.db_path(&parsed);
    if !db_path.exists() {
        return Err(AppError::org_not_found(code));
    }

    let kind = probe_org_file(&db_path).unwrap_or(OrgFileKind::Empty);
    let resolved = resolve_org_key(keystore, parsed.as_str(), current_password, kind)?;

    change_org_db_key(
        &db_path,
        resolved.as_secret().map(|s| s.expose()),
        new_password,
    )
    .map_err(|e| match e {
        OpenOrgError::WrongPassword => AppError::org_wrong_password(code),
        OpenOrgError::NotFound => AppError::org_not_found(code),
        other => AppError::internal(other.to_string()),
    })?;

    let keyring_result = match (new_password, remember) {
        (Some(p), true) => keystore.set(parsed.as_str(), p),
        _ => keystore.delete(parsed.as_str()),
    };
    if let Err(e) = keyring_result {
        eprintln!("keyring sync after rekey failed for org '{code}': {e}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use parking_lot::Mutex;
    use tempfile::TempDir;

    use super::*;
    use crate::application::ports::OrgKeyStore;
    use crate::application::RepoError;

    /// In-memory test double for `OrgKeyStore` — verifies write/delete
    /// side effects without touching the real OS keystore.
    #[derive(Default)]
    struct FakeKeyStore {
        entries: Mutex<HashMap<String, String>>,
    }

    impl OrgKeyStore for FakeKeyStore {
        fn get(&self, code: &str) -> Result<Option<String>, RepoError> {
            Ok(self.entries.lock().get(code).cloned())
        }
        fn set(&self, code: &str, password: &str) -> Result<(), RepoError> {
            self.entries
                .lock()
                .insert(code.to_string(), password.to_string());
            Ok(())
        }
        fn delete(&self, code: &str) -> Result<(), RepoError> {
            self.entries.lock().remove(code);
            Ok(())
        }
    }

    fn fixture() -> (TempDir, OrgRegistry, FakeKeyStore) {
        let tmp = tempfile::tempdir().unwrap();
        let reg = OrgRegistry::new(tmp.path().join("orgs"));
        (tmp, reg, FakeKeyStore::default())
    }

    fn parse(c: &str) -> OrgCode {
        OrgCode::parse(c).unwrap()
    }

    // === resolve_org_key ============================================

    #[test]
    fn resolve_org_key_returns_not_needed_for_plaintext_kind() {
        let keystore = FakeKeyStore::default();
        let r = resolve_org_key(&keystore, "acme", None, OrgFileKind::Plaintext).unwrap();
        assert!(matches!(r, ResolvedKey::NotNeeded));
    }

    #[test]
    fn resolve_org_key_returns_not_needed_for_empty_kind() {
        let keystore = FakeKeyStore::default();
        let r = resolve_org_key(&keystore, "acme", None, OrgFileKind::Empty).unwrap();
        assert!(matches!(r, ResolvedKey::NotNeeded));
    }

    #[test]
    fn resolve_org_key_prefers_caller_password_over_keyring() {
        let keystore = FakeKeyStore::default();
        keystore.set("acme", "from-keyring").unwrap();
        let r = resolve_org_key(
            &keystore,
            "acme",
            Some("from-caller"),
            OrgFileKind::Encrypted,
        )
        .unwrap();
        match r {
            ResolvedKey::FromCaller(s) => assert_eq!(s.expose(), "from-caller"),
            other => panic!("expected FromCaller, got {other:?}"),
        }
    }

    #[test]
    fn resolve_org_key_falls_back_to_keyring_when_caller_silent() {
        let keystore = FakeKeyStore::default();
        keystore.set("acme", "stored").unwrap();
        let r = resolve_org_key(&keystore, "acme", None, OrgFileKind::Encrypted).unwrap();
        match r {
            ResolvedKey::FromKeyring(s) => assert_eq!(s.expose(), "stored"),
            other => panic!("expected FromKeyring, got {other:?}"),
        }
    }

    #[test]
    fn resolve_org_key_returns_password_required_when_encrypted_with_no_key_source() {
        let keystore = FakeKeyStore::default();
        let err = resolve_org_key(&keystore, "acme", None, OrgFileKind::Encrypted).unwrap_err();
        assert!(err.is(crate::application::ErrorCode::OrgPasswordRequired));
    }

    #[test]
    fn resolve_org_key_maps_foreign_kind_to_org_not_found() {
        let keystore = FakeKeyStore::default();
        let err = resolve_org_key(&keystore, "acme", None, OrgFileKind::Foreign).unwrap_err();
        assert!(err.is(crate::application::ErrorCode::OrgNotFound));
    }

    // === rekey_org ==================================================

    #[test]
    fn rekey_org_returns_org_not_found_for_missing_org() {
        let (_t, reg, ks) = fixture();
        let err = rekey_org(&reg, &ks, "ghost", None, Some("pw"), true).unwrap_err();
        assert!(err.is(crate::application::ErrorCode::OrgNotFound));
    }

    #[test]
    fn rekey_org_encrypts_a_plaintext_org_and_stores_key_in_keyring() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), None).unwrap();

        rekey_org(&reg, &ks, "acme", None, Some("hunter2"), true).unwrap();

        // File is now encrypted.
        let kind = probe_org_file(&reg.db_path(&parse("acme"))).unwrap();
        assert_eq!(kind, OrgFileKind::Encrypted);
        // Keyring holds the new key.
        assert_eq!(ks.get("acme").unwrap().as_deref(), Some("hunter2"));
    }

    #[test]
    fn rekey_org_with_remember_false_does_not_store_key_in_keyring() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), None).unwrap();

        rekey_org(&reg, &ks, "acme", None, Some("hunter2"), false).unwrap();

        assert_eq!(
            probe_org_file(&reg.db_path(&parse("acme"))).unwrap(),
            OrgFileKind::Encrypted,
        );
        assert!(ks.get("acme").unwrap().is_none());
    }

    #[test]
    fn rekey_org_remove_password_decrypts_and_clears_keyring() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), Some("pw1")).unwrap();
        ks.set("acme", "pw1").unwrap();

        rekey_org(&reg, &ks, "acme", Some("pw1"), None, false).unwrap();

        assert_eq!(
            probe_org_file(&reg.db_path(&parse("acme"))).unwrap(),
            OrgFileKind::Plaintext,
        );
        assert!(ks.get("acme").unwrap().is_none());
    }

    #[test]
    fn rekey_org_falls_back_to_keyring_when_caller_omits_current_password() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), Some("pw1")).unwrap();
        ks.set("acme", "pw1").unwrap();

        // Caller passes None; the keyring entry must be used to unlock.
        rekey_org(&reg, &ks, "acme", None, Some("pw2"), true).unwrap();
        assert_eq!(ks.get("acme").unwrap().as_deref(), Some("pw2"));
    }

    #[test]
    fn rekey_org_returns_password_required_when_encrypted_with_no_key_source() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), Some("pw1")).unwrap();
        // ks deliberately empty — no caller pw, no keyring entry.

        let err = rekey_org(&reg, &ks, "acme", None, Some("pw2"), true).unwrap_err();
        assert!(err.is(crate::application::ErrorCode::OrgPasswordRequired));
    }

    #[test]
    fn rekey_org_returns_wrong_password_for_bad_current_key() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), Some("pw1")).unwrap();

        let err = rekey_org(&reg, &ks, "acme", Some("nope"), Some("pw2"), true).unwrap_err();
        assert!(err.is(crate::application::ErrorCode::OrgWrongPassword));
        // The on-disk file must still open under pw1.
        assert!(
            crate::adapters::sqlite::connection::open_with_key(
                &reg.db_path(&parse("acme")),
                Some("pw1"),
            )
            .is_ok()
        );
    }

    #[test]
    fn rekey_org_rotates_password_on_encrypted_org() {
        let (_t, reg, ks) = fixture();
        reg.create(parse("acme"), Some("pw1")).unwrap();

        rekey_org(&reg, &ks, "acme", Some("pw1"), Some("pw2"), true).unwrap();

        assert!(crate::adapters::sqlite::connection::open_with_key(
            &reg.db_path(&parse("acme")),
            Some("pw1"),
        )
        .is_err());
        assert!(crate::adapters::sqlite::connection::open_with_key(
            &reg.db_path(&parse("acme")),
            Some("pw2"),
        )
        .is_ok());
        assert_eq!(ks.get("acme").unwrap().as_deref(), Some("pw2"));
    }
}
