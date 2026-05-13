//! Filesystem-level service for managing org folders.
//!
//! `OrgRegistry` is stateless beyond the orgs root path. It owns no DB
//! connection — it's available before any org is opened, used by the org
//! picker / create flow.
//!
//! Layout managed here:
//! ```text
//! <orgs_root>/
//! └── <code>/
//!     ├── <code>.sqlite
//!     ├── backups/{system,user}/
//!     └── invoices/
//! ```

use std::path::{Path, PathBuf};

use crate::adapters::sqlite::connection::{probe_org_file, OrgFileKind};
use crate::application::AppError;
use crate::domain::org::OrgCode;

/// Lightweight info about an org on disk. Does not require opening the DB
/// proper — only reads the file's metadata + magic.
#[derive(Debug, Clone)]
pub struct OrgSummary {
    pub code: OrgCode,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub file_size_bytes: u64,
    /// `true` when the on-disk file is SQLCipher-encrypted (header bytes
    /// don't read as SQLite). The picker uses this to show a lock icon
    /// and route through the unlock modal.
    pub encrypted: bool,
}

pub struct OrgRegistry {
    root: PathBuf,
}

impl OrgRegistry {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn dir_for(&self, code: &OrgCode) -> PathBuf {
        self.root.join(code.as_str())
    }

    pub fn db_path(&self, code: &OrgCode) -> PathBuf {
        self.dir_for(code).join(format!("{}.sqlite", code.as_str()))
    }

    pub fn invoices_dir(&self, code: &OrgCode) -> PathBuf {
        self.dir_for(code).join("invoices")
    }

    pub fn user_backup_dir(&self, code: &OrgCode) -> PathBuf {
        self.dir_for(code).join("backups").join("user")
    }

    pub fn system_backup_dir(&self, code: &OrgCode) -> PathBuf {
        self.dir_for(code).join("backups").join("system")
    }

    /// Per-bookmark webview data directory, scoped to the org. Used by the
    /// bookmark webviews so each org keeps its own cookies, localStorage,
    /// service-worker caches — different Gmail accounts logged in per org,
    /// for example.
    pub fn bookmark_webview_dir(&self, code: &OrgCode, bookmark_id: &str) -> PathBuf {
        self.dir_for(code).join("webviews").join(bookmark_id)
    }

    /// True if an org folder exists on disk for `code`. Uses the OS's
    /// native path semantics — case-sensitive on Linux, case-insensitive
    /// on macOS/Windows — so distinct-code orgs can coexist where the
    /// filesystem permits.
    pub fn code_taken_on_disk(&self, code: &str) -> bool {
        self.root.join(code).exists()
    }

    /// Enumerate every valid Terative org under `root`. Foreign sqlite
    /// files are filtered out via `application_id`. Folders without a
    /// matching sqlite are ignored.
    pub fn list(&self) -> Result<Vec<OrgSummary>, AppError> {
        if !self.root.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        let entries = std::fs::read_dir(&self.root).map_err(AppError::from)?;
        for entry in entries.flatten() {
            let folder = entry.path();
            if !folder.is_dir() {
                continue;
            }
            let folder_name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let code = match OrgCode::parse(&folder_name) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let db_path = folder.join(format!("{folder_name}.sqlite"));
            if !db_path.exists() {
                continue;
            }
            let kind = match probe_org_file(&db_path) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let encrypted = match kind {
                OrgFileKind::Plaintext | OrgFileKind::Empty => false,
                OrgFileKind::Encrypted => true,
                OrgFileKind::Foreign => continue,
            };
            let meta = std::fs::metadata(&db_path).map_err(AppError::from)?;
            let last_modified = meta.modified().ok().map(|t| t.into());
            out.push(OrgSummary {
                code,
                last_modified,
                file_size_bytes: meta.len(),
                encrypted,
            });
        }
        out.sort_by(|a, b| {
            a.code
                .as_str()
                .to_ascii_lowercase()
                .cmp(&b.code.as_str().to_ascii_lowercase())
        });
        Ok(out)
    }

    /// Create a new org using the user-supplied `code`. When `password`
    /// is `Some`, the freshly-created `<code>.sqlite` is encrypted under
    /// that key. Errors with `OrgCodeAlreadyExists` if the folder is
    /// already taken.
    pub fn create(&self, code: OrgCode, password: Option<&str>) -> Result<OrgCode, AppError> {
        std::fs::create_dir_all(&self.root).map_err(AppError::from)?;

        if self.code_taken_on_disk(code.as_str()) {
            return Err(AppError::org_code_already_exists(code.as_str()));
        }

        let folder = self.dir_for(&code);
        std::fs::create_dir_all(&folder).map_err(AppError::from)?;

        let result = (|| -> Result<(), AppError> {
            std::fs::create_dir_all(self.system_backup_dir(&code)).map_err(AppError::from)?;
            std::fs::create_dir_all(self.user_backup_dir(&code)).map_err(AppError::from)?;
            std::fs::create_dir_all(self.invoices_dir(&code)).map_err(AppError::from)?;

            let db_path = self.db_path(&code);
            let _db = crate::adapters::sqlite::connection::create_org_db(&db_path, password)
                .map_err(|e| AppError::internal(e.to_string()))?;
            // _db drops here — connection closed. Re-opened by org_open.
            Ok(())
        })();

        if let Err(e) = result {
            // Best-effort cleanup so a half-created org doesn't pollute the
            // picker. If this fails too, the partial folder will be ignored
            // by `list()` (no valid sqlite inside).
            let _ = std::fs::remove_dir_all(&folder);
            return Err(e);
        }

        Ok(code)
    }

    /// Remove an org folder recursively. Caller must close any active
    /// connection first.
    pub fn delete(&self, code: &OrgCode) -> Result<(), AppError> {
        let folder = self.dir_for(code);
        if !folder.exists() {
            return Err(AppError::org_not_found(code.as_str()));
        }
        std::fs::remove_dir_all(&folder).map_err(AppError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ErrorCode;

    fn registry() -> (tempfile::TempDir, OrgRegistry) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("orgs");
        let reg = OrgRegistry::new(root);
        (tmp, reg)
    }

    fn code(s: &str) -> OrgCode {
        OrgCode::parse(s).unwrap()
    }

    #[test]
    fn list_empty_when_root_does_not_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let reg = OrgRegistry::new(tmp.path().join("orgs"));
        assert!(reg.list().unwrap().is_empty());
    }

    #[test]
    fn create_produces_full_folder_layout() {
        let (_t, reg) = registry();
        let c = reg.create(code("acme_corp"), None).unwrap();
        assert_eq!(c.as_str(), "acme_corp");

        let folder = reg.dir_for(&c);
        assert!(folder.is_dir());
        assert!(reg.db_path(&c).is_file());
        assert!(reg.invoices_dir(&c).is_dir());
        assert!(reg.user_backup_dir(&c).is_dir());
        assert!(reg.system_backup_dir(&c).is_dir());
    }

    #[test]
    fn create_sets_application_id_on_sqlite_file() {
        let (_t, reg) = registry();
        let c = reg.create(code("acme"), None).unwrap();
        assert_eq!(
            probe_org_file(&reg.db_path(&c)).unwrap(),
            OrgFileKind::Plaintext,
        );
    }

    #[test]
    fn create_with_password_produces_encrypted_file() {
        let (_t, reg) = registry();
        let c = reg.create(code("locked"), Some("hunter2")).unwrap();
        assert_eq!(
            probe_org_file(&reg.db_path(&c)).unwrap(),
            OrgFileKind::Encrypted,
        );
        // The on-disk file must not begin with the SQLite plaintext magic.
        let bytes = std::fs::read(reg.db_path(&c)).unwrap();
        assert!(!bytes.starts_with(b"SQLite format 3\0"));
    }

    #[test]
    fn list_reports_encrypted_flag_per_org() {
        let (_t, reg) = registry();
        reg.create(code("plain"), None).unwrap();
        reg.create(code("locked"), Some("hunter2")).unwrap();

        let result = reg.list().unwrap();
        let by_code: std::collections::HashMap<&str, bool> = result
            .iter()
            .map(|s| (s.code.as_str(), s.encrypted))
            .collect();
        assert_eq!(by_code.get("plain"), Some(&false));
        assert_eq!(by_code.get("locked"), Some(&true));
    }

    #[test]
    fn create_rejects_existing_code() {
        let (_t, reg) = registry();
        reg.create(code("acme"), None).unwrap();
        let err = reg.create(code("acme"), None).unwrap_err();
        assert!(err.is(ErrorCode::OrgCodeAlreadyExists));
    }


    #[test]
    fn list_returns_created_orgs_sorted_case_insensitive() {
        let (_t, reg) = registry();
        reg.create(code("Zeta"), None).unwrap();
        reg.create(code("alpha"), None).unwrap();
        reg.create(code("Mu"), None).unwrap();
        reg.create(code("beta"), None).unwrap();

        let result = reg.list().unwrap();
        let codes: Vec<&str> = result.iter().map(|s| s.code.as_str()).collect();
        assert_eq!(codes, vec!["alpha", "beta", "Mu", "Zeta"]);
    }

    #[test]
    fn list_filters_out_foreign_sqlite_files() {
        let (_t, reg) = registry();
        reg.create(code("real_org"), None).unwrap();

        let foreign_dir = reg.root.join("foreign");
        std::fs::create_dir_all(&foreign_dir).unwrap();
        let foreign_db = foreign_dir.join("foreign.sqlite");
        let conn = rusqlite::Connection::open(&foreign_db).unwrap();
        conn.execute_batch("PRAGMA application_id = 999;").unwrap();
        drop(conn);

        let result = reg.list().unwrap();
        let codes: Vec<&str> = result.iter().map(|s| s.code.as_str()).collect();
        assert_eq!(codes, vec!["real_org"]);
    }

    #[test]
    fn list_skips_non_sqlite_folders() {
        let (_t, reg) = registry();
        reg.create(code("real"), None).unwrap();
        std::fs::create_dir_all(reg.root.join("noise")).unwrap();

        let result = reg.list().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code.as_str(), "real");
    }

    #[test]
    fn list_skips_folders_with_invalid_codes() {
        let (_t, reg) = registry();
        // Directly create a folder with characters that aren't valid codes.
        std::fs::create_dir_all(reg.root.join("Acme Corp")).unwrap();
        let result = reg.list().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn delete_removes_folder_recursively() {
        let (_t, reg) = registry();
        let c = reg.create(code("acme"), None).unwrap();
        let folder = reg.dir_for(&c);
        assert!(folder.exists());

        reg.delete(&c).unwrap();
        assert!(!folder.exists());
    }

    #[test]
    fn delete_returns_org_not_found_for_missing_code() {
        let (_t, reg) = registry();
        let phantom = code("ghost");
        let err = reg.delete(&phantom).unwrap_err();
        assert!(err.is(ErrorCode::OrgNotFound));
    }
}
