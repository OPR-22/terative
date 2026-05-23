//! Audit-log handler for the `BackupCreated` application event.

use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::events::BackupCreated;
use crate::application::ports::{AuditRepository, EventHandler};
use crate::domain::audit::NewAudit;
use crate::domain::events::DomainEvent;

pub struct BackupCreatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl BackupCreatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<BackupCreated> for BackupCreatedAuditHandler {
    fn handle(&self, event: &BackupCreated) {
        // A backup has no entity id and no client/invoice scope — it is an
        // org-wide event, so it only ever surfaces on the dashboard feed.
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "backup".into(),
                entity_id: None,
                client_id: None,
                invoice_id: None,
                metadata_json: json!({
                    "path": event.path,
                    "kind": event.kind.as_str(),
                    // No entity scope for a backup; use the kind ("manual" /
                    // "auto") as a uniform `entity_label` so the FE renderer
                    // can treat backup rows the same as everything else.
                    "entity_label": event.kind.as_str(),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::audit_handlers::test_support::InMemoryAuditRepo;
    use crate::application::ports::BackupKind;
    use chrono::{DateTime, Utc};

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn backup_handler_projects_unscoped_row() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let handler = BackupCreatedAuditHandler::new(repo.clone());

        handler.handle(&BackupCreated {
            kind: BackupKind::Manual,
            path: "/backups/terative-20260515.sqlite".into(),
            at: at(),
        });

        let rows = repo.rows.lock();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_type, "backup.created");
        assert_eq!(rows[0].entity_type, "backup");
        assert_eq!(rows[0].entity_id, None);
        assert_eq!(rows[0].client_id, None);
        assert_eq!(rows[0].invoice_id, None);
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["kind"], "manual");
        assert_eq!(meta["path"], "/backups/terative-20260515.sqlite");
    }
}
