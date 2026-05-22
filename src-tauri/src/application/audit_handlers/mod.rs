//! Audit-log projectors: one [`EventHandler`](crate::application::ports::EventHandler)
//! per domain-event type. Each handler projects its event into a single
//! `Audit` row.
//!
//! Handlers swallow their own errors (log to stderr): the audit log is a
//! UX read-model, never a transactional invariant, so a failed projection
//! must not roll back the business action that produced the event. See the
//! rationale in `application::ports::event_bus`.
//!
//! There is no central dispatcher — every event type has its own handler
//! struct and its own `register` call in [`register_all`]. The wiring
//! integration test asserts none is missed.

mod backup;
mod catalog_item;
mod client;
mod invoice;
mod payment;
mod tax;

pub use backup::*;
pub use catalog_item::*;
pub use client::*;
pub use invoice::*;
pub use payment::*;
pub use tax::*;

use std::sync::Arc;

use crate::adapters::event_bus::InProcessEventBus;
use crate::application::ports::AuditRepository;
use crate::domain::audit::{Audit, NewAudit};

/// Build an `Audit` from `new` and append it via `repo`, logging — never
/// propagating — any failure. The single seam every handler funnels through.
fn project(repo: &dyn AuditRepository, new: NewAudit) {
    let event_type = new.event_type.clone();
    match Audit::record(new) {
        Ok(audit) => {
            if let Err(e) = repo.insert(&audit) {
                eprintln!("audit projector: insert failed for {event_type}: {e}");
            }
        }
        Err(e) => eprintln!("audit projector: invalid audit for {event_type}: {e}"),
    }
}

/// Repositories the audit handlers need to resolve IDs into user-facing
/// labels at write time (e.g. invoice id → `"#1001"`, client id → `"Acme"`).
/// Passed to [`register_all`] so the wiring isn't a long arg list.
pub struct AuditHandlerContext {
    pub audits: Arc<dyn AuditRepository>,
    pub invoices: Arc<dyn crate::application::ports::InvoiceRepository>,
    pub clients: Arc<dyn crate::application::ports::ClientRepository>,
    pub catalog_items: Arc<dyn crate::application::ports::CatalogItemRepository>,
    pub taxes: Arc<dyn crate::application::ports::TaxRepository>,
}

/// Register every audit-log handler against `bus`. One `register` call per
/// (handler, event) pair.
pub fn register_all(bus: &mut InProcessEventBus, ctx: AuditHandlerContext) {
    use crate::application::events::BackupCreated;
    use crate::domain::events::catalog_item_events::{CatalogItemCreated, CatalogItemUpdated};
    use crate::domain::events::client_events::{
        ClientArchived, ClientCreated, ClientUnarchived, ClientUpdated,
    };
    use crate::domain::events::invoice_events::{
        InvoiceCancelled, InvoiceDraftCreated, InvoiceDraftUpdated, InvoiceDuplicated,
        InvoiceFinalized, InvoiceSent,
    };
    use crate::domain::events::payment_events::{
        PaymentDeleted, PaymentRecorded, PaymentUpdated,
    };
    use crate::domain::events::tax_events::{TaxCreated, TaxUpdated};

    let AuditHandlerContext {
        audits,
        invoices,
        clients,
        catalog_items,
        taxes,
    } = ctx;

    bus.register::<InvoiceDraftCreated, _>(InvoiceDraftCreatedAuditHandler::new(
        audits.clone(),
        invoices.clone(),
    ));
    bus.register::<InvoiceDraftUpdated, _>(InvoiceDraftUpdatedAuditHandler::new(
        audits.clone(),
        invoices.clone(),
    ));
    bus.register::<InvoiceFinalized, _>(InvoiceFinalizedAuditHandler::new(audits.clone()));
    bus.register::<InvoiceCancelled, _>(InvoiceCancelledAuditHandler::new(audits.clone()));
    bus.register::<InvoiceDuplicated, _>(InvoiceDuplicatedAuditHandler::new(
        audits.clone(),
        invoices.clone(),
    ));
    bus.register::<InvoiceSent, _>(InvoiceSentAuditHandler::new(audits.clone()));
    bus.register::<ClientCreated, _>(ClientCreatedAuditHandler::new(audits.clone()));
    bus.register::<ClientUpdated, _>(ClientUpdatedAuditHandler::new(
        audits.clone(),
        clients.clone(),
    ));
    bus.register::<ClientArchived, _>(ClientArchivedAuditHandler::new(audits.clone()));
    bus.register::<ClientUnarchived, _>(ClientUnarchivedAuditHandler::new(audits.clone()));
    bus.register::<PaymentRecorded, _>(PaymentRecordedAuditHandler::new(
        audits.clone(),
        invoices.clone(),
        clients.clone(),
    ));
    bus.register::<PaymentUpdated, _>(PaymentUpdatedAuditHandler::new(
        audits.clone(),
        invoices,
        clients.clone(),
    ));
    bus.register::<PaymentDeleted, _>(PaymentDeletedAuditHandler::new(
        audits.clone(),
        clients,
    ));
    bus.register::<CatalogItemCreated, _>(CatalogItemCreatedAuditHandler::new(audits.clone()));
    bus.register::<CatalogItemUpdated, _>(CatalogItemUpdatedAuditHandler::new(
        audits.clone(),
        catalog_items,
    ));
    bus.register::<TaxCreated, _>(TaxCreatedAuditHandler::new(audits.clone()));
    bus.register::<TaxUpdated, _>(TaxUpdatedAuditHandler::new(audits.clone(), taxes));
    bus.register::<BackupCreated, _>(BackupCreatedAuditHandler::new(audits));
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::application::ports::{Page, PaginationParams};
    use crate::application::RepoError;
    use crate::domain::audit::Audit;
    use crate::domain::client::ClientId;
    use crate::domain::invoice::InvoiceId;
    use parking_lot::Mutex;

    /// In-memory `AuditRepository` for handler and use-case tests.
    #[derive(Default)]
    pub struct InMemoryAuditRepo {
        pub rows: Mutex<Vec<Audit>>,
    }

    fn paginate(mut v: Vec<Audit>, params: &PaginationParams) -> Page<Audit> {
        let total = v.len() as u64;
        let offset = params.offset() as usize;
        let end = (offset + params.per_page as usize).min(v.len());
        let slice = if offset >= v.len() {
            Vec::new()
        } else {
            v.drain(offset..end).collect()
        };
        Page::new(slice, total, params)
    }

    impl AuditRepository for InMemoryAuditRepo {
        fn insert(&self, audit: &Audit) -> Result<(), RepoError> {
            self.rows.lock().push(audit.clone());
            Ok(())
        }
        fn paginate_recent(
            &self,
            params: &PaginationParams,
        ) -> Result<Page<Audit>, RepoError> {
            let mut v = self.rows.lock().clone();
            v.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
            Ok(paginate(v, params))
        }
        fn paginate_by_client(
            &self,
            client_id: ClientId,
            params: &PaginationParams,
        ) -> Result<Page<Audit>, RepoError> {
            let mut v: Vec<Audit> = self
                .rows
                .lock()
                .iter()
                .filter(|a| a.client_id == Some(client_id))
                .cloned()
                .collect();
            v.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
            Ok(paginate(v, params))
        }
        fn paginate_by_invoice(
            &self,
            invoice_id: InvoiceId,
            params: &PaginationParams,
        ) -> Result<Page<Audit>, RepoError> {
            let mut v: Vec<Audit> = self
                .rows
                .lock()
                .iter()
                .filter(|a| a.invoice_id == Some(invoice_id))
                .cloned()
                .collect();
            v.sort_by(|a, b| a.occurred_at.cmp(&b.occurred_at));
            Ok(paginate(v, params))
        }
        fn delete_older_than(
            &self,
            cutoff: chrono::DateTime<chrono::Utc>,
        ) -> Result<u64, RepoError> {
            let mut g = self.rows.lock();
            let before = g.len();
            g.retain(|a| a.occurred_at >= cutoff);
            Ok((before - g.len()) as u64)
        }
    }
}
