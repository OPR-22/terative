//! Audit-log handlers for `CatalogItem` domain events. `CatalogItemUpdated`
//! resolves its `entity_label` via [`CatalogItemRepository::labels_for`];
//! `CatalogItemCreated` reuses the snapshot already in the event payload.

use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::ports::{AuditRepository, CatalogItemRepository, EventHandler};
use crate::domain::audit::NewAudit;
use crate::domain::events::catalog_item_events::{CatalogItemCreated, CatalogItemUpdated};
use crate::domain::events::DomainEvent;

pub struct CatalogItemCreatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl CatalogItemCreatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<CatalogItemCreated> for CatalogItemCreatedAuditHandler {
    fn handle(&self, event: &CatalogItemCreated) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "catalog_item".into(),
                entity_id: Some(event.id.to_string()),
                client_id: None,
                invoice_id: None,
                metadata_json: json!({ "entity_label": event.name }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct CatalogItemUpdatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    catalog_items: Arc<dyn CatalogItemRepository>,
}
impl CatalogItemUpdatedAuditHandler {
    pub fn new(
        repo: Arc<dyn AuditRepository>,
        catalog_items: Arc<dyn CatalogItemRepository>,
    ) -> Self {
        Self { repo, catalog_items }
    }
}
impl EventHandler<CatalogItemUpdated> for CatalogItemUpdatedAuditHandler {
    fn handle(&self, event: &CatalogItemUpdated) {
        let entity_label = self
            .catalog_items
            .labels_for(&[event.id])
            .unwrap_or_default()
            .remove(&event.id);
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "catalog_item".into(),
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
    use crate::domain::catalog_item::{CatalogItem, CatalogItemId};
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
    struct StubCatalogItemRepo {
        labels: Mutex<HashMap<CatalogItemId, String>>,
    }
    impl StubCatalogItemRepo {
        fn with(id: CatalogItemId, name: &str) -> Self {
            let mut m = HashMap::new();
            m.insert(id, name.to_string());
            Self { labels: Mutex::new(m) }
        }
    }
    impl CatalogItemRepository for StubCatalogItemRepo {
        fn insert(&self, _: &CatalogItem) -> Result<(), RepoError> { unimplemented!() }
        fn update(&self, _: &CatalogItem) -> Result<(), RepoError> { unimplemented!() }
        fn get(&self, _: CatalogItemId) -> Result<Option<CatalogItem>, RepoError> {
            unimplemented!()
        }
        fn list(&self, _: bool) -> Result<Vec<CatalogItem>, RepoError> { unimplemented!() }
        fn delete(&self, _: CatalogItemId) -> Result<(), RepoError> { unimplemented!() }
        fn labels_for(
            &self,
            ids: &[CatalogItemId],
        ) -> Result<HashMap<CatalogItemId, String>, RepoError> {
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
        let handler = CatalogItemCreatedAuditHandler::new(repo.clone());
        let id = CatalogItemId::new();
        handler.handle(&CatalogItemCreated {
            id,
            name: "Consulting".into(),
            at: at(),
        });
        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Consulting");
    }

    #[test]
    fn updated_handler_resolves_entity_label_via_repo() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let id = CatalogItemId::new();
        let items = Arc::new(StubCatalogItemRepo::with(id, "Premium consulting"));
        let handler = CatalogItemUpdatedAuditHandler::new(repo.clone(), items);
        handler.handle(&CatalogItemUpdated {
            id,
            changes: vec![],
            at: at(),
        });
        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Premium consulting");
    }
}
