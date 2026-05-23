use std::collections::HashMap;

use crate::application::RepoError;
use crate::application::ports::pagination::{Page, PaginationParams};
use crate::domain::client::ClientId;
use crate::domain::invoice::{Invoice, InvoiceId, InvoiceStatus};

pub trait InvoiceRepository: Send + Sync {
    fn insert(&self, invoice: &Invoice) -> Result<(), RepoError>;
    fn update(&self, invoice: &Invoice) -> Result<(), RepoError>;
    fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError>;
    fn list(&self, query: ListInvoicesQuery) -> Result<Page<Invoice>, RepoError>;
    fn delete(&self, id: InvoiceId) -> Result<(), RepoError>;

    /// Batch fetch of user-facing labels for invoices — `"#1001"` for
    /// finalized invoices (those with an assigned number), or `"#-"` for
    /// drafts that haven't been assigned a number yet. Missing entries mean
    /// the invoice doesn't exist. Used by audit handlers to render
    /// `entity_label` strings without an N+1 lookup.
    fn labels_for(
        &self,
        ids: &[InvoiceId],
    ) -> Result<HashMap<InvoiceId, String>, RepoError>;

    /// True when at least one invoice has been assigned a number (i.e. has
    /// been finalized). Used to decide whether the invoice-number sequence's
    /// starting point can still be reconfigured — once a number has been
    /// handed out, changing the start would risk colliding with it.
    fn has_numbered_invoices(&self) -> Result<bool, RepoError>;
}

/// User-facing groupings over the underlying `DerivedPaymentStatus` —
/// the InvoiceList page exposes these as a second pills filter, in
/// addition to lifecycle status. Defined here (alongside the query)
/// because it's a read-path concept driven entirely by repo SQL.
///
/// `Unpaid` deliberately groups `DerivedPaymentStatus::Unpaid` with
/// `Partial`: from a "money still owed" perspective they're the same
/// thing to a user, and surfacing two near-identical pills would be
/// noise. `Late` is the renamed `Overdue` for the same UX reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoicePaymentFilter {
    Paid,
    Unpaid,
    Late,
}

#[derive(Debug, Clone, Default)]
pub struct ListInvoicesQuery {
    pub status: Option<InvoiceStatus>,
    pub client_id: Option<ClientId>,
    pub search: Option<String>,
    pub payment_filter: Option<InvoicePaymentFilter>,
    pub pagination: PaginationParams,
}
