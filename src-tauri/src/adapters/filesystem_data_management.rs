use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::application::ports::DataManagement;
use crate::application::RepoError;

pub struct FilesystemDataManagement {
    db_path: PathBuf,
}

impl FilesystemDataManagement {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

fn storage(msg: impl Into<String>) -> RepoError {
    RepoError::Storage(msg.into())
}

/// Sanity check that a candidate restore source is actually a SQLite file.
/// SQLite's file format starts with the magic bytes "SQLite format 3\x00".
fn validate_sqlite_file(path: &Path) -> Result<(), RepoError> {
    let bytes = fs::read(path).map_err(|e| storage(format!("read source: {e}")))?;
    const MAGIC: &[u8] = b"SQLite format 3\0";
    if bytes.len() < MAGIC.len() || &bytes[..MAGIC.len()] != MAGIC {
        return Err(storage("source is not a valid SQLite database"));
    }
    Ok(())
}

impl DataManagement for FilesystemDataManagement {
    fn export_database(&self, destination: &Path) -> Result<PathBuf, RepoError> {
        if let Some(parent) = destination.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| storage(e.to_string()))?;
            }
        }
        fs::copy(&self.db_path, destination).map_err(|e| storage(e.to_string()))?;
        Ok(destination.to_path_buf())
    }

    fn create_backup(&self, backup_dir: &Path) -> Result<PathBuf, RepoError> {
        fs::create_dir_all(backup_dir).map_err(|e| storage(e.to_string()))?;
        let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let path = backup_dir.join(format!("terative-{stamp}.sqlite"));
        fs::copy(&self.db_path, &path).map_err(|e| storage(e.to_string()))?;
        Ok(path)
    }

    fn restore_database(&self, source: &Path) -> Result<(), RepoError> {
        validate_sqlite_file(source)?;
        // Best effort atomic replace: copy into a sibling temp, then rename.
        let parent = self
            .db_path
            .parent()
            .ok_or_else(|| storage("db path has no parent"))?;
        fs::create_dir_all(parent).map_err(|e| storage(e.to_string()))?;
        let tmp = parent.join(format!(
            ".{}.restore-tmp",
            self.db_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("db")
        ));
        fs::copy(source, &tmp).map_err(|e| storage(e.to_string()))?;
        fs::rename(&tmp, &self.db_path).map_err(|e| storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open;

    /// Build a real sqlite file via the real migrations so we have valid magic
    /// bytes and a realistic layout.
    fn seed_real_db(path: &Path) {
        let _db = open(path).expect("open real sqlite");
        // _db is dropped at end of scope, closing the connection cleanly.
    }

    #[test]
    fn export_database_copies_file_to_destination() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("live.sqlite");
        seed_real_db(&src);
        let dest = tmp.path().join("exports").join("snapshot.sqlite");

        let mgr = FilesystemDataManagement::new(src.clone());
        let path = mgr.export_database(&dest).unwrap();

        assert_eq!(path, dest);
        assert!(dest.exists());
        let original = fs::read(&src).unwrap();
        let copied = fs::read(&dest).unwrap();
        assert_eq!(original, copied);
    }

    #[test]
    fn create_backup_generates_timestamped_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("live.sqlite");
        seed_real_db(&src);
        let backup_dir = tmp.path().join("backups");

        let mgr = FilesystemDataManagement::new(src);
        let path = mgr.create_backup(&backup_dir).unwrap();

        assert!(path.starts_with(&backup_dir));
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(
            file_name.starts_with("terative-") && file_name.ends_with(".sqlite"),
            "unexpected backup name: {file_name}",
        );
        assert!(path.exists());
    }

    #[test]
    fn create_backup_creates_missing_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("live.sqlite");
        seed_real_db(&src);
        let nested = tmp.path().join("a").join("b");

        let mgr = FilesystemDataManagement::new(src);
        let path = mgr.create_backup(&nested).unwrap();
        assert!(path.exists());
        assert!(nested.exists());
    }

    #[test]
    fn restore_replaces_live_database() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        seed_real_db(&live);
        // Touch the live file to something distinctive, then create a fresh
        // source sqlite file to restore from.
        fs::write(&live, b"SQLite format 3\0 placeholder garbage").unwrap();
        let source = tmp.path().join("source.sqlite");
        seed_real_db(&source);
        let source_bytes = fs::read(&source).unwrap();

        let mgr = FilesystemDataManagement::new(live.clone());
        mgr.restore_database(&source).unwrap();

        let restored = fs::read(&live).unwrap();
        assert_eq!(restored, source_bytes);
    }

    #[test]
    fn restore_rejects_non_sqlite_file() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        seed_real_db(&live);
        let bad = tmp.path().join("not-a-db.txt");
        fs::write(&bad, b"this is not sqlite").unwrap();

        let mgr = FilesystemDataManagement::new(live.clone());
        let err = mgr.restore_database(&bad).unwrap_err();
        match err {
            RepoError::Storage(msg) => assert!(msg.contains("SQLite"), "{msg}"),
            other => panic!("expected Storage error, got {other:?}"),
        }
        // Live db must be untouched.
        assert!(live.exists());
    }

    #[test]
    fn restore_rejects_missing_source() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        seed_real_db(&live);
        let missing = tmp.path().join("nowhere.sqlite");
        let mgr = FilesystemDataManagement::new(live);
        let err = mgr.restore_database(&missing).unwrap_err();
        assert!(matches!(err, RepoError::Storage(_)));
    }
}
