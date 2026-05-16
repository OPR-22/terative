//! Audit-log handlers for `Payment` domain events.
//!
//! `PaymentRecorded` and `PaymentUpdated` fan out: one audit row per allocated
//! invoice (each scoped to that `invoice_id` + the `client_id`), so the
//! payment surfaces on every relevant invoice strip *and* the client tab and
//! dashboard. With zero allocations a single client-scoped row is written
//! instead — the payment is unallocated but still belongs to the client.
//!
//! Both handlers also resolve allocation invoice ids → invoice numbers via
//! the injected `InvoiceRepository`, so the audit row can carry the human
//! label `"#1001"` instead of a raw UUID — both in the per-row metadata
//! (collapsed view: "Payment €30 for Invoice #1001") and inside the
//! indexed-collection diff (expanded view: "Invoice #1001: €100 → €120").

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;

use super::project;
use crate::application::ports::{
    AuditRepository, ClientRepository, EventHandler, InvoiceRepository,
};
use crate::domain::audit::NewAudit;
use crate::domain::client::ClientId;
use crate::domain::events::payment_events::{PaymentDeleted, PaymentRecorded, PaymentUpdated};
use crate::domain::events::DomainEvent;
use crate::domain::field_change::{FieldChange, IndexedDelta};
use crate::domain::invoice::InvoiceId;
use crate::domain::money::Money;
use crate::domain::payment::{PaymentAllocation, PaymentId};

pub struct PaymentRecordedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    invoices: Arc<dyn InvoiceRepository>,
    clients: Arc<dyn ClientRepository>,
}
impl PaymentRecordedAuditHandler {
    pub fn new(
        repo: Arc<dyn AuditRepository>,
        invoices: Arc<dyn InvoiceRepository>,
        clients: Arc<dyn ClientRepository>,
    ) -> Self {
        Self { repo, invoices, clients }
    }
}
impl EventHandler<PaymentRecorded> for PaymentRecordedAuditHandler {
    fn handle(&self, event: &PaymentRecorded) {
        // Created events don't carry a diff (everything is new) — pass an
        // empty `changes` slice so the metadata is just the snapshot.
        let client_label = resolve_client_label(self.clients.as_ref(), event.client_id);
        fan_out_payment(
            self.repo.as_ref(),
            self.invoices.as_ref(),
            event.event_name(),
            event.id,
            event.client_id,
            event.amount,
            &event.allocations,
            &[],
            client_label.as_deref(),
            event.occurred_at(),
        );
    }
}

pub struct PaymentUpdatedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    invoices: Arc<dyn InvoiceRepository>,
    clients: Arc<dyn ClientRepository>,
}
impl PaymentUpdatedAuditHandler {
    pub fn new(
        repo: Arc<dyn AuditRepository>,
        invoices: Arc<dyn InvoiceRepository>,
        clients: Arc<dyn ClientRepository>,
    ) -> Self {
        Self { repo, invoices, clients }
    }
}
impl EventHandler<PaymentUpdated> for PaymentUpdatedAuditHandler {
    fn handle(&self, event: &PaymentUpdated) {
        // Same fan-out as Created, but each row also carries the per-field
        // diff so the FE can render "amount: 50 → 60; allocation to invoice
        // X added".
        let client_label = resolve_client_label(self.clients.as_ref(), event.client_id);
        fan_out_payment(
            self.repo.as_ref(),
            self.invoices.as_ref(),
            event.event_name(),
            event.id,
            event.client_id,
            event.amount,
            &event.allocations,
            &event.changes,
            client_label.as_deref(),
            event.occurred_at(),
        );
    }
}

pub struct PaymentDeletedAuditHandler {
    repo: Arc<dyn AuditRepository>,
    clients: Arc<dyn ClientRepository>,
}
impl PaymentDeletedAuditHandler {
    pub fn new(repo: Arc<dyn AuditRepository>, clients: Arc<dyn ClientRepository>) -> Self {
        Self { repo, clients }
    }
}
impl EventHandler<PaymentDeleted> for PaymentDeletedAuditHandler {
    fn handle(&self, event: &PaymentDeleted) {
        // Delete is single-row: we no longer know the prior allocations
        // (the payment is gone), so it surfaces on the client tab + dashboard
        // only, never on invoice strips.
        let entity_label = resolve_client_label(self.clients.as_ref(), event.client_id);
        project(
            self.repo.as_ref(),
            NewAudit {
                event_type: event.event_name().into(),
                entity_type: "payment".into(),
                entity_id: Some(event.id.to_string()),
                client_id: Some(event.client_id),
                invoice_id: None,
                metadata_json: json!({ "entity_label": entity_label }).to_string(),
                occurred_at: event.occurred_at(),
            },
        );
    }
}

fn resolve_client_label(repo: &dyn ClientRepository, id: ClientId) -> Option<String> {
    repo.labels_for(&[id])
        .unwrap_or_default()
        .remove(&id)
}

#[allow(clippy::too_many_arguments)]
fn fan_out_payment(
    repo: &dyn AuditRepository,
    invoices: &dyn InvoiceRepository,
    event_name: &'static str,
    payment_id: PaymentId,
    client_id: ClientId,
    amount: Money,
    allocations: &[PaymentAllocation],
    changes: &[FieldChange],
    client_label: Option<&str>,
    occurred_at: chrono::DateTime<chrono::Utc>,
) {
    // Collect every invoice id we need to label: the current allocations
    // (for per-row labels) plus any allocation deltas referenced inside the
    // `changes` diff (added/removed/changed by invoice uuid string).
    let mut needed: Vec<InvoiceId> = allocations.iter().map(|a| a.invoice_id).collect();
    for change in changes {
        if let FieldChange::IndexedCollection {
            field,
            added,
            removed,
            changed,
        } = change
        {
            if *field == "allocations" {
                for d in added.iter().chain(removed.iter()).chain(changed.iter()) {
                    if let Ok(id) = uuid::Uuid::parse_str(&d.key).map(InvoiceId) {
                        if !needed.contains(&id) {
                            needed.push(id);
                        }
                    }
                }
            }
        }
    }
    // Lookup failure is non-fatal — without labels the audit row still works,
    // it just shows raw UUIDs in the diff and no allocation invoice label
    // on the row.
    let labels = invoices.labels_for(&needed).unwrap_or_default();

    // Build the enriched changes once (allocations indexed-collection gets
    // its per-entry `label` populated; everything else passes through).
    let enriched_changes: Vec<FieldChange> = changes
        .iter()
        .map(|c| enrich_change(c, &labels))
        .collect();

    let base = json!({
        "amount": amount.format(),
        "allocations_count": allocations.len(),
        // Always present; empty array on Created events. Same diff is
        // repeated on every fanned-out row so each invoice strip sees the
        // full update context.
        "changes": enriched_changes,
        // Client name as the universal "what is this row about" label.
        // Per-row context (invoice label) lives in `allocation_invoice_label`.
        "entity_label": client_label,
    });
    if allocations.is_empty() {
        // Unallocated payment — one client-scoped row.
        project(
            repo,
            NewAudit {
                event_type: event_name.into(),
                entity_type: "payment".into(),
                entity_id: Some(payment_id.to_string()),
                client_id: Some(client_id),
                invoice_id: None,
                metadata_json: base.to_string(),
                occurred_at,
            },
        );
        return;
    }
    // One row per allocated invoice — surfaces on each invoice strip plus
    // the client tab via `client_id`.
    for alloc in allocations {
        let mut meta = base.clone();
        meta["allocation_amount"] = json!(alloc.amount.format());
        if let Some(label) = labels.get(&alloc.invoice_id) {
            meta["allocation_invoice_label"] = json!(label);
        }
        project(
            repo,
            NewAudit {
                event_type: event_name.into(),
                entity_type: "payment".into(),
                entity_id: Some(payment_id.to_string()),
                client_id: Some(client_id),
                invoice_id: Some(alloc.invoice_id),
                metadata_json: meta.to_string(),
                occurred_at,
            },
        );
    }
}

/// Walk a single [`FieldChange`]; if it's the `allocations` indexed-collection,
/// populate each [`IndexedDelta::label`] with the resolved invoice label
/// (e.g. `"#1001"`). Everything else passes through unchanged.
fn enrich_change(
    change: &FieldChange,
    labels: &HashMap<InvoiceId, String>,
) -> FieldChange {
    match change {
        FieldChange::IndexedCollection {
            field,
            added,
            removed,
            changed,
        } if *field == "allocations" => FieldChange::IndexedCollection {
            field,
            added: added.iter().map(|d| with_invoice_label(d, labels)).collect(),
            removed: removed.iter().map(|d| with_invoice_label(d, labels)).collect(),
            changed: changed.iter().map(|d| with_invoice_label(d, labels)).collect(),
        },
        other => other.clone(),
    }
}

fn with_invoice_label(
    d: &IndexedDelta,
    labels: &HashMap<InvoiceId, String>,
) -> IndexedDelta {
    let label = uuid::Uuid::parse_str(&d.key)
        .ok()
        .map(InvoiceId)
        .and_then(|id| labels.get(&id))
        .cloned();
    IndexedDelta {
        key: d.key.clone(),
        label,
        from: d.from.clone(),
        to: d.to.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::audit_handlers::test_support::InMemoryAuditRepo;
    use crate::application::ports::{ClientAttributeValues, ListClientsQuery, ListInvoicesQuery};
    use crate::application::RepoError;
    use crate::application::ports::Page;
    use crate::domain::client::{Client, ClientId};
    use crate::domain::invoice::{Invoice, InvoiceId};
    use crate::domain::money::{Currency, Money};
    use crate::domain::payment::{PaymentAllocation, PaymentId};
    use chrono::{DateTime, Utc};
    use parking_lot::Mutex;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    /// In-memory `InvoiceRepository` that only implements `labels_for` (the
    /// only method the audit handler calls). Other methods are unimplemented;
    /// the handler never reaches them.
    #[derive(Default)]
    struct StubInvoiceRepo {
        labels: Mutex<HashMap<InvoiceId, String>>,
    }
    impl StubInvoiceRepo {
        fn with(labels: &[(InvoiceId, &str)]) -> Self {
            let map = labels.iter().map(|(id, l)| (*id, (*l).to_string())).collect();
            Self {
                labels: Mutex::new(map),
            }
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
    fn unallocated_payment_writes_one_client_scoped_row_with_client_entity_label() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let invoices = Arc::new(StubInvoiceRepo::default());
        let payment_id = PaymentId::new();
        let client_id = ClientId::new();
        let clients = Arc::new(StubClientRepo::with(client_id, "Acme"));
        let handler = PaymentRecordedAuditHandler::new(repo.clone(), invoices, clients);

        handler.handle(&PaymentRecorded {
            id: payment_id,
            client_id,
            amount: Money::new(50_000, Currency::Eur),
            allocations: vec![],
            at: at(),
        });

        let rows = repo.rows.lock();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_type, "payment.recorded");
        assert_eq!(rows[0].client_id, Some(client_id));
        assert_eq!(rows[0].invoice_id, None);
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        assert_eq!(meta["amount"], "500.00 EUR");
        assert_eq!(meta["allocations_count"], 0);
        assert_eq!(meta["entity_label"], "Acme");
        // No allocation → no per-row invoice label.
        assert!(meta.get("allocation_invoice_label").is_none());
    }

    #[test]
    fn payment_with_two_allocations_writes_one_row_per_invoice_with_invoice_labels() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let inv_a = InvoiceId::new();
        let inv_b = InvoiceId::new();
        let invoices = Arc::new(StubInvoiceRepo::with(&[(inv_a, "#1001"), (inv_b, "#1002")]));
        let payment_id = PaymentId::new();
        let client_id = ClientId::new();
        let clients = Arc::new(StubClientRepo::with(client_id, "Acme"));
        let handler = PaymentRecordedAuditHandler::new(repo.clone(), invoices, clients);

        handler.handle(&PaymentRecorded {
            id: payment_id,
            client_id,
            amount: Money::new(50_000, Currency::Eur),
            allocations: vec![
                PaymentAllocation {
                    invoice_id: inv_a,
                    amount: Money::new(30_000, Currency::Eur),
                },
                PaymentAllocation {
                    invoice_id: inv_b,
                    amount: Money::new(20_000, Currency::Eur),
                },
            ],
            at: at(),
        });

        let rows = repo.rows.lock();
        assert_eq!(rows.len(), 2);
        for r in rows.iter() {
            assert_eq!(r.event_type, "payment.recorded");
            let meta: serde_json::Value = serde_json::from_str(&r.metadata_json).unwrap();
            assert_eq!(meta["allocations_count"], 2);
            // The per-row invoice label matches the row's invoice_id scope.
            let expected = match r.invoice_id {
                Some(id) if id == inv_a => "#1001",
                Some(id) if id == inv_b => "#1002",
                _ => panic!("unexpected invoice_id"),
            };
            assert_eq!(meta["allocation_invoice_label"], expected);
        }
    }

    #[test]
    fn payment_updated_enriches_indexed_collection_with_invoice_labels() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let inv = InvoiceId::new();
        let invoices = Arc::new(StubInvoiceRepo::with(&[(inv, "#1234")]));
        let client_id = ClientId::new();
        let clients = Arc::new(StubClientRepo::with(client_id, "Acme"));
        let handler = PaymentUpdatedAuditHandler::new(repo.clone(), invoices, clients);

        // A change payload mentioning one added allocation by uuid string.
        let added_delta = IndexedDelta {
            key: inv.to_string(),
            label: None,
            from: None,
            to: Some(serde_json::json!({"currency": "EUR", "amount": "10.00"})),
        };
        let changes = vec![FieldChange::IndexedCollection {
            field: "allocations",
            added: vec![added_delta],
            removed: vec![],
            changed: vec![],
        }];

        handler.handle(&PaymentUpdated {
            id: PaymentId::new(),
            client_id,
            amount: Money::new(1_000, Currency::Eur),
            allocations: vec![PaymentAllocation {
                invoice_id: inv,
                amount: Money::new(1_000, Currency::Eur),
            }],
            changes,
            at: at(),
        });

        let rows = repo.rows.lock();
        assert_eq!(rows.len(), 1);
        let meta: serde_json::Value = serde_json::from_str(&rows[0].metadata_json).unwrap();
        // Per-row invoice label is set.
        assert_eq!(meta["allocation_invoice_label"], "#1234");
        // The indexed_collection inside `changes` has its `label` populated.
        let added = &meta["changes"][0]["added"];
        assert_eq!(added[0]["key"], inv.to_string());
        assert_eq!(added[0]["label"], "#1234");
    }
}
