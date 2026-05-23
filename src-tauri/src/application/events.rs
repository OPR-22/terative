//! Application-level domain events.
//!
//! Most domain events are emitted by aggregate roots and live in
//! `domain::events`. `BackupCreated` is different: there is no backup
//! aggregate — it is emitted by the `CreateBackup` use case (and the
//! auto-backup ticker). It still implements [`DomainEvent`] so it rides the
//! same bus and projects into the audit log like any other event.

use chrono::{DateTime, Utc};

use crate::application::ports::BackupKind;
use crate::domain::events::DomainEvent;

#[derive(Debug, Clone)]
pub struct BackupCreated {
    pub kind: BackupKind,
    pub path: String,
    pub at: DateTime<Utc>,
}

impl DomainEvent for BackupCreated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "backup.created"
    }
}
