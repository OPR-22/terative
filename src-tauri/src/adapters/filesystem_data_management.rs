use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use rusqlite::{Connection, OpenFlags};
use rusqlite_migration::SchemaVersion;

use crate::adapters::sqlite::connection::{migrations, Db, APPLICATION_ID};
use crate::application::ports::{BackupKind, BackupMetadata, DataManagement, SettingsRepository};
use crate::application::RepoError;

pub struct FilesystemDataManagement {
    db: Db,
    db_path: PathBuf,
    settings: Arc<dyn SettingsRepository>,
    default_user_backup_dir: PathBuf,
    system_backup_dir: PathBuf,
}

impl FilesystemDataManagement {
    pub fn new(
        db: Db,
        db_path: PathBuf,
        settings: Arc<dyn SettingsRepository>,
        default_user_backup_dir: PathBuf,
        system_backup_dir: PathBuf,
    ) -> Self {
        Self {
            db,
            db_path,
            settings,
            default_user_backup_dir,
            system_backup_dir,
        }
    }

    /// Reads the user's backup dir preference, falling back to the
    /// app-data default when unset. Always acquires and releases the db
    /// lock before any VACUUM INTO call to avoid reentrant locking.
    fn resolve_user_backup_dir(&self) -> Result<PathBuf, RepoError> {
        let prefs = self.settings.get_app_preferences()?;
        let configured = prefs.user_backup_dir.trim();
        Ok(if configured.is_empty() {
            self.default_user_backup_dir.clone()
        } else {
            PathBuf::from(configured)
        })
    }

    /// Writes a WAL-safe consistent copy of the live database to `dest` using
    /// SQLite's `VACUUM INTO`. This captures any commits still sitting in the
    /// WAL that a raw `fs::copy` of the main file would miss. The target path
    /// must not already exist.
    fn vacuum_into(&self, dest: &Path) -> Result<(), RepoError> {
        // `VACUUM INTO` does not accept bound parameters for the target path,
        // so we interpolate and escape SQL single quotes by doubling them.
        let dest_sql = dest.to_string_lossy().replace('\'', "''");
        let conn = self.db.lock();
        conn.execute(&format!("VACUUM INTO '{dest_sql}'"), [])
            .map_err(|e| storage(format!("vacuum into {}: {e}", dest.display())))?;
        Ok(())
    }

    /// Writes a backup of the given [`BackupKind`] into `dir`, using the
    /// canonical filename format.
    pub(crate) fn write_backup(
        &self,
        kind: BackupKind,
        dir: &Path,
    ) -> Result<PathBuf, RepoError> {
        fs::create_dir_all(dir).map_err(|e| storage(format!("create backup dir: {e}")))?;
        let path = dir.join(format_backup_filename(kind, Utc::now()));
        self.vacuum_into(&path)?;
        Ok(path)
    }
}

/// Canonical filename: `terative-YYYYMMDD-HHMMSS-<kind>.sqlite`. Timestamp
/// first so file managers sort chronologically; kind suffix gives us a trivial
/// parse path.
pub(crate) fn format_backup_filename(kind: BackupKind, timestamp: DateTime<Utc>) -> String {
    format!(
        "terative-{}-{}.sqlite",
        timestamp.format("%Y%m%d-%H%M%S"),
        kind_suffix(kind),
    )
}

fn kind_suffix(kind: BackupKind) -> &'static str {
    match kind {
        BackupKind::Manual => "manual",
        BackupKind::Auto => "auto",
        BackupKind::PreRestore => "prerestore",
        BackupKind::PreMigration => "premigration",
    }
}

fn parse_kind_suffix(s: &str) -> Option<BackupKind> {
    match s {
        "manual" => Some(BackupKind::Manual),
        "auto" => Some(BackupKind::Auto),
        "prerestore" => Some(BackupKind::PreRestore),
        "premigration" => Some(BackupKind::PreMigration),
        _ => None,
    }
}

/// Parses a canonical backup filename. Returns None for anything that doesn't
/// match the expected grammar — callers use this to filter unrelated files
/// out of the scan.
fn parse_backup_filename(name: &str) -> Option<(DateTime<Utc>, BackupKind)> {
    let stem = name.strip_suffix(".sqlite")?;
    let rest = stem.strip_prefix("terative-")?;
    let (datetime_part, kind_part) = rest.rsplit_once('-')?;
    let kind = parse_kind_suffix(kind_part)?;
    let naive = NaiveDateTime::parse_from_str(datetime_part, "%Y%m%d-%H%M%S").ok()?;
    Some((Utc.from_utc_datetime(&naive), kind))
}

/// Returns every well-formed backup file under `dir`. Malformed/unrelated
/// files are skipped silently.
fn scan_backup_dir(dir: &Path) -> Vec<BackupMetadata> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            let (timestamp, kind) = parse_backup_filename(name)?;
            let size_bytes = entry.metadata().ok()?.len();
            Some(BackupMetadata {
                path,
                timestamp,
                kind,
                size_bytes,
            })
        })
        .collect()
}

fn storage(msg: impl Into<String>) -> RepoError {
    RepoError::Storage(msg.into())
}

/// SQLite's file format starts with the magic bytes "SQLite format 3\x00".
fn validate_sqlite_magic(path: &Path) -> Result<(), RepoError> {
    let bytes = fs::read(path).map_err(|e| storage(format!("read source: {e}")))?;
    const MAGIC: &[u8] = b"SQLite format 3\0";
    if bytes.len() < MAGIC.len() || &bytes[..MAGIC.len()] != MAGIC {
        return Err(storage("source is not a valid SQLite database"));
    }
    Ok(())
}

/// Validates a candidate restore source: magic bytes, SQLite-level integrity,
/// FK consistency, and a Terative-specific `application_id` stamped into the
/// SQLite header by the initial migration.
fn validate_restore_source(path: &Path) -> Result<(), RepoError> {
    validate_sqlite_magic(path)?;

    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| storage(format!("open source: {e}")))?;

    let app_id: i32 = conn
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(|e| storage(format!("read application_id: {e}")))?;
    if app_id != APPLICATION_ID {
        return Err(storage(
            "source is not a terative database (application_id mismatch)",
        ));
    }

    // Ask the migration runner itself whether the backup's schema version is
    // one this binary knows about. `Outside` means the backup carries more
    // migrations than we do — i.e., it was taken with a newer app version.
    match migrations()
        .current_version(&conn)
        .map_err(|e| storage(format!("schema version check: {e}")))?
    {
        SchemaVersion::Outside(v) => {
            return Err(storage(format!(
                "source is from a newer app version (schema v{v}) — please update the app"
            )));
        }
        SchemaVersion::Inside(_) | SchemaVersion::NoneSet => {}
    }

    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| storage(format!("integrity_check failed: {e}")))?;
    if integrity != "ok" {
        return Err(storage(format!(
            "source failed integrity check: {integrity}"
        )));
    }

    let mut stmt = conn
        .prepare("PRAGMA foreign_key_check")
        .map_err(|e| storage(format!("prepare foreign_key_check: {e}")))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| storage(format!("foreign_key_check: {e}")))?;
    if rows
        .next()
        .map_err(|e| storage(format!("foreign_key_check: {e}")))?
        .is_some()
    {
        return Err(storage("source has foreign key violations"));
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
        // VACUUM INTO refuses to overwrite, so clear any existing target first.
        if destination.exists() {
            fs::remove_file(destination)
                .map_err(|e| storage(format!("overwrite destination: {e}")))?;
        }
        self.vacuum_into(destination)?;
        Ok(destination.to_path_buf())
    }

    fn create_backup(&self) -> Result<PathBuf, RepoError> {
        let user_dir = self.resolve_user_backup_dir()?;
        self.write_backup(BackupKind::Manual, &user_dir)
    }

    fn restore_database(&self, source: &Path) -> Result<PathBuf, RepoError> {
        validate_restore_source(source)?;

        // Snapshot the live db BEFORE touching it, so the restore is
        // reversible. If this fails, the live db is never modified.
        let snapshot = self.write_backup(BackupKind::PreRestore, &self.system_backup_dir)?;

        // Cap the history of rollback points at a handful — restore mistakes
        // are noticed immediately; older prerestore snapshots are dead weight.
        prune_backups(&self.system_backup_dir, BackupKind::PreRestore, 5);

        // Best-effort atomic replace: copy source into a sibling temp file,
        // then rename over the live path. `source` is never moved.
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

        // The live connection's WAL/SHM sidecars reference the previous main
        // file. Remove them so the next process opens the restored file with
        // a fresh WAL. Errors here are non-fatal — the imminent restart will
        // close all handles anyway, and SQLite tolerates absent sidecars.
        let wal = sidecar_path(&self.db_path, "-wal");
        let shm = sidecar_path(&self.db_path, "-shm");
        let _ = fs::remove_file(wal);
        let _ = fs::remove_file(shm);

        Ok(snapshot)
    }

    fn list_backups(&self) -> Result<Vec<BackupMetadata>, RepoError> {
        let user_dir = self.resolve_user_backup_dir()?;
        let mut out = scan_backup_dir(&user_dir);
        out.extend(scan_backup_dir(&self.system_backup_dir));
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(out)
    }

    fn delete_backup(&self, path: &Path) -> Result<(), RepoError> {
        let parent = path
            .parent()
            .ok_or_else(|| storage("backup path has no parent"))?;
        let user_dir = self.resolve_user_backup_dir()?;
        let in_user = parent == user_dir.as_path();
        let in_system = parent == self.system_backup_dir.as_path();
        if !in_user && !in_system {
            return Err(storage(format!(
                "refusing to delete file outside configured backup dirs: {}",
                path.display()
            )));
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| storage("backup path has no filename"))?;
        let (_, kind) = parse_backup_filename(name)
            .ok_or_else(|| storage(format!("not a canonical backup filename: {name}")))?;
        match kind {
            BackupKind::Manual | BackupKind::Auto => {}
            BackupKind::PreRestore | BackupKind::PreMigration => {
                return Err(storage(
                    "refusing to delete system backup (pre-restore/pre-migration)",
                ));
            }
        }
        fs::remove_file(path).map_err(|e| storage(format!("delete backup: {e}")))?;
        Ok(())
    }

    fn auto_backup_if_due(&self) -> Result<Option<PathBuf>, RepoError> {
        // Read prefs + release DB lock before any vacuum step (which re-locks).
        let prefs = self.settings.get_app_preferences()?;
        if !prefs.auto_backup_enabled {
            return Ok(None);
        }

        let user_dir = self.resolve_user_backup_dir()?;
        let newest_auto_ts = scan_backup_dir(&user_dir)
            .into_iter()
            .filter(|b| b.kind == BackupKind::Auto)
            .map(|b| b.timestamp)
            .max();

        let interval = chrono::Duration::hours(prefs.auto_backup_interval_hours as i64);
        let due = match newest_auto_ts {
            None => true,
            Some(ts) => Utc::now() - ts >= interval,
        };
        if !due {
            return Ok(None);
        }

        let path = self.write_backup(BackupKind::Auto, &user_dir)?;

        // Apply retention only to auto backups in the user dir. Manual backups
        // and system-dir snapshots are never auto-pruned.
        if let crate::domain::settings::RetentionMode::KeepLast = prefs.retention_mode {
            prune_backups(&user_dir, BackupKind::Auto, prefs.retention_count as usize);
        }

        Ok(Some(path))
    }
}

/// Deletes the oldest backups of `kind` in `dir` so only `keep` remain.
/// Errors deleting individual files are logged and swallowed — the next
/// trigger will try again.
pub(crate) fn prune_backups(dir: &Path, kind: BackupKind, keep: usize) {
    let mut matches: Vec<BackupMetadata> = scan_backup_dir(dir)
        .into_iter()
        .filter(|b| b.kind == kind)
        .collect();
    // Newest first; everything beyond `keep` is excess.
    matches.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    for excess in matches.into_iter().skip(keep) {
        if let Err(e) = fs::remove_file(&excess.path) {
            eprintln!(
                "backup retention: failed to delete {}: {e}",
                excess.path.display()
            );
        }
    }
}

fn sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut p = db_path.as_os_str().to_os_string();
    p.push(suffix);
    PathBuf::from(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open;

    /// Build a real sqlite file via the real migrations. Returns the open
    /// connection so callers that need a live handle (e.g. the adapter under
    /// test) can hold it; callers that only need the file on disk can drop
    /// the result immediately.
    fn seed_real_db(path: &Path) -> Db {
        open(path).expect("open real sqlite")
    }

    fn client_count(db_path: &Path, id: &str) -> i64 {
        let conn = rusqlite::Connection::open(db_path).unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM clients WHERE id = ?1",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn insert_client(db: &Db, id: &str, name: &str) {
        let conn = db.lock();
        conn.execute(
            "INSERT INTO clients (id, name, active, created_at) VALUES (?1, ?2, 1, ?3)",
            rusqlite::params![id, name, "2026-01-01T00:00:00Z"],
        )
        .unwrap();
    }

    /// Standard test setup: creates user/system dirs under `root` and builds
    /// a mgr configured to use them. Returns `(mgr, default_user_dir, system_dir)`.
    /// The mgr's settings repo is bound to the same `db`, so tests can mutate
    /// `user_backup_dir` in app_preferences to exercise the resolution path.
    fn build_mgr(
        root: &Path,
        live_path: PathBuf,
        db: Db,
    ) -> (FilesystemDataManagement, PathBuf, PathBuf) {
        let user_dir = root.join("backups-user");
        let system_dir = root.join("backups-system");
        let settings = Arc::new(crate::adapters::sqlite::SqliteSettingsRepository::new(
            db.clone(),
        ));
        let mgr = FilesystemDataManagement::new(
            db,
            live_path,
            settings,
            user_dir.clone(),
            system_dir.clone(),
        );
        (mgr, user_dir, system_dir)
    }

    // -------- filename parser ---------------------------------------------

    #[test]
    fn format_and_parse_backup_filename_roundtrip() {
        let ts = Utc.with_ymd_and_hms(2026, 4, 21, 14, 30, 22).unwrap();
        for kind in [
            BackupKind::Manual,
            BackupKind::Auto,
            BackupKind::PreRestore,
            BackupKind::PreMigration,
        ] {
            let name = format_backup_filename(kind, ts);
            let (parsed_ts, parsed_kind) = parse_backup_filename(&name)
                .unwrap_or_else(|| panic!("failed to parse {name}"));
            assert_eq!(parsed_ts, ts, "{name}");
            assert_eq!(parsed_kind, kind, "{name}");
        }
    }

    #[test]
    fn parse_backup_filename_rejects_unrelated_names() {
        // Extra validation cases to make sure the parser can't be tricked.
        for bad in [
            "random.sqlite",                      // missing terative prefix
            "terative-manual.sqlite",             // missing timestamp
            "terative-20260421-143022.sqlite",    // missing kind
            "terative-20260421-143022-manual",    // missing .sqlite suffix
            "terative-20260421-143022-bogus.sqlite", // unknown kind
            "terative-notadate-000000-manual.sqlite", // non-parseable timestamp
        ] {
            assert!(
                parse_backup_filename(bad).is_none(),
                "{bad} should not parse",
            );
        }
    }

    // -------- export -------------------------------------------------------

    #[test]
    fn export_database_writes_logical_copy_of_live() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("live.sqlite");
        let db = seed_real_db(&src);
        insert_client(&db, "export-row", "Exported");
        let dest = tmp.path().join("exports").join("snapshot.sqlite");

        let (mgr, _, _) = build_mgr(tmp.path(), src, db);
        let path = mgr.export_database(&dest).unwrap();

        assert_eq!(path, dest);
        assert!(dest.exists());
        assert_eq!(client_count(&dest, "export-row"), 1);
    }

    // -------- create_backup ------------------------------------------------

    #[test]
    fn create_backup_writes_manual_backup_into_user_dir_with_canonical_name() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        let path = mgr.create_backup().unwrap();

        assert!(path.starts_with(&user_dir), "expected in user dir: {path:?}");
        let name = path.file_name().unwrap().to_str().unwrap();
        let (_, kind) =
            parse_backup_filename(name).expect("canonical filename must parse");
        assert_eq!(kind, BackupKind::Manual);
    }

    #[test]
    fn create_backup_uses_user_backup_dir_from_preferences_when_set() {
        use crate::application::ports::SettingsRepository;
        use crate::domain::settings::AppPreferences;

        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        // Configure the user_backup_dir preference to point somewhere other
        // than the default passed to the constructor.
        let configured = tmp.path().join("my-custom-backups");
        let settings = crate::adapters::sqlite::SqliteSettingsRepository::new(db.clone());
        settings
            .set_app_preferences(&AppPreferences {
                user_backup_dir: configured.to_string_lossy().to_string(),
                ..Default::default()
            })
            .unwrap();

        let (mgr, default_user_dir, _) = build_mgr(tmp.path(), live, db);
        let path = mgr.create_backup().unwrap();

        assert!(
            path.starts_with(&configured),
            "expected backup in configured dir {configured:?}, got {path:?}",
        );
        assert!(
            !default_user_dir.exists(),
            "default dir must not be created when a custom one is set",
        );
    }

    #[test]
    fn create_backup_creates_missing_user_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        // Use a deeply-nested path as user dir that does not yet exist.
        let user_dir = tmp.path().join("a").join("b");
        let system_dir = tmp.path().join("system");
        let settings = Arc::new(crate::adapters::sqlite::SqliteSettingsRepository::new(
            db.clone(),
        ));
        let mgr = FilesystemDataManagement::new(
            db,
            live,
            settings,
            user_dir.clone(),
            system_dir,
        );

        let path = mgr.create_backup().unwrap();
        assert!(path.exists());
        assert!(user_dir.exists());
    }

    /// Regression guard: with WAL mode on, a raw `fs::copy` of the main file
    /// would miss commits still sitting in the WAL. `VACUUM INTO` reads through
    /// the live connection and captures them.
    #[test]
    fn create_backup_captures_writes_still_in_the_wal() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        // Pin writes into the WAL: disable automatic checkpointing so the
        // next INSERT definitely stays out of the main file.
        {
            let conn = db.lock();
            conn.pragma_update(None, "wal_autocheckpoint", 0_i64).unwrap();
        }
        insert_client(&db, "wal-row", "Pending WAL Row");

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let backup_path = mgr.create_backup().unwrap();

        assert_eq!(
            client_count(&backup_path, "wal-row"),
            1,
            "backup must include commits that were still in the WAL",
        );
    }

    // -------- restore ------------------------------------------------------

    #[test]
    fn restore_replaces_live_database_with_source_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        insert_client(&db, "live-only", "Gets Overwritten");

        let source = tmp.path().join("source.sqlite");
        {
            let src_db = seed_real_db(&source);
            insert_client(&src_db, "from-source", "Restored From Source");
        }

        let (mgr, _, _) = build_mgr(tmp.path(), live.clone(), db);
        mgr.restore_database(&source).unwrap();

        assert_eq!(client_count(&live, "from-source"), 1);
        assert_eq!(client_count(&live, "live-only"), 0);
    }

    #[test]
    fn restore_preserves_source_file() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let source = tmp.path().join("source.sqlite");
        {
            let _ = seed_real_db(&source);
        }
        let source_bytes_before = fs::read(&source).unwrap();

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        mgr.restore_database(&source).unwrap();

        assert!(source.exists(), "source file must not be consumed");
        let source_bytes_after = fs::read(&source).unwrap();
        assert_eq!(
            source_bytes_before, source_bytes_after,
            "source file contents must not be modified",
        );
    }

    #[test]
    fn restore_writes_pre_restore_snapshot_into_system_dir_with_canonical_name() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        insert_client(&db, "pre-snapshot-marker", "Preserved By Snapshot");

        let source = tmp.path().join("source.sqlite");
        {
            let _ = seed_real_db(&source);
        }

        let (mgr, _, system_dir) = build_mgr(tmp.path(), live, db);
        let snapshot = mgr.restore_database(&source).unwrap();

        assert!(snapshot.exists(), "pre-restore snapshot must exist");
        assert!(
            snapshot.starts_with(&system_dir),
            "snapshot must land in system dir: {snapshot:?}",
        );
        let name = snapshot.file_name().unwrap().to_str().unwrap();
        let (_, kind) =
            parse_backup_filename(name).expect("canonical filename must parse");
        assert_eq!(kind, BackupKind::PreRestore);
        assert_eq!(
            client_count(&snapshot, "pre-snapshot-marker"),
            1,
            "snapshot must capture live db contents from before the swap",
        );
    }

    #[test]
    fn restore_does_not_touch_live_db_when_snapshot_dir_cannot_be_created() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        insert_client(&db, "live-marker", "Should Survive Failed Restore");
        let source = tmp.path().join("source.sqlite");
        {
            let _ = seed_real_db(&source);
        }

        // Configure the mgr's system dir to a path that is actually a file:
        // create_dir_all will fail with NotADirectory.
        let blocked = tmp.path().join("blocked");
        fs::write(&blocked, b"not a directory").unwrap();
        let user_dir = tmp.path().join("user");
        let settings = Arc::new(crate::adapters::sqlite::SqliteSettingsRepository::new(
            db.clone(),
        ));
        let mgr = FilesystemDataManagement::new(
            db,
            live.clone(),
            settings,
            user_dir,
            blocked,
        );

        let err = mgr.restore_database(&source).unwrap_err();
        assert!(matches!(err, RepoError::Storage(_)));
        assert_eq!(
            client_count(&live, "live-marker"),
            1,
            "live db must be untouched when snapshot fails",
        );
    }

    #[test]
    fn restore_rejects_non_sqlite_file() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let bad = tmp.path().join("not-a-db.txt");
        fs::write(&bad, b"this is not sqlite").unwrap();

        let (mgr, _, _) = build_mgr(tmp.path(), live.clone(), db);
        let err = mgr.restore_database(&bad).unwrap_err();
        match err {
            RepoError::Storage(msg) => assert!(msg.contains("SQLite"), "{msg}"),
            other => panic!("expected Storage error, got {other:?}"),
        }
        assert!(live.exists());
    }

    #[test]
    fn restore_rejects_missing_source() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let missing = tmp.path().join("nowhere.sqlite");
        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&missing).unwrap_err();
        assert!(matches!(err, RepoError::Storage(_)));
    }

    #[test]
    fn restore_rejects_corrupt_sqlite_file() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let corrupt = tmp.path().join("corrupt.sqlite");
        {
            let _ = seed_real_db(&corrupt);
        }
        let mut bytes = fs::read(&corrupt).unwrap();
        for b in bytes.iter_mut().skip(100) {
            *b = 0xFF;
        }
        fs::write(&corrupt, bytes).unwrap();

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&corrupt).unwrap_err();
        match err {
            RepoError::Storage(msg) => {
                let m = msg.to_lowercase();
                assert!(
                    m.contains("integrity") || m.contains("malformed") || m.contains("corrupt"),
                    "unexpected: {msg}",
                );
            }
            other => panic!("expected Storage, got {other:?}"),
        }
    }

    #[test]
    fn restore_rejects_unrelated_sqlite_database() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let foreign = tmp.path().join("foreign.sqlite");
        {
            let conn = rusqlite::Connection::open(&foreign).unwrap();
            conn.execute_batch("CREATE TABLE unrelated (x INTEGER);")
                .unwrap();
        }

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&foreign).unwrap_err();
        match err {
            RepoError::Storage(msg) => assert!(
                msg.to_lowercase().contains("terative"),
                "unexpected: {msg}",
            ),
            other => panic!("expected Storage, got {other:?}"),
        }
    }

    #[test]
    fn restore_rejects_database_with_wrong_application_id() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let foreign = tmp.path().join("foreign.sqlite");
        {
            let conn = rusqlite::Connection::open(&foreign).unwrap();
            conn.pragma_update(None, "application_id", 0x12345678_i32)
                .unwrap();
            conn.execute_batch("CREATE TABLE clients (id TEXT PRIMARY KEY);")
                .unwrap();
        }

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&foreign).unwrap_err();
        match err {
            RepoError::Storage(msg) => assert!(
                msg.to_lowercase().contains("terative"),
                "unexpected: {msg}",
            ),
            other => panic!("expected Storage, got {other:?}"),
        }
    }

    #[test]
    fn restore_rejects_database_from_newer_app_version() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let future = tmp.path().join("future.sqlite");
        {
            let _ = seed_real_db(&future);
        }
        {
            let conn = rusqlite::Connection::open(&future).unwrap();
            conn.pragma_update(None, "user_version", 999_i64).unwrap();
        }

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&future).unwrap_err();
        match err {
            RepoError::Storage(msg) => {
                let m = msg.to_lowercase();
                assert!(
                    m.contains("newer") || m.contains("version"),
                    "unexpected: {msg}",
                );
            }
            other => panic!("expected Storage, got {other:?}"),
        }
    }

    #[test]
    fn restore_rejects_database_with_foreign_key_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let src = tmp.path().join("src.sqlite");
        {
            let _ = seed_real_db(&src);
        }
        {
            let conn = rusqlite::Connection::open(&src).unwrap();
            conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
            conn.execute(
                "INSERT INTO client_emails (id, client_id, value) VALUES (?1, ?2, ?3)",
                rusqlite::params!["orphan-1", "does-not-exist", "a@b.c"],
            )
            .unwrap();
        }

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let err = mgr.restore_database(&src).unwrap_err();
        match err {
            RepoError::Storage(msg) => assert!(
                msg.to_lowercase().contains("foreign key"),
                "unexpected: {msg}",
            ),
            other => panic!("expected Storage, got {other:?}"),
        }
    }

    // -------- auto_backup_if_due ------------------------------------------

    fn set_auto_backup_prefs(db: &Db, enabled: bool, interval_hours: u32) {
        use crate::application::ports::SettingsRepository;
        use crate::domain::settings::AppPreferences;
        let repo = crate::adapters::sqlite::SqliteSettingsRepository::new(db.clone());
        let mut prefs = repo.get_app_preferences().unwrap();
        prefs.auto_backup_enabled = enabled;
        prefs.auto_backup_interval_hours = interval_hours;
        repo.set_app_preferences(&prefs).unwrap();
    }

    /// Helper: back-date a file's mtime so the staleness check treats it as old.
    fn set_mtime_hours_ago(path: &Path, hours: i64) {
        let when = std::time::SystemTime::now() - std::time::Duration::from_secs((hours * 3600) as u64);
        let f = fs::File::options().write(true).open(path).unwrap();
        f.set_modified(when).unwrap();
    }

    #[test]
    fn auto_backup_creates_first_auto_backup_when_none_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        set_auto_backup_prefs(&db, true, 24);

        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        let path = mgr.auto_backup_if_due().unwrap().expect("should backup");

        assert!(path.starts_with(&user_dir));
        let name = path.file_name().unwrap().to_str().unwrap();
        let (_, kind) = parse_backup_filename(name).unwrap();
        assert_eq!(kind, BackupKind::Auto);
    }

    #[test]
    fn auto_backup_is_noop_when_recent_auto_backup_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        set_auto_backup_prefs(&db, true, 24);
        let (mgr, _, _) = build_mgr(tmp.path(), live, db);

        // First call creates a backup.
        let first = mgr.auto_backup_if_due().unwrap().expect("first should backup");
        assert!(first.exists());

        // Second call immediately after should be a no-op (still within interval).
        let second = mgr.auto_backup_if_due().unwrap();
        assert!(second.is_none(), "expected no-op, got {second:?}");
    }

    #[test]
    fn auto_backup_returns_none_when_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        set_auto_backup_prefs(&db, false, 24);

        let (mgr, _, _) = build_mgr(tmp.path(), live, db);
        let out = mgr.auto_backup_if_due().unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn auto_backup_creates_new_backup_when_interval_has_elapsed() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        set_auto_backup_prefs(&db, true, 24);
        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);

        // Seed an auto backup file whose timestamp is 48h ago.
        fs::create_dir_all(&user_dir).unwrap();
        let stale_name = format_backup_filename(
            BackupKind::Auto,
            Utc::now() - chrono::Duration::hours(48),
        );
        let stale = user_dir.join(stale_name);
        fs::write(&stale, b"SQLite format 3\0").unwrap();
        // Back-date mtime too in case the implementation uses it.
        set_mtime_hours_ago(&stale, 48);

        let out = mgr.auto_backup_if_due().unwrap().expect("should backup");
        assert!(out.exists());
        assert_ne!(out, stale, "new backup must not reuse the stale filename");
    }

    #[test]
    fn auto_backup_prunes_older_autos_beyond_retention_count() {
        use crate::application::ports::SettingsRepository;
        use crate::domain::settings::{AppPreferences, RetentionMode};

        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        // KeepLast=2: the newly-created auto backup plus one older survives.
        let repo = crate::adapters::sqlite::SqliteSettingsRepository::new(db.clone());
        repo.set_app_preferences(&AppPreferences {
            auto_backup_enabled: true,
            auto_backup_interval_hours: 24,
            retention_mode: RetentionMode::KeepLast,
            retention_count: 2,
            ..Default::default()
        })
        .unwrap();

        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();

        // Seed three auto backups spanning different days so there's something
        // to prune. They all pre-date the 24h interval so the next call would
        // create a fourth — which should trigger retention down to 2.
        let seed = |days_ago: i64| {
            let path = user_dir.join(format_backup_filename(
                BackupKind::Auto,
                Utc::now() - chrono::Duration::days(days_ago),
            ));
            fs::write(&path, b"SQLite format 3\0").unwrap();
            path
        };
        let oldest = seed(10);
        let middle = seed(5);
        let _second_newest = seed(2);

        let new_backup = mgr.auto_backup_if_due().unwrap().expect("should backup");

        // After pruning: the just-created backup plus the most recent of the
        // three seeds (_second_newest) should remain. The two oldest gone.
        assert!(new_backup.exists());
        assert!(
            !oldest.exists(),
            "oldest auto backup should have been pruned",
        );
        assert!(
            !middle.exists(),
            "middle auto backup should have been pruned",
        );
        let remaining: Vec<_> = scan_backup_dir(&user_dir)
            .into_iter()
            .filter(|b| b.kind == BackupKind::Auto)
            .collect();
        assert_eq!(remaining.len(), 2, "should keep exactly 2 auto backups");
    }

    #[test]
    fn auto_backup_retention_never_prunes_manual_or_system_backups() {
        use crate::application::ports::SettingsRepository;
        use crate::domain::settings::{AppPreferences, RetentionMode};

        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let repo = crate::adapters::sqlite::SqliteSettingsRepository::new(db.clone());
        repo.set_app_preferences(&AppPreferences {
            auto_backup_enabled: true,
            retention_mode: RetentionMode::KeepLast,
            retention_count: 1,
            ..Default::default()
        })
        .unwrap();

        let (mgr, user_dir, system_dir) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&system_dir).unwrap();

        // Manual backups pre-existing in the user dir.
        let manual_old = user_dir.join(format_backup_filename(
            BackupKind::Manual,
            Utc::now() - chrono::Duration::days(30),
        ));
        let manual_recent = user_dir.join(format_backup_filename(
            BackupKind::Manual,
            Utc::now() - chrono::Duration::hours(1),
        ));
        fs::write(&manual_old, b"SQLite format 3\0").unwrap();
        fs::write(&manual_recent, b"SQLite format 3\0").unwrap();

        // A system snapshot that should also survive retention.
        let system_snapshot = system_dir.join(format_backup_filename(
            BackupKind::PreRestore,
            Utc::now() - chrono::Duration::days(30),
        ));
        fs::write(&system_snapshot, b"SQLite format 3\0").unwrap();

        mgr.auto_backup_if_due().unwrap().expect("should backup");

        assert!(manual_old.exists(), "manual backups must not be pruned");
        assert!(manual_recent.exists(), "manual backups must not be pruned");
        assert!(
            system_snapshot.exists(),
            "system (prerestore) backups must not be pruned",
        );
    }

    #[test]
    fn auto_backup_retention_all_keeps_every_auto_backup() {
        use crate::application::ports::SettingsRepository;
        use crate::domain::settings::{AppPreferences, RetentionMode};

        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        let repo = crate::adapters::sqlite::SqliteSettingsRepository::new(db.clone());
        repo.set_app_preferences(&AppPreferences {
            auto_backup_enabled: true,
            retention_mode: RetentionMode::All,
            retention_count: 1, // irrelevant when mode == All
            ..Default::default()
        })
        .unwrap();

        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        for days_ago in [30, 10, 5, 2] {
            let path = user_dir.join(format_backup_filename(
                BackupKind::Auto,
                Utc::now() - chrono::Duration::days(days_ago),
            ));
            fs::write(&path, b"SQLite format 3\0").unwrap();
        }

        mgr.auto_backup_if_due().unwrap().expect("should backup");

        let auto_count = scan_backup_dir(&user_dir)
            .into_iter()
            .filter(|b| b.kind == BackupKind::Auto)
            .count();
        // 4 pre-existing + 1 new = 5 retained.
        assert_eq!(auto_count, 5);
    }

    #[test]
    fn auto_backup_ignores_manual_backups_when_deciding_staleness() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        set_auto_backup_prefs(&db, true, 24);
        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);

        // A very recent MANUAL backup — should not block an auto backup.
        fs::create_dir_all(&user_dir).unwrap();
        let manual = user_dir.join(format_backup_filename(
            BackupKind::Manual,
            Utc::now(),
        ));
        fs::write(&manual, b"SQLite format 3\0").unwrap();

        let out = mgr.auto_backup_if_due().unwrap();
        assert!(
            out.is_some(),
            "manual backups must not prevent the first auto backup",
        );
    }

    // -------- system-backup retention -------------------------------------

    #[test]
    fn restore_prunes_old_prerestore_snapshots_to_keep_last_five() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);

        // Seed a valid restore source that passes validation.
        let source = tmp.path().join("source.sqlite");
        {
            let _ = seed_real_db(&source);
        }

        let (mgr, _, system_dir) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&system_dir).unwrap();

        // Pre-seed 6 prerestore snapshots with different timestamps. After a
        // real restore writes a 7th, pruning should leave exactly 5.
        let days_ago = [30_i64, 25, 20, 15, 10, 5];
        for d in days_ago {
            let p = system_dir.join(format_backup_filename(
                BackupKind::PreRestore,
                Utc::now() - chrono::Duration::days(d),
            ));
            fs::write(&p, b"SQLite format 3\0").unwrap();
        }

        mgr.restore_database(&source).unwrap();

        let prerestores = scan_backup_dir(&system_dir)
            .into_iter()
            .filter(|b| b.kind == BackupKind::PreRestore)
            .count();
        assert_eq!(prerestores, 5);
    }

    // -------- delete_backup ------------------------------------------------

    #[test]
    fn delete_backup_removes_manual_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        let target = user_dir.join(format_backup_filename(
            BackupKind::Manual,
            Utc::now(),
        ));
        fs::write(&target, b"SQLite format 3\0").unwrap();

        mgr.delete_backup(&target).unwrap();
        assert!(!target.exists());
    }

    #[test]
    fn delete_backup_removes_auto_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        let target = user_dir.join(format_backup_filename(BackupKind::Auto, Utc::now()));
        fs::write(&target, b"SQLite format 3\0").unwrap();

        mgr.delete_backup(&target).unwrap();
        assert!(!target.exists());
    }

    #[test]
    fn delete_backup_refuses_system_snapshots() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, _, system_dir) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&system_dir).unwrap();
        for kind in [BackupKind::PreRestore, BackupKind::PreMigration] {
            let target = system_dir.join(format_backup_filename(kind, Utc::now()));
            fs::write(&target, b"SQLite format 3\0").unwrap();

            let err = mgr.delete_backup(&target).unwrap_err();
            assert!(
                matches!(err, RepoError::Storage(_)),
                "expected refusal for {kind:?}",
            );
            assert!(target.exists(), "system snapshot must remain on disk");
        }
    }

    #[test]
    fn delete_backup_refuses_paths_outside_configured_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, _, _) = build_mgr(tmp.path(), live, db);

        // A file that matches the backup filename grammar but lives in an
        // arbitrary location — deletion must be refused to prevent the IPC
        // from being misused as a generic file-delete endpoint.
        let sneaky = tmp.path().join(format_backup_filename(
            BackupKind::Manual,
            Utc::now(),
        ));
        fs::write(&sneaky, b"SQLite format 3\0").unwrap();

        let err = mgr.delete_backup(&sneaky).unwrap_err();
        assert!(matches!(err, RepoError::Storage(_)));
        assert!(sneaky.exists());
    }

    #[test]
    fn delete_backup_refuses_malformed_filenames() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, user_dir, _) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        let bogus = user_dir.join("definitely-not-a-backup.txt");
        fs::write(&bogus, b"hi").unwrap();

        let err = mgr.delete_backup(&bogus).unwrap_err();
        assert!(matches!(err, RepoError::Storage(_)));
        assert!(bogus.exists());
    }

    // -------- list_backups -------------------------------------------------

    #[test]
    fn list_backups_returns_entries_from_both_dirs_sorted_newest_first() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, user_dir, system_dir) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&system_dir).unwrap();

        // Seed one backup of each kind with distinct, out-of-order timestamps.
        let seed = |dir: &Path, kind: BackupKind, ts: DateTime<Utc>| {
            let path = dir.join(format_backup_filename(kind, ts));
            fs::write(&path, b"SQLite format 3\0").unwrap(); // contents irrelevant for listing
            path
        };
        let oldest = seed(
            &user_dir,
            BackupKind::Manual,
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        );
        let middle = seed(
            &system_dir,
            BackupKind::PreRestore,
            Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
        );
        let newest = seed(
            &user_dir,
            BackupKind::Auto,
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap(),
        );

        let list = mgr.list_backups().unwrap();
        let paths: Vec<_> = list.iter().map(|b| b.path.clone()).collect();
        assert_eq!(paths, vec![newest, middle, oldest]);
    }

    #[test]
    fn list_backups_ignores_unrelated_files() {
        let tmp = tempfile::tempdir().unwrap();
        let live = tmp.path().join("live.sqlite");
        let db = seed_real_db(&live);
        let (mgr, user_dir, system_dir) = build_mgr(tmp.path(), live, db);
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&system_dir).unwrap();

        // Noise files the scanner must skip.
        fs::write(user_dir.join("notes.txt"), b"hello").unwrap();
        fs::write(user_dir.join("terative-bogus.sqlite"), b"x").unwrap();
        fs::write(system_dir.join(".DS_Store"), b"").unwrap();

        // One real backup to prove the happy path still works.
        let good = user_dir.join(format_backup_filename(
            BackupKind::Manual,
            Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
        ));
        fs::write(&good, b"SQLite format 3\0").unwrap();

        let list = mgr.list_backups().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, good);
        assert_eq!(list[0].kind, BackupKind::Manual);
    }
}
