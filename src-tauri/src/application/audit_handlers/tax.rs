//! Audit-log handlers for `TaxDefinition` domain events. `TaxUpdated`
//! resolves its `entity_label` via [`TaxRepository::labels_for`]; `TaxCreated`
//! reuses the snapshot already in the event payload.

use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::ports::{AuditRepository, EventHandler, TaxRepository};
use crate::domain::audit::NewAudit;
use crate::domain::events::tax_events::{TaxCreated, TaxUpdated};
use crate::domain::events::DomainEvent;

pub struct TaxCreatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl TaxCreatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<TaxCreated> for TaxCreatedAuditHandler {
    fn handle(&self, event: &TaxCreated) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "tax".into(),
                entity_id: Some(event.id.to_string()),
                client_id: None,
                invoice_id: None,
                metadata_json: json!({ "entity_label": event.name }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct TaxUpdatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    taxes: Arc<dyn TaxRepository>,
}
impl TaxUpdatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, taxes: Arc<dyn TaxRepository>) -> Self {
        Self { repo, taxes }
    }
}
impl EventHandler<TaxUpdated> for TaxUpdatedAuditHandler {
    fn handle(&self, event: &TaxUpdated) {
        let entity_label = self
            .taxes
            .labels_for(&[event.id])
            .unwrap_or_default()
            .remove(&event.id);
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "tax".into(),
                entity_id: Some(event.id.to_string()),
                client_id: None,
                invoice_id: None,
                metadata_json: json!({
                    "changes": event.changes,
                    "entity_label": entity_label,
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
    use crate::application::RepoError;
    use crate::domain::tax::{TaxDefinition, TaxId};
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;
    use std::collections::HashMap;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    /// Minimal stub — only `labels_for` is exercised here.
    #[derive(Default)]
    struct StubTaxRepo {
        labels: Mutex<HashMap<TaxId, String>>,
    }
    impl StubTaxRepo {
        fn with(id: TaxId, name: &str) -> Self {
            let mut m = HashMap::new();
            m.insert(id, name.to_string());
            Self { labels: Mutex::new(m) }
        }
    }
    impl TaxRepository for StubTaxRepo {
        fn insert(&self, _: &TaxDefinition) -> Result<(), RepoError> { unimplemented!() }
        fn update(&self, _: &TaxDefinition) -> Result<(), RepoError> { unimplemented!() }
        fn get(&self, _: TaxId) -> Result<Option<TaxDefinition>, RepoError> { unimplemented!() }
        fn list(&self, _: bool) -> Result<Vec<TaxDefinition>, RepoError> { unimplemented!() }
        fn get_many(&self, _: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError> {
            unimplemented!()
        }
        fn delete(&self, _: TaxId) -> Result<(), RepoError> { unimplemented!() }
        fn labels_for(
            &self,
            ids: &[TaxId],
        ) -> Result<HashMap<TaxId, String>, RepoError> {
            let g = self.labels.lock();
            Ok(ids
                .iter()
                .filter_map(|id| g.get(id).map(|n| (*id, n.clone())))
                .collect())
        }
    }

    #[test]
    fn created_handler_includes_entity_label() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let handler = TaxCreatedAuditHandler::new(repo.clone());
        let id = TaxId::new();
        handler.handle(&TaxCreated {
            id,
            name: "TVA".into(),
            at: at(),
        });
        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "TVA");
    }

    #[test]
    fn updated_handler_resolves_entity_label_via_repo() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let id = TaxId::new();
        let taxes = Arc::new(StubTaxRepo::with(id, "VAT 20%"));
        let handler = TaxUpdatedAuditHandler::new(repo.clone(), taxes);
        handler.handle(&TaxUpdated {
            id,
            changes: vec![],
            at: at(),
        });
        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "VAT 20%");
    }
}
