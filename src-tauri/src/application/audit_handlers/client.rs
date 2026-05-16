//! Audit-log handlers for `Client` domain events.
//!
//! Every row written by these handlers carries an `entity_label` in
//! `metadata_json` — for Created/Archived/Unarchived it's taken straight
//! from the event payload's `name` snapshot; for Updated (which has no name
//! field) the handler resolves it via [`ClientRepository::labels_for`].

use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::ports::{AuditRepository, ClientRepository, EventHandler};
use crate::domain::audit::NewAudit;
use crate::domain::events::client_events::{
    ClientArchived, ClientCreated, ClientUnarchived, ClientUpdated,
};
use crate::domain::events::DomainEvent;

pub struct ClientCreatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl ClientCreatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<ClientCreated> for ClientCreatedAuditHandler {
    fn handle(&self, event: &ClientCreated) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "client".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.id),
                invoice_id: None,
                metadata_json: json!({ "entity_label": event.name }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct ClientUpdatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    clients: Arc<dyn ClientRepository>,
}
impl ClientUpdatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, clients: Arc<dyn ClientRepository>) -> Self {
        Self { repo, clients }
    }
}
impl EventHandler<ClientUpdated> for ClientUpdatedAuditHandler {
    fn handle(&self, event: &ClientUpdated) {
        let entity_label = self
            .clients
            .labels_for(&[event.id])
            .unwrap_or_default()
            .remove(&event.id);
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "client".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.id),
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

pub struct ClientArchivedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl ClientArchivedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<ClientArchived> for ClientArchivedAuditHandler {
    fn handle(&self, event: &ClientArchived) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "client".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.id),
                invoice_id: None,
                metadata_json: json!({ "entity_label": event.name }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct ClientUnarchivedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl ClientUnarchivedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<ClientUnarchived> for ClientUnarchivedAuditHandler {
    fn handle(&self, event: &ClientUnarchived) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "client".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.id),
                invoice_id: None,
                metadata_json: json!({ "entity_label": event.name }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::audit_handlers::test_support::InMemoryAuditRepo;
    use crate::application::ports::{ClientAttributeValues, ListClientsQuery, Page};
    use crate::application::RepoError;
    use crate::domain::client::{Client, ClientId};
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;
    use std::collections::HashMap;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    /// Minimal stub — only `labels_for` is exercised by these tests.
    #[derive(Default)]
    struct StubClientRepo {
        labels: Mutex<HashMap<ClientId, String>>,
    }
    impl StubClientRepo {
        fn with(id: ClientId, name: &str) -> Self {
            let mut m = HashMap::new();
            m.insert(id, name.to_string());
            Self { labels: Mutex::new(m) }
        }
    }
    impl ClientRepository for StubClientRepo {
        fn insert(&self, _: &Client) -> Result<(), RepoError> { unimplemented!() }
        fn update(&self, _: &Client) -> Result<(), RepoError> { unimplemented!() }
        fn get(&self, _: ClientId) -> Result<Option<Client>, RepoError> { unimplemented!() }
        fn list(&self, _: ListClientsQuery) -> Result<Page<Client>, RepoError> {
            unimplemented!()
        }
        fn distinct_attribute_values(&self) -> Result<ClientAttributeValues, RepoError> {
            unimplemented!()
        }
        fn labels_for(
            &self,
            ids: &[ClientId],
        ) -> Result<HashMap<ClientId, String>, RepoError> {
            let g = self.labels.lock();
            Ok(ids
                .iter()
                .filter_map(|id| g.get(id).map(|n| (*id, n.clone())))
                .collect())
        }
    }

    #[test]
    fn created_handler_projects_client_scoped_row_with_entity_label() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let handler = ClientCreatedAuditHandler::new(repo.clone());
        let id = ClientId::new();

        handler.handle(&ClientCreated {
            id,
            name: "Acme".into(),
            at: at(),
        });

        let rows = repo.rows.lock();
        assert_eq!(rows.len(), 1);
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Acme");
        // `name` is no longer duplicated — `entity_label` is the only label.
        assert!(meta.get("name").is_none());
    }

    #[test]
    fn updated_handler_resolves_entity_label_via_repo() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let id = ClientId::new();
        let clients = Arc::new(StubClientRepo::with(id, "Acme Corp"));
        let handler = ClientUpdatedAuditHandler::new(repo.clone(), clients);

        handler.handle(&ClientUpdated {
            id,
            changes: vec![],
            at: at(),
        });

        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Acme Corp");
    }
}
