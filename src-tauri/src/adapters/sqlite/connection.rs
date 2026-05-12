use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};
use rusqlite_migration::{Migrations, SchemaVersion, M};

pub type Db = Arc<Mutex<Connection>>;

/// Magic 4-byte tag stored in the SQLite header's `application_id` field
/// (ASCII "TERA"). Set by the initial migration; read during restore to
/// confirm a backup file originates from this app.
pub const APPLICATION_ID: i32 = 0x5445_5241;

/// Ordered list of embedded migration SQL, one entry per schema version.
/// Add new migrations by appending to this slice — no other bookkeeping needed.
const MIGRATION_SQL: &[&str] = &[include_str!("../../../migrations/001_initial.sql")];

pub(crate) fn migrations() -> Migrations<'static> {
    Migrations::new(MIGRATION_SQL.iter().map(|sql| M::up(sql)).collect())
}

/// Snapshots the database file at `db_path` into `system_backup_dir` with a
/// pre-migration suffix, but only if migrations would actually run on the
/// next [`open`] call. Returns the snapshot path if one was written, or
/// None when the db is already at the latest schema (or does not exist).
///
/// Safe to call before [`open`]: uses a read-only SQLite connection, does
/// not modify the source file. Meant to be invoked at app startup so an
/// upgrade that applies migrations always leaves a rollback point behind.
pub fn snapshot_pre_migration_if_pending(
    db_path: &Path,
    system_backup_dir: &Path,
) -> anyhow::Result<Option<PathBuf>> {
    if !db_path.exists() {
        // Fresh install; no data to protect.
        return Ok(None);
    }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let current = migrations().current_version(&conn)?;
    let max = MIGRATION_SQL.len();
    let pending = match current {
        SchemaVersion::NoneSet => max > 0,
        SchemaVersion::Inside(v) => usize::from(v) < max,
        // `Outside` means the db is newer than this binary; don't snapshot
        // and let `open` surface the mismatch.
        SchemaVersion::Outside(_) => false,
    };
    if !pending {
        return Ok(None);
    }

    std::fs::create_dir_all(system_backup_dir)?;
    let dest = system_backup_dir.join(
        crate::adapters::filesystem_data_management::format_backup_filename(
            crate::application::ports::BackupKind::PreMigration,
            chrono::Utc::now(),
        ),
    );
    let dest_sql = dest.to_string_lossy().replace('\'', "''");
    conn.execute(&format!("VACUUM INTO '{dest_sql}'"), [])?;

    // Cap the history of pre-migration rollback points. Migration bugs can
    // surface weeks later, so keep a generous tail; not unbounded because
    // every app upgrade adds one.
    crate::adapters::filesystem_data_management::prune_backups(
        system_backup_dir,
        crate::application::ports::BackupKind::PreMigration,
        10,
    );
    Ok(Some(dest))
}

pub fn open(path: &Path) -> anyhow::Result<Db> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrations().to_latest(&mut conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Errors specific to the org-database open path. Surfaced separately from
/// generic I/O failures so the orgs/registry layer can produce the right
/// `AppError` variant (e.g. `OrgNotFound` vs `Db { detail }`).
#[derive(Debug, thiserror::Error)]
pub enum OpenOrgError {
    #[error("org file does not exist")]
    NotFound,
    #[error("file is not a Terative org (application_id mismatch)")]
    ForeignFile,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Create a brand-new org database at `path`. The parent directory must
/// already exist. Errors if `path` already exists. After this returns,
/// `application_id` and the schema are in place.
pub fn create_org_db(path: &Path) -> anyhow::Result<Db> {
    if path.exists() {
        anyhow::bail!("org database already exists at {}", path.display());
    }
    open(path)
}

/// Open an existing org database. Validates the SQLite `application_id`
/// header *before* running migrations — otherwise a foreign SQLite file
/// would silently get a Terative schema injected into it.
pub fn open_org_db(path: &Path) -> Result<Db, OpenOrgError> {
    if !path.exists() {
        return Err(OpenOrgError::NotFound);
    }

    // Read application_id with a read-only connection. Empty files (just
    // created) report 0; anything else either matches or is foreign.
    let probe = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    let id: i32 = probe
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    drop(probe);

    if id != 0 && id != APPLICATION_ID {
        return Err(OpenOrgError::ForeignFile);
    }

    open(path).map_err(OpenOrgError::Other)
}

#[cfg(test)]
pub fn open_memory() -> Db {
    let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    migrations().to_latest(&mut conn).expect("run migrations");
    Arc::new(Mutex::new(conn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_set_application_id() {
        let db = open_memory();
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn snapshot_pre_migration_returns_none_when_no_db_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("does-not-exist.sqlite");
        let system_dir = tmp.path().join("system");
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir).unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn snapshot_pre_migration_returns_none_when_db_is_at_latest_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("live.sqlite");
        // `open` runs all migrations, so the resulting db is at max version.
        let _db = open(&db_path).unwrap();
        drop(_db);

        let system_dir = tmp.path().join("system");
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir).unwrap();
        assert!(out.is_none());
        assert!(!system_dir.exists(), "should not create dir unnecessarily");
    }

    #[test]
    fn snapshot_pre_migration_prunes_old_snapshots_to_keep_last_ten() {
        use crate::application::ports::BackupKind;
        use std::fs;

        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("live.sqlite");
        // Create a db at schema v0 so a migration is pending.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            drop(conn);
        }

        let system_dir = tmp.path().join("system");
        fs::create_dir_all(&system_dir).unwrap();

        // Pre-seed 11 premigration snapshots with distinct timestamps. After
        // one more is written by the snapshot call, pruning should leave 10.
        for d in 1..=11 {
            let name = crate::adapters::filesystem_data_management::format_backup_filename(
                BackupKind::PreMigration,
                chrono::Utc::now() - chrono::Duration::days(d as i64 * 3),
            );
            fs::write(system_dir.join(name), b"SQLite format 3\0").unwrap();
        }

        super::snapshot_pre_migration_if_pending(&db_path, &system_dir)
            .unwrap()
            .expect("snapshot should write");

        let count = fs::read_dir(&system_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.contains("premigration"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(count, 10);
    }

    #[test]
    fn snapshot_pre_migration_writes_premigration_backup_when_migrations_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("live.sqlite");
        // Seed a db at schema v0 (no migrations applied). We open an
        // empty rusqlite connection directly to skip the migration runner.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            // user_version defaults to 0 already; nothing else needed.
            drop(conn);
        }

        let system_dir = tmp.path().join("system");
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir)
            .unwrap()
            .expect("should write snapshot");
        assert!(out.exists());
        assert!(out.starts_with(&system_dir));
        let name = out.file_name().unwrap().to_str().unwrap();
        assert!(
            name.contains("premigration"),
            "expected 'premigration' in filename: {name}",
        );
    }

    #[test]
    fn migrations_leave_db_at_inside_schema_version() {
        use rusqlite_migration::SchemaVersion;
        let db = open_memory();
        let conn = db.lock();
        let current = super::migrations().current_version(&*conn).unwrap();
        assert!(
            matches!(current, SchemaVersion::Inside(_)),
            "expected Inside, got {current:?}",
        );
    }

    #[test]
    fn migrations_apply_cleanly_in_memory() {
        let db = open_memory();
        let conn = db.lock();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clients'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn singleton_settings_rows_are_seeded() {
        let db = open_memory();
        let conn = db.lock();
        let seller_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM seller_profile WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(seller_ok, 1);
        let cur_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM currency_config WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(cur_ok, 1);
        let prefs_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_preferences WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(prefs_ok, 1);
    }

    #[test]
    fn create_org_db_creates_file_with_application_id() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        let db = super::create_org_db(&path).unwrap();
        assert!(path.exists());
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn create_org_db_rejects_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        std::fs::write(&path, b"not really sqlite").unwrap();
        let err = super::create_org_db(&path).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn open_org_db_succeeds_for_terative_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path).unwrap();
        // Drop and re-open via open_org_db
        let db = super::open_org_db(&path).expect("should open existing terative db");
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn open_org_db_returns_not_found_for_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("missing.sqlite");
        let err = super::open_org_db(&path).unwrap_err();
        assert!(matches!(err, super::OpenOrgError::NotFound));
    }

    #[test]
    fn open_org_db_rejects_foreign_sqlite_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("foreign.sqlite");
        // Create a SQLite db with a different application_id (not Terative).
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA application_id = 999; CREATE TABLE x (a INT);")
            .unwrap();
        let id_after_set: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        drop(conn);
        assert_eq!(id_after_set, 999, "fixture did not set application_id");

        let err = super::open_org_db(&path).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::ForeignFile),
            "expected ForeignFile, got {err:?}"
        );
    }

    #[test]
    fn views_are_created() {
        let db = open_memory();
        let conn = db.lock();
        for view in ["v_invoice_payment_status", "v_client_balance", "v_aging_report"] {
            let c: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name=?1",
                    rusqlite::params![view],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(c, 1, "view {view} should exist");
        }
    }
}
