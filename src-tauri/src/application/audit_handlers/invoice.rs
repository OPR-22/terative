//! Audit-log handlers for `Invoice` domain events. One handler struct per
//! event type — each projects its event into a single `Audit` row and writes
//! a uniform `entity_label` into the metadata.
//!
//! For events whose payload already carries an invoice number (Finalized,
//! Cancelled, Sent), the label is composed inline as `"#1001"`. For draft
//! events (DraftCreated, DraftUpdated, Duplicated) the invoice has no number
//! yet, so the handler resolves the label via
//! [`InvoiceRepository::labels_for`] — which returns the *client* name for
//! drafts ("Acme") as a sensible user-facing identifier.

use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::ports::{AuditRepository, EventHandler, InvoiceRepository};
use crate::domain::audit::NewAudit;
use crate::domain::events::invoice_events::{
    InvoiceCancelled, InvoiceDraftCreated, InvoiceDraftUpdated, InvoiceDuplicated,
    InvoiceFinalized, InvoiceSent,
};
use crate::domain::events::DomainEvent;
use crate::domain::invoice::{InvoiceId, InvoiceNumber};

/// Compose the `"#1001"` label from an `InvoiceNumber`. Used by handlers
/// whose event already carries the number — no repo round-trip needed.
fn label_from_number(n: InvoiceNumber) -> String {
    format!("#{}", n.0)
}

fn label_from_optional(n: Option<InvoiceNumber>) -> Option<String> {
    n.map(label_from_number)
}

pub struct InvoiceDraftCreatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    invoices: Arc<dyn InvoiceRepository>,
}
impl InvoiceDraftCreatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, invoices: Arc<dyn InvoiceRepository>) -> Self {
        Self { repo, invoices }
    }
}
impl EventHandler<InvoiceDraftCreated> for InvoiceDraftCreatedAuditHandler {
    fn handle(&self, event: &InvoiceDraftCreated) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.id),
                metadata_json: json!({
                    "total": event.total.format(),
                    "entity_label": resolve_invoice_label(self.invoices.as_ref(), event.id),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct InvoiceDraftUpdatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    invoices: Arc<dyn InvoiceRepository>,
}
impl InvoiceDraftUpdatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, invoices: Arc<dyn InvoiceRepository>) -> Self {
        Self { repo, invoices }
    }
}
impl EventHandler<InvoiceDraftUpdated> for InvoiceDraftUpdatedAuditHandler {
    fn handle(&self, event: &InvoiceDraftUpdated) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.id),
                metadata_json: json!({
                    "changes": event.changes,
                    "entity_label": resolve_invoice_label(self.invoices.as_ref(), event.id),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct InvoiceFinalizedAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl InvoiceFinalizedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<InvoiceFinalized> for InvoiceFinalizedAuditHandler {
    fn handle(&self, event: &InvoiceFinalized) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.id),
                metadata_json: json!({
                    "total": event.total.format(),
                    "entity_label": label_from_number(event.number),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct InvoiceCancelledAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl InvoiceCancelledAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<InvoiceCancelled> for InvoiceCancelledAuditHandler {
    fn handle(&self, event: &InvoiceCancelled) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.id),
                metadata_json: json!({
                    "entity_label": label_from_optional(event.number),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct InvoiceDuplicatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    invoices: Arc<dyn InvoiceRepository>,
}
impl InvoiceDuplicatedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, invoices: Arc<dyn InvoiceRepository>) -> Self {
        Self { repo, invoices }
    }
}
impl EventHandler<InvoiceDuplicated> for InvoiceDuplicatedAuditHandler {
    fn handle(&self, event: &InvoiceDuplicated) {
        // The audit is *about* the new draft, so it scopes to `new_id`;
        // `source_id` rides along in the metadata for "duplicated from …".
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.new_id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.new_id),
                metadata_json: json!({
                    "source_id": event.source_id.to_string(),
                    "entity_label": resolve_invoice_label(self.invoices.as_ref(), event.new_id),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

pub struct InvoiceSentAuditHandler {
    repo: Arc<dyn AuditRepository>,
}
impl InvoiceSentAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}
impl EventHandler<InvoiceSent> for InvoiceSentAuditHandler {
    fn handle(&self, event: &InvoiceSent) {
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "invoice".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: Some(event.id),
                metadata_json: json!({
                    "to_address": event.to_address,
                    "entity_label": label_from_optional(event.number),
                })
                .to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

/// Look up the invoice's display label (`"#1001"` when finalized, client
/// name when still a draft). Failure is non-fatal — the row falls back to
/// no label and still renders by its other fields.
fn resolve_invoice_label(repo: &dyn InvoiceRepository, id: InvoiceId) -> Option<String> {
    repo.labels_for(&[id])
        .unwrap_or_default()
        .remove(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::audit_handlers::test_support::InMemoryAuditRepo;
    use crate::application::RepoError;
    use crate::application::ports::{ListInvoicesQuery, Page};
    use crate::domain::client::ClientId;
    use crate::domain::invoice::{Invoice, InvoiceId, InvoiceNumber};
    use crate::domain::money::{Currency, Money};
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;
    use std::collections::HashMap;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[derive(Default)]
    struct StubInvoiceRepo {
        labels: Mutex<HashMap<InvoiceId, String>>,
    }
    impl StubInvoiceRepo {
        fn with(id: InvoiceId, label: &str) -> Self {
            let mut m = HashMap::new();
            m.insert(id, label.to_string());
            Self { labels: Mutex::new(m) }
        }
    }
    impl InvoiceRepository for StubInvoiceRepo {
        fn insert(&self, _: &Invoice) -> Result<(), RepoError> { unimplemented!() }
        fn update(&self, _: &Invoice) -> Result<(), RepoError> { unimplemented!() }
        fn get(&self, _: InvoiceId) -> Result<Option<Invoice>, RepoError> { unimplemented!() }
        fn list(&self, _: ListInvoicesQuery) -> Result<Page<Invoice>, RepoError> {
            unimplemented!()
        }
        fn delete(&self, _: InvoiceId) -> Result<(), RepoError> { unimplemented!() }
        fn labels_for(
            &self,
            ids: &[InvoiceId],
        ) -> Result<HashMap<InvoiceId, String>, RepoError> {
            let g = self.labels.lock();
            Ok(ids
                .iter()
                .filter_map(|id| g.get(id).map(|l| (*id, l.clone())))
                .collect())
        }
    }

    #[test]
    fn finalized_handler_emits_entity_label_from_number() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let handler = InvoiceFinalizedAuditHandler::new(repo.clone());
        let id = InvoiceId::new();
        let client_id = ClientId::new();

        handler.handle(&InvoiceFinalized {
            id,
            client_id,
            number: InvoiceNumber(1001),
            total: Money::new(12_300, Currency::Eur),
            at: at(),
        });

        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "#1001");
        assert!(meta.get("number").is_none(), "number is redundant with entity_label");
    }

    #[test]
    fn draft_created_handler_resolves_entity_label_via_repo() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let id = InvoiceId::new();
        let invoices = Arc::new(StubInvoiceRepo::with(id, "Acme"));
        let handler = InvoiceDraftCreatedAuditHandler::new(repo.clone(), invoices);

        handler.handle(&InvoiceDraftCreated {
            id,
            client_id: ClientId::new(),
            total: Money::new(0, Currency::Eur),
            at: at(),
        });

        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Acme");
    }

    #[test]
    fn duplicated_handler_resolves_label_for_new_id() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let source_id = InvoiceId::new();
        let new_id = InvoiceId::new();
        let invoices = Arc::new(StubInvoiceRepo::with(new_id, "Acme"));
        let handler = InvoiceDuplicatedAuditHandler::new(repo.clone(), invoices);

        handler.handle(&InvoiceDuplicated {
            source_id,
            new_id,
            client_id: ClientId::new(),
            at: at(),
        });

        let rows = repo.rows.lock();
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["entity_label"], "Acme");
        assert_eq!(meta["source_id"], source_id.to_string());
    }
}
