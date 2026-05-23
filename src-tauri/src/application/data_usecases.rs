//! Application use cases for database file management.
//!
//! The `FilesystemDataManagement` adapter does the actual file work; this use
//! case wraps `create_backup` so a manual backup can publish a `BackupCreated`
//! audit event without coupling the adapter to the event bus.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;

use crate::application::events::BackupCreated;
use crate::application::ports::{BackupKind, DataManagement, EventBus, NoopEventBus};
use crate::application::AppError;

pub struct CreateBackup {
    data: Arc<dyn DataManagement>,
    events: Arc<dyn EventBus>,
}

impl CreateBackup {
    pub fn new(data: Arc<dyn DataManagement>) -> Self {
        Self {
            data,
            events: Arc::new(NoopEventBus),
        }
    }

    /// Inject the real event bus. Production wiring (`OrgServices::new`) calls
    /// this; tests that don't assert on events keep the no-op default.
    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }

    /// Writes a manual backup and records a `BackupCreated` audit event.
    pub fn execute(&self) -> Result<PathBuf, AppError> {
        let path = self.data.create_backup()?;
        self.events.dispatch(&BackupCreated {
            kind: BackupKind::Manual,
            path: path.to_string_lossy().to_string(),
            at: Utc::now(),
        });
        Ok(path)
    }
}

/// The scheduled-backup counterpart of [`CreateBackup`]: runs the staleness
/// check and, *if* it actually wrote a backup, records a `BackupCreated`
/// audit event with `BackupKind::Auto`. Driven by the auto-backup ticker.
pub struct AutoBackupIfDue {
    data: Arc<dyn DataManagement>,
    events: Arc<dyn EventBus>,
}

impl AutoBackupIfDue {
    pub fn new(data: Arc<dyn DataManagement>) -> Self {
        Self {
            data,
            events: Arc::new(NoopEventBus),
        }
    }

    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }

    /// Returns the new backup path when one was written, or `None` when the
    /// staleness check decided nothing was due. Only the `Some` case emits an
    /// event — a no-op tick is not audit.
    pub fn execute(&self) -> Result<Option<PathBuf>, AppError> {
        let made = self.data.auto_backup_if_due()?;
        if let Some(path) = &made {
            self.events.dispatch(&BackupCreated {
                kind: BackupKind::Auto,
                path: path.to_string_lossy().to_string(),
                at: Utc::now(),
            });
        }
        Ok(made)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::event_bus::test_support::CollectingEventBus;
    use crate::application::ports::BackupMetadata;
    use crate::application::RepoError;
    use std::path::Path;

    /// Minimal `DataManagement` stub. `create_backup` returns a fixed path;
    /// `auto_backup_if_due` returns whatever was configured; every other
    /// method is unused by these tests.
    struct StubDataManagement {
        backup_path: PathBuf,
        auto_due: Option<PathBuf>,
    }

    impl StubDataManagement {
        fn new(backup_path: &str) -> Self {
            Self {
                backup_path: PathBuf::from(backup_path),
                auto_due: None,
            }
        }
        fn with_auto_due(mut self, path: Option<&str>) -> Self {
            self.auto_due = path.map(PathBuf::from);
            self
        }
    }

    impl DataManagement for StubDataManagement {
        fn export_database(&self, _destination: &Path) -> Result<PathBuf, RepoError> {
            unimplemented!()
        }
        fn create_backup(&self) -> Result<PathBuf, RepoError> {
            Ok(self.backup_path.clone())
        }
        fn restore_database(
            &self,
            _source: &Path,
            _source_password: Option<&str>,
        ) -> Result<PathBuf, RepoError> {
            unimplemented!()
        }
        fn source_appears_encrypted(&self, _source: &Path) -> Result<bool, RepoError> {
            unimplemented!()
        }
        fn list_backups(&self) -> Result<Vec<BackupMetadata>, RepoError> {
            unimplemented!()
        }
        fn auto_backup_if_due(&self) -> Result<Option<PathBuf>, RepoError> {
            Ok(self.auto_due.clone())
        }
        fn delete_backup(&self, _path: &Path) -> Result<(), RepoError> {
            unimplemented!()
        }
    }

    #[test]
    fn create_backup_returns_path_and_publishes_backup_created() {
        let data = Arc::new(StubDataManagement::new("/backups/terative-20260515.sqlite"));
        let bus = Arc::new(CollectingEventBus::default());

        let path = CreateBackup::new(data)
            .with_events(bus.clone())
            .execute()
            .unwrap();

        assert_eq!(path, PathBuf::from("/backups/terative-20260515.sqlite"));
        assert_eq!(bus.names(), ["backup.created"]);
    }

    #[test]
    fn auto_backup_publishes_event_when_a_backup_was_written() {
        let data = Arc::new(
            StubDataManagement::new("/unused")
                .with_auto_due(Some("/backups/terative-auto.sqlite")),
        );
        let bus = Arc::new(CollectingEventBus::default());

        let made = AutoBackupIfDue::new(data)
            .with_events(bus.clone())
            .execute()
            .unwrap();

        assert_eq!(made, Some(PathBuf::from("/backups/terative-auto.sqlite")));
        assert_eq!(bus.names(), ["backup.created"]);
    }

    #[test]
    fn auto_backup_publishes_nothing_when_not_due() {
        let data = Arc::new(StubDataManagement::new("/unused").with_auto_due(None));
        let bus = Arc::new(CollectingEventBus::default());

        let made = AutoBackupIfDue::new(data)
            .with_events(bus.clone())
            .execute()
            .unwrap();

        assert_eq!(made, None);
        assert!(bus.names().is_empty(), "a no-op tick is not audit");
    }
}
