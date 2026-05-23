use std::collections::HashMap;

use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;
use crate::domain::money::{Currency, Money};
use crate::domain::payment::{Payment, PaymentId};

pub trait PaymentRepository: Send + Sync {
    fn insert(&self, payment: &Payment) -> Result<(), RepoError>;
    fn update(&self, payment: &Payment) -> Result<(), RepoError>;
    fn get(&self, id: PaymentId) -> Result<Option<Payment>, RepoError>;
    fn list(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, RepoError>;
    fn delete(&self, id: PaymentId) -> Result<(), RepoError>;
    /// Sum of allocations targeting a given invoice, across all payments.
    /// The caller passes the invoice's own currency so the zero-allocations
    /// case returns `Money(0, invoice_currency)` instead of guessing.
    fn allocated_for_invoice(
        &self,
        id: InvoiceId,
        invoice_currency: Currency,
    ) -> Result<Money, RepoError>;
    /// Batch version of [`allocated_for_invoice`]: returns the allocated total
    /// for every id in `ids` that has at least one allocation. Invoices with
    /// no allocations are absent from the map (caller should default to zero
    /// in the invoice's own currency).
    fn allocated_for_invoices(
        &self,
        ids: &[InvoiceId],
    ) -> Result<HashMap<InvoiceId, Money>, RepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct ListPaymentsQuery {
    pub client_id: Option<ClientId>,
    /// Filter to payments that have at least one allocation pointing at
    /// this invoice. Used by the invoice viewer's "Payments" section.
    pub invoice_id: Option<InvoiceId>,
    pub search: Option<String>,
}
