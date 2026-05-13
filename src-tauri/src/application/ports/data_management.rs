use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::application::RepoError;

/// Discriminates the provenance of a backup file. Encoded into the filename
/// (see [`crate::adapters::filesystem_data_management`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupKind {
    /// User clicked "Back up now".
    Manual,
    /// Scheduled/auto backup created by the staleness check.
    Auto,
    /// Snapshot taken immediately before a restore overwrites the live db.
    PreRestore,
    /// Snapshot taken immediately before a schema migration runs.
    PreMigration,
}

/// Ownership of a backup. User-scope backups are created at the user's
/// request (or automated on their behalf) and can be deleted from the UI.
/// System-scope backups are safety rails written by the app itself before
/// destructive operations and are not user-deletable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupScope {
    User,
    System,
}

impl BackupKind {
    pub fn scope(&self) -> BackupScope {
        match self {
            BackupKind::Manual | BackupKind::Auto => BackupScope::User,
            BackupKind::PreRestore | BackupKind::PreMigration => BackupScope::System,
        }
    }
}

/// One backup file on disk, with metadata parsed from its filename.
#[derive(Debug, Clone)]
pub struct BackupMetadata {
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub kind: BackupKind,
    pub size_bytes: u64,
}

/// Manages the on-disk database file: export (user-picked destination),
/// timestamped backup, restore from a snapshot, and discovery of existing
/// backups.
pub trait DataManagement: Send + Sync {
    /// Copies the current database to `destination`. Returns the absolute path
    /// that was written.
    fn export_database(&self, destination: &Path) -> Result<PathBuf, RepoError>;

    /// Writes a manual backup into the configured user backup directory.
    /// Returns the path of the new backup file.
    fn create_backup(&self) -> Result<PathBuf, RepoError>;

    /// Replaces the live database file with the contents of `source`. Before
    /// the swap, snapshots the live database into the configured system
    /// backup directory so the operation is reversible. Returns the snapshot
    /// path. The caller is responsible for restarting the app so the new
    /// file is opened.
    ///
    /// `source_password` is required iff the source file is SQLCipher-
    /// encrypted; for plaintext sources it is ignored. The post-restore
    /// live db will be encrypted under this password — the caller must
    /// clear / refresh any cached unlock key (see
    /// [`DataManagement::source_appears_encrypted`]).
    fn restore_database(
        &self,
        source: &Path,
        source_password: Option<&str>,
    ) -> Result<PathBuf, RepoError>;

    /// Cheap heuristic: looks at the file's leading bytes. True when the
    /// file does not begin with `SQLite format 3\0`, meaning it's either
    /// SQLCipher-encrypted, junk, or another format. Used by the restore
    /// UI to decide whether to prompt for a password before calling
    /// `restore_database`.
    fn source_appears_encrypted(&self, source: &Path) -> Result<bool, RepoError>;

    /// Scans the configured backup directories and returns all Terative
    /// backup files found, sorted newest-first.
    fn list_backups(&self) -> Result<Vec<BackupMetadata>, RepoError>;

    /// Creates an auto backup if enabled in preferences and the most recent
    /// auto backup is older than the configured interval. Returns the new
    /// backup path if one was created, or None if the check decided nothing
    /// needs to happen yet. Meant to be called from app startup and a
    /// periodic ticker.
    fn auto_backup_if_due(&self) -> Result<Option<PathBuf>, RepoError>;

    /// Deletes a manual or auto backup from disk. Refuses system snapshots
    /// (pre-restore, pre-migration) so the safety rails stay intact, and
    /// refuses any path that is not inside one of the configured backup
    /// directories so this cannot be abused as a generic file-delete
    /// endpoint.
    fn delete_backup(&self, path: &Path) -> Result<(), RepoError>;
}
