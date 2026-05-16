use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;
use tauri::State;

use super::AppState;
use crate::application::AppError;
use crate::application::ports::{BackupKind, BackupMetadata, BackupScope, DataManagement, OrgKeyStore};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "PascalCase")]
pub enum BackupKindDto {
    Manual,
    Auto,
    PreRestore,
    PreMigration,
}

impl From<BackupKind> for BackupKindDto {
    fn from(value: BackupKind) -> Self {
        match value {
            BackupKind::Manual => BackupKindDto::Manual,
            BackupKind::Auto => BackupKindDto::Auto,
            BackupKind::PreRestore => BackupKindDto::PreRestore,
            BackupKind::PreMigration => BackupKindDto::PreMigration,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "PascalCase")]
pub enum BackupScopeDto {
    User,
    System,
}

impl From<BackupScope> for BackupScopeDto {
    fn from(value: BackupScope) -> Self {
        match value {
            BackupScope::User => BackupScopeDto::User,
            BackupScope::System => BackupScopeDto::System,
        }
    }
}

#[derive(Debug, Serialize, Type)]
pub struct BackupDto {
    pub path: String,
    pub timestamp: DateTime<Utc>,
    pub kind: BackupKindDto,
    pub scope: BackupScopeDto,
    pub size_bytes: u64,
}

impl From<BackupMetadata> for BackupDto {
    fn from(value: BackupMetadata) -> Self {
        BackupDto {
            path: value.path.to_string_lossy().to_string(),
            timestamp: value.timestamp,
            scope: value.kind.scope().into(),
            kind: value.kind.into(),
            size_bytes: value.size_bytes,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn data_export(
    state: State<'_, AppState>,
    destination: String,
) -> Result<String, AppError> {
    state.org()?
        .data_management
        .export_database(&PathBuf::from(destination))
        .map(|p| p.to_string_lossy().to_string())
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_backup(state: State<'_, AppState>) -> Result<String, AppError> {
    // Goes through the `CreateBackup` use case (not `data_management`
    // directly) so a manual backup also records a `BackupCreated` audit.
    state.org()?
        .create_backup
        .execute()
        .map(|p| p.to_string_lossy().to_string())
}

/// Restore the live db from a backup file. `source_password` is required
/// iff the source is SQLCipher-encrypted (see `data_source_appears_encrypted`).
///
/// The keyring entry for the active org is cleared *before* the swap so the
/// next launch always re-prompts: if we cleared after, a failing
/// `keystore.delete` would leave a stale cached key that no longer matches
/// the restored db, surfacing as `OrgWrongPassword` on the next open.
#[tauri::command]
#[specta::specta]
pub fn data_restore(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    source: String,
    source_password: Option<String>,
) -> Result<(), AppError> {
    let org = state.org()?;
    restore_org_data(
        org.data_management.as_ref(),
        state.org_key_store.as_ref(),
        org.code.as_str(),
        &PathBuf::from(source),
        source_password.as_deref(),
    )?;
    // Restart so the live DB connection is closed and the newly-swapped file
    // is opened cleanly by a fresh process. `restart` never returns.
    app.restart()
}

/// Business logic of [`data_restore`], extracted so it can be tested
/// without a Tauri runtime (the command also calls `app.restart()`).
///
/// Clears the keyring entry *first*, then performs the restore. If the
/// keyring delete fails we surface the error and skip the restore — the
/// alternative (restore + stale cached key) confuses the next launch
/// with a misleading `OrgWrongPassword`.
pub(crate) fn restore_org_data(
    data_management: &dyn DataManagement,
    keystore: &dyn OrgKeyStore,
    code: &str,
    source: &Path,
    source_password: Option<&str>,
) -> Result<(), AppError> {
    keystore.delete(code).map_err(AppError::from)?;
    data_management
        .restore_database(source, source_password)
        .map_err(AppError::from)?;
    Ok(())
}

/// Cheap probe: returns true when the file does not begin with the
/// SQLite plaintext magic. The frontend uses this to decide whether to
/// prompt for a password before calling `data_restore`.
#[tauri::command]
#[specta::specta]
pub fn data_source_appears_encrypted(
    state: State<'_, AppState>,
    source: String,
) -> Result<bool, AppError> {
    state.org()?
        .data_management
        .source_appears_encrypted(&PathBuf::from(source))
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_list_backups(state: State<'_, AppState>) -> Result<Vec<BackupDto>, AppError> {
    state.org()?
        .data_management
        .list_backups()
        .map(|items| items.into_iter().map(BackupDto::from).collect())
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_delete_backup(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    state.org()?
        .data_management
        .delete_backup(&PathBuf::from(path))
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_user_backup_dir(state: State<'_, AppState>) -> Result<String, AppError> {
    Ok(state.org()?.user_backup_dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    use parking_lot::Mutex;

    use super::*;
    use crate::adapters::sqlite::connection::open;
    use crate::adapters::FilesystemDataManagement;
    use crate::application::ports::{OrgKeyStore, SettingsRepository};
    use crate::application::{ErrorCode, RepoError, SecretKey};

    #[derive(Default)]
    struct FakeKeyStore {
        entries: Mutex<HashMap<String, String>>,
        fail_delete: Mutex<bool>,
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
            if *self.fail_delete.lock() {
                return Err(RepoError::Storage("simulated delete failure".into()));
            }
            self.entries.lock().remove(code);
            Ok(())
        }
    }

    /// Builds a real `FilesystemDataManagement` against a temp directory.
    /// Returns `(mgr, live_path, tempdir-owner)` — keep the dir alive for
    /// the test's duration.
    fn make_mgr(
        key: Option<SecretKey>,
    ) -> (FilesystemDataManagement, PathBuf, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = match key.as_ref() {
            None => open(&live).unwrap(),
            Some(k) => {
                crate::adapters::sqlite::connection::open_with_key(&live, Some(k.expose())).unwrap()
            }
        };
        let user_dir = tmp.path().join("user");
        let system_dir = tmp.path().join("system");
        let settings = Arc::new(crate::adapters::sqlite::SqliteSettingsRepository::new(
            db.clone(),
        )) as Arc<dyn SettingsRepository>;
        let mgr = FilesystemDataManagement::new(db, live.clone(), settings, user_dir, system_dir, key);
        (mgr, live, tmp)
    }

    #[test]
    fn restore_org_data_clears_keyring_entry_on_success() {
        let (mgr, _live, tmp) = make_mgr(None);
        let ks = FakeKeyStore::default();
        ks.set("acme", "stale-cached-key").unwrap();

        // Build a plaintext source that passes validation.
        let source = tmp.path().join("source.sqlite");
        {
            let _ = open(&source).unwrap();
        }

        restore_org_data(&mgr, &ks, "acme", &source, None).unwrap();

        assert!(
            ks.get("acme").unwrap().is_none(),
            "keyring entry must be cleared after restore",
        );
    }

    #[test]
    fn restore_org_data_does_not_restore_when_keyring_delete_fails() {
        let (mgr, live, tmp) = make_mgr(None);
        let ks = FakeKeyStore::default();
        *ks.fail_delete.lock() = true;

        // Mark the live db so we can detect any mutation.
        {
            let conn = rusqlite::Connection::open(&live).unwrap();
            conn.execute(
                "INSERT INTO clients (id, name, default_currency, archived_at, created_at)
                 VALUES (?1, ?2, 'EUR', NULL, ?3)",
                rusqlite::params!["live-marker", "Should Survive", "2026-01-01T00:00:00Z"],
            )
            .unwrap();
        }

        let source = tmp.path().join("source.sqlite");
        {
            let _ = open(&source).unwrap();
        }

        let err = restore_org_data(&mgr, &ks, "acme", &source, None).unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));

        // Live db must still hold the marker — the keystore failure short-
        // circuited the restore before any file swap.
        let conn = rusqlite::Connection::open(&live).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clients WHERE id = ?1",
                rusqlite::params!["live-marker"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "live db must not be touched when keyring delete fails");
    }

    #[test]
    fn restore_org_data_propagates_wrong_password_from_encrypted_source() {
        let (mgr, _live, tmp) = make_mgr(None);
        let ks = FakeKeyStore::default();

        let source = tmp.path().join("source.sqlite");
        {
            let _ = crate::adapters::sqlite::connection::open_with_key(&source, Some("backup-pw"))
                .unwrap();
        }

        let err = restore_org_data(&mgr, &ks, "acme", &source, Some("wrong-pw")).unwrap_err();
        assert!(err.is(ErrorCode::RestoreWrongPassword));
    }
}
