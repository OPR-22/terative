use std::path::{Path, PathBuf};

use crate::application::RepoError;

/// Manages the on-disk database file: export (user-picked destination),
/// timestamped backup, and restore from a snapshot.
pub trait DataManagement: Send + Sync {
    /// Copies the current database to `destination`. Returns the absolute path
    /// that was written.
    fn export_database(&self, destination: &Path) -> Result<PathBuf, RepoError>;

    /// Writes a timestamped copy into `backup_dir`. Creates the directory if it
    /// does not exist. Returns the path of the new backup file.
    fn create_backup(&self, backup_dir: &Path) -> Result<PathBuf, RepoError>;

    /// Replaces the live database file with the contents of `source`. The
    /// caller is responsible for restarting the app so the new file is opened.
    fn restore_database(&self, source: &Path) -> Result<(), RepoError>;
}
