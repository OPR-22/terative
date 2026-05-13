use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};
use rusqlite_migration::{Migrations, SchemaVersion, M};
use zeroize::Zeroizing;

pub type Db = Arc<Mutex<Connection>>;

/// Escape a SQLCipher passphrase for embedding in a SQL literal. SQLite
/// doubles single quotes; the result is wrapped in `Zeroizing` so the
/// intermediate copy is wiped from the heap once the caller drops it.
pub(crate) fn escape_key_for_sql(key: &str) -> Zeroizing<String> {
    Zeroizing::new(key.replace('\'', "''"))
}

/// Issue `PRAGMA key = '<escaped>'` on the given connection. Centralised so
/// every SQLCipher key-set site shares the same escape rule and lifetime
/// for the intermediate buffer.
fn set_pragma_key(conn: &Connection, key: &str) -> rusqlite::Result<()> {
    let escaped = escape_key_for_sql(key);
    let stmt = Zeroizing::new(format!("PRAGMA key = '{}'", &*escaped));
    conn.execute_batch(&stmt)
}

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
    key: Option<&str>,
) -> anyhow::Result<Option<PathBuf>> {
    if !db_path.exists() {
        // Fresh install; no data to protect.
        return Ok(None);
    }

    // Plaintext: read-only is enough for VACUUM INTO. Encrypted: we'll
    // ATTACH a freshly-created target db below, which requires the
    // source connection to be read-write so the attached handle inherits
    // write permissions.
    let conn = match key {
        Some(k) => {
            let c = Connection::open(db_path)?;
            set_pragma_key(&c, k)?;
            c
        }
        None => Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?,
    };
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
    if let Some(k) = key {
        // SQLCipher: VACUUM INTO writes a *plaintext* db. Use
        // sqlcipher_export to produce an encrypted snapshot under the
        // same key. We deliberately don't re-stamp user_version on the
        // target here — this is a *pre*-migration snapshot, so leaving
        // the schema at v0 (or whatever the source was) is what makes
        // the rollback point useful.
        let escaped_key = escape_key_for_sql(k);
        let stmt = Zeroizing::new(format!(
            "ATTACH DATABASE '{dest_sql}' AS backup KEY '{}';\
             SELECT sqlcipher_export('backup');\
             DETACH DATABASE backup;",
            &*escaped_key,
        ));
        conn.execute_batch(&stmt)?;
    } else {
        conn.execute(&format!("VACUUM INTO '{dest_sql}'"), [])?;
    }

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

/// Open a Terative database, optionally encrypted with SQLCipher.
///
/// When `key` is `None`, the file is treated as a plaintext SQLite db.
/// When `key` is `Some`, SQLCipher's `PRAGMA key` is issued before any other
/// statement; a wrong (or missing) key for an encrypted file surfaces as
/// [`OpenOrgError::WrongPassword`], detected via the SQLite NotADatabase
/// error code on the canary read of `sqlite_master`. Passing a key against
/// a plaintext file (or vice versa) also produces `WrongPassword` — the
/// header bytes won't decrypt and the canary fails the same way.
///
/// Once the key has been accepted, `application_id` is checked so a foreign
/// SQLCipher db that happened to use the same passphrase cannot be
/// migrated by mistake.
pub fn open_with_key(path: &Path, key: Option<&str>) -> Result<Db, OpenOrgError> {
    let mut conn = Connection::open(path).map_err(|e| OpenOrgError::Other(e.into()))?;

    if let Some(k) = key {
        // SQLCipher requires `PRAGMA key` before any other operation on the
        // connection.
        set_pragma_key(&conn, k)
            .map_err(|e| OpenOrgError::Other(anyhow::anyhow!("set key: {e}")))?;
    }

    // Canary read: if the key is wrong or the db is encrypted without a
    // key, SQLCipher rejects this with SQLITE_NOTADB. For a plaintext db
    // opened with `None`, or an encrypted db opened with the right key,
    // this is a cheap no-op (zero rows on a brand-new file).
    if let Err(e) = conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
    {
        if matches!(
            e.sqlite_error_code(),
            Some(rusqlite::ErrorCode::NotADatabase)
        ) {
            return Err(OpenOrgError::WrongPassword);
        }
        return Err(OpenOrgError::Other(e.into()));
    }

    // Foreign-file guard. We check after the canary so it covers both
    // plaintext and encrypted dbs — for encrypted dbs the header bytes
    // are ciphertext until the key has been accepted. `id == 0` means
    // the file is brand-new (no migration has run yet); the migration
    // runner below will stamp APPLICATION_ID on it.
    let id: i32 = conn
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    if id != 0 && id != APPLICATION_ID {
        return Err(OpenOrgError::ForeignFile);
    }

    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    migrations()
        .to_latest(&mut conn)
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn open(path: &Path) -> anyhow::Result<Db> {
    open_with_key(path, None).map_err(|e| match e {
        OpenOrgError::Other(err) => err,
        other => anyhow::anyhow!(other.to_string()),
    })
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
    #[error("wrong password (or db is encrypted and none was supplied)")]
    WrongPassword,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Create a brand-new org database at `path`. The parent directory must
/// already exist. Errors if `path` already exists. When `key` is `Some`,
/// the freshly-created file is encrypted with SQLCipher under that key.
pub fn create_org_db(path: &Path, key: Option<&str>) -> Result<Db, OpenOrgError> {
    if path.exists() {
        return Err(OpenOrgError::Other(anyhow::anyhow!(
            "org database already exists at {}",
            path.display()
        )));
    }
    open_with_key(path, key)
}

/// Open an existing org database, optionally with a SQLCipher key.
///
/// `open_with_key` performs the actual `application_id` validation post-key;
/// this wrapper exists only to convert a missing file into the dedicated
/// `NotFound` variant (the bare open would surface a generic I/O error).
pub fn open_org_db(path: &Path, key: Option<&str>) -> Result<Db, OpenOrgError> {
    if !path.exists() {
        return Err(OpenOrgError::NotFound);
    }
    open_with_key(path, key)
}

/// Convert the on-disk org database between encryption states.
///
/// Caller must ensure the file at `db_path` is not currently open by any
/// other connection (the active org should be closed first). Writes the
/// converted db to a sibling temp file, atomic-renames over the original,
/// and cleans up the temp on failure. For a same-mode change of key
/// (encrypted → encrypted), this is functionally a rekey; for cross-mode
/// changes it uses `sqlcipher_export`.
pub fn change_org_db_key(
    db_path: &Path,
    current_key: Option<&str>,
    new_key: Option<&str>,
) -> Result<(), OpenOrgError> {
    if !db_path.exists() {
        return Err(OpenOrgError::NotFound);
    }

    let parent = db_path
        .parent()
        .ok_or_else(|| OpenOrgError::Other(anyhow::anyhow!("db path has no parent")))?;
    let stem = db_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| OpenOrgError::Other(anyhow::anyhow!("db path has no filename")))?;
    let tmp = parent.join(format!(".{stem}.rekey-tmp"));
    if tmp.exists() {
        std::fs::remove_file(&tmp)
            .map_err(|e| OpenOrgError::Other(e.into()))?;
    }

    let result = (|| -> Result<(), OpenOrgError> {
        let conn = Connection::open(db_path).map_err(|e| OpenOrgError::Other(e.into()))?;
        if let Some(k) = current_key {
            set_pragma_key(&conn, k).map_err(|_| OpenOrgError::WrongPassword)?;
        }
        // Canary verifies the current key actually unlocks the file.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
            .map_err(|e| match e.sqlite_error_code() {
                Some(rusqlite::ErrorCode::NotADatabase) => OpenOrgError::WrongPassword,
                _ => OpenOrgError::Other(e.into()),
            })?;

        // sqlcipher_export copies schema + data but not file-format
        // PRAGMAs like user_version/application_id. Read user_version
        // off the source so the target stays at the same schema (re-
        // running every migration on the new file would be a no-op for
        // a well-formed source, but if it ever wasn't this would mask
        // a real bug). `application_id` is stamped from the constant
        // unconditionally — by construction we only rekey files we
        // already opened as a Terative org.
        let src_user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| OpenOrgError::Other(e.into()))?;

        let tmp_sql = tmp.to_string_lossy().replace('\'', "''");
        let new_key_escaped = escape_key_for_sql(new_key.unwrap_or(""));
        // `KEY ''` produces a plaintext target. Any non-empty key produces
        // an encrypted target. sqlcipher_export copies every page through
        // the destination's keying.
        let stmt = Zeroizing::new(format!(
            "ATTACH DATABASE '{tmp_sql}' AS rekey_target KEY '{}';\
             SELECT sqlcipher_export('rekey_target');\
             PRAGMA rekey_target.user_version = {src_user_version};\
             PRAGMA rekey_target.application_id = {APPLICATION_ID};\
             DETACH DATABASE rekey_target;",
            &*new_key_escaped,
        ));
        conn.execute_batch(&stmt)
            .map_err(|e| OpenOrgError::Other(e.into()))?;
        drop(conn);

        std::fs::rename(&tmp, db_path).map_err(|e| OpenOrgError::Other(e.into()))?;
        // The previous WAL/SHM sidecars referenced the old keying; remove
        // them so the next open starts fresh under the new key.
        let _ = std::fs::remove_file(sidecar(db_path, "-wal"));
        let _ = std::fs::remove_file(sidecar(db_path, "-shm"));
        Ok(())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

fn sidecar(db_path: &Path, suffix: &str) -> PathBuf {
    let mut p = db_path.as_os_str().to_os_string();
    p.push(suffix);
    PathBuf::from(p)
}

/// Cheap, read-only check that `key` (or its absence) unlocks the file
/// at `db_path`. Returns `WrongPassword` when SQLCipher rejects the
/// key, `Ok(())` for missing files (fresh orgs) and for keys that
/// successfully decrypt the header. Meant to be called before any
/// snapshot/migration step that could otherwise surface a NotADatabase
/// error as a generic `Internal` failure.
pub fn validate_org_key(db_path: &Path, key: Option<&str>) -> Result<(), OpenOrgError> {
    if !db_path.exists() {
        return Ok(());
    }
    let probe = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| OpenOrgError::Other(e.into()))?;
    if let Some(k) = key {
        set_pragma_key(&probe, k)
            .map_err(|e| OpenOrgError::Other(anyhow::anyhow!("set key: {e}")))?;
    }
    match probe.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)) {
        Ok(_) => Ok(()),
        Err(e) => {
            if matches!(
                e.sqlite_error_code(),
                Some(rusqlite::ErrorCode::NotADatabase)
            ) {
                Err(OpenOrgError::WrongPassword)
            } else {
                Err(OpenOrgError::Other(e.into()))
            }
        }
    }
}

/// Classification produced by [`probe_org_file`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrgFileKind {
    /// Plaintext Terative db with a valid `application_id`.
    Plaintext,
    /// File is empty / brand-new (size 0 or 0 rows in sqlite_master).
    /// Treated as plaintext by the open path.
    Empty,
    /// SQLite header doesn't read — most likely SQLCipher-encrypted.
    Encrypted,
    /// Foreign SQLite file (wrong `application_id`).
    Foreign,
}

/// Cheap, read-only classification of an org candidate file. Used by the
/// registry to populate `has_password` on the picker without ever asking
/// for a key. Encrypted files report `Encrypted` because the header is
/// itself ciphertext.
pub fn probe_org_file(path: &Path) -> std::io::Result<OrgFileKind> {
    let meta = std::fs::metadata(path)?;
    if meta.len() == 0 {
        return Ok(OrgFileKind::Empty);
    }
    let probe = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => return Ok(OrgFileKind::Encrypted),
    };
    match probe.query_row("PRAGMA application_id", [], |r| r.get::<_, i32>(0)) {
        Ok(id) if id == APPLICATION_ID => Ok(OrgFileKind::Plaintext),
        Ok(0) => Ok(OrgFileKind::Empty),
        Ok(_) => Ok(OrgFileKind::Foreign),
        Err(e) if matches!(e.sqlite_error_code(), Some(rusqlite::ErrorCode::NotADatabase)) => {
            Ok(OrgFileKind::Encrypted)
        }
        Err(_) => Ok(OrgFileKind::Encrypted),
    }
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
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir, None).unwrap();
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
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir, None).unwrap();
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

        super::snapshot_pre_migration_if_pending(&db_path, &system_dir, None)
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
        let out = super::snapshot_pre_migration_if_pending(&db_path, &system_dir, None)
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
        let db = super::create_org_db(&path, None).unwrap();
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
        let err = super::create_org_db(&path, None).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn open_org_db_succeeds_for_terative_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, None).unwrap();
        // Drop and re-open via open_org_db
        let db = super::open_org_db(&path, None).expect("should open existing terative db");
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
        let err = super::open_org_db(&path, None).unwrap_err();
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

        let err = super::open_org_db(&path, None).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::ForeignFile),
            "expected ForeignFile, got {err:?}"
        );
    }

    #[test]
    fn validate_org_key_returns_wrong_password_for_bad_key() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("pw")).unwrap();

        assert!(super::validate_org_key(&path, Some("pw")).is_ok());
        assert!(matches!(
            super::validate_org_key(&path, Some("wrong")).unwrap_err(),
            super::OpenOrgError::WrongPassword
        ));
        assert!(matches!(
            super::validate_org_key(&path, None).unwrap_err(),
            super::OpenOrgError::WrongPassword
        ));
    }

    #[test]
    fn validate_org_key_is_ok_for_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("does-not-exist.sqlite");
        assert!(super::validate_org_key(&path, None).is_ok());
        assert!(super::validate_org_key(&path, Some("anything")).is_ok());
    }

    #[test]
    fn snapshot_pre_migration_works_for_encrypted_db_and_keeps_snapshot_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("live.sqlite");
        // Create an unmigrated encrypted db so snapshot has work to do.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA key = 'hunter2';").unwrap();
            // Touch the db so SQLCipher initialises page 1 with the key.
            conn.execute_batch("CREATE TABLE _seed (a INT); DROP TABLE _seed;")
                .unwrap();
        }

        let system_dir = tmp.path().join("system");
        let snap =
            super::snapshot_pre_migration_if_pending(&db_path, &system_dir, Some("hunter2"))
                .unwrap()
                .expect("snapshot should run");

        // Snapshot must be readable only with the same key, never plaintext.
        let snap_bytes = std::fs::read(&snap).unwrap();
        assert!(!snap_bytes.starts_with(b"SQLite format 3\0"));
        let opened = super::open_with_key(&snap, Some("hunter2")).expect("reopen snapshot");
        let conn = opened.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    // === SQLCipher encryption ===

    #[test]
    fn open_with_key_creates_encrypted_db_and_reopens_with_same_key() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        {
            let _db = super::open_with_key(&path, Some("hunter2"))
                .expect("create encrypted db");
        }
        let db = super::open_with_key(&path, Some("hunter2")).expect("reopen with same key");
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn open_with_key_rejects_wrong_key() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        {
            let _db = super::open_with_key(&path, Some("hunter2")).unwrap();
        }
        let err = super::open_with_key(&path, Some("wrong")).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::WrongPassword),
            "expected WrongPassword, got {err:?}"
        );
    }

    #[test]
    fn open_with_key_rejects_encrypted_db_without_key() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        {
            let _db = super::open_with_key(&path, Some("hunter2")).unwrap();
        }
        let err = super::open_with_key(&path, None).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::WrongPassword),
            "expected WrongPassword for missing key, got {err:?}"
        );
    }

    #[test]
    fn encrypted_db_file_does_not_start_with_sqlite_magic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        {
            let _db = super::open_with_key(&path, Some("hunter2")).unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.starts_with(b"SQLite format 3\0"),
            "encrypted db must not begin with the SQLite plaintext magic",
        );
    }

    #[test]
    fn plaintext_db_starts_with_sqlite_magic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        {
            let _db = super::open_with_key(&path, None).unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3\0"));
    }

    #[test]
    fn probe_org_file_classifies_plaintext_encrypted_foreign_and_empty() {
        let tmp = tempfile::tempdir().unwrap();

        let plaintext = tmp.path().join("plaintext.sqlite");
        super::create_org_db(&plaintext, None).unwrap();
        assert_eq!(
            super::probe_org_file(&plaintext).unwrap(),
            super::OrgFileKind::Plaintext,
        );

        let encrypted = tmp.path().join("encrypted.sqlite");
        super::create_org_db(&encrypted, Some("hunter2")).unwrap();
        assert_eq!(
            super::probe_org_file(&encrypted).unwrap(),
            super::OrgFileKind::Encrypted,
        );

        let foreign = tmp.path().join("foreign.sqlite");
        {
            let c = rusqlite::Connection::open(&foreign).unwrap();
            c.execute_batch("PRAGMA application_id = 999; CREATE TABLE x (a INT);")
                .unwrap();
        }
        assert_eq!(
            super::probe_org_file(&foreign).unwrap(),
            super::OrgFileKind::Foreign,
        );

        let empty = tmp.path().join("empty.sqlite");
        std::fs::write(&empty, b"").unwrap();
        assert_eq!(
            super::probe_org_file(&empty).unwrap(),
            super::OrgFileKind::Empty,
        );
    }

    #[test]
    fn open_org_db_with_correct_key_succeeds_for_encrypted_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("hunter2")).unwrap();
        let db = super::open_org_db(&path, Some("hunter2"))
            .expect("open with correct key");
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn open_org_db_with_wrong_key_returns_wrong_password() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("hunter2")).unwrap();
        let err = super::open_org_db(&path, Some("nope")).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::WrongPassword),
            "expected WrongPassword, got {err:?}"
        );
    }

    #[test]
    fn change_org_db_key_plaintext_to_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, None).unwrap();

        super::change_org_db_key(&path, None, Some("hunter2")).unwrap();

        assert!(matches!(
            super::open_with_key(&path, None).unwrap_err(),
            super::OpenOrgError::WrongPassword
        ));
        let db = super::open_with_key(&path, Some("hunter2")).unwrap();
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn change_org_db_key_encrypted_to_plaintext() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("pw1")).unwrap();

        super::change_org_db_key(&path, Some("pw1"), None).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3\0"));
        let db = super::open_with_key(&path, None).unwrap();
        let conn = db.lock();
        let id: i32 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .unwrap();
        assert_eq!(id, super::APPLICATION_ID);
    }

    #[test]
    fn change_org_db_key_encrypted_to_encrypted_change_password() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("pw1")).unwrap();

        super::change_org_db_key(&path, Some("pw1"), Some("pw2")).unwrap();

        assert!(super::open_with_key(&path, Some("pw1")).is_err());
        let _ = super::open_with_key(&path, Some("pw2")).expect("reopen with pw2");
    }

    #[test]
    fn change_org_db_key_rejects_wrong_current_password() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("pw1")).unwrap();

        let err = super::change_org_db_key(&path, Some("wrong"), Some("pw2")).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::WrongPassword),
            "got {err:?}"
        );

        // Original file is unchanged: still opens with pw1.
        let _ = super::open_with_key(&path, Some("pw1")).expect("original key still works");
    }

    /// Regression for the "right key on a foreign SQLCipher db" case: once
    /// `PRAGMA key` has been accepted, `application_id` becomes readable
    /// even on encrypted files. Without the post-canary check, the
    /// foreign db would have Terative migrations injected on top of its
    /// existing tables.
    #[test]
    fn open_with_key_rejects_encrypted_db_with_foreign_application_id() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("foreign.sqlite");
        // Build an encrypted db with a non-Terative application_id and a
        // table that proves we're talking to a real db (not just an empty
        // file with `application_id` set in the header).
        {
            let c = rusqlite::Connection::open(&path).unwrap();
            super::set_pragma_key(&c, "shared").unwrap();
            c.execute_batch(
                "PRAGMA application_id = 12345678;\
                 CREATE TABLE foreign_table (x INT);",
            )
            .unwrap();
        }

        let err = super::open_with_key(&path, Some("shared")).unwrap_err();
        assert!(
            matches!(err, super::OpenOrgError::ForeignFile),
            "expected ForeignFile, got {err:?}",
        );

        // And it must not have run our migrations on the foreign db.
        let c = rusqlite::Connection::open(&path).unwrap();
        super::set_pragma_key(&c, "shared").unwrap();
        let has_clients: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clients'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(has_clients, 0, "migrations must not have run on foreign db");
    }

    #[test]
    fn rekey_changes_existing_encrypted_db_password() {
        // SQLCipher's `PRAGMA rekey` only mutates the key of an already-
        // encrypted db. Plaintext<->encrypted conversions go through
        // `sqlcipher_export` instead and live in the password use case.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("acme.sqlite");
        super::create_org_db(&path, Some("pw1")).unwrap();

        {
            let db = super::open_with_key(&path, Some("pw1")).unwrap();
            let conn = db.lock();
            conn.execute_batch("PRAGMA rekey = 'pw2';").unwrap();
        }
        assert!(super::open_with_key(&path, Some("pw1")).is_err());
        let _ = super::open_with_key(&path, Some("pw2")).expect("reopen with pw2");
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
