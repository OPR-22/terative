use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;
use crate::domain::money::Money;
use crate::domain::payment::{Payment, PaymentId};

pub trait PaymentRepository: Send + Sync {
    fn insert(&self, payment: &Payment) -> Result<(), RepoError>;
    fn update(&self, payment: &Payment) -> Result<(), RepoError>;
    fn get(&self, id: PaymentId) -> Result<Option<Payment>, RepoError>;
    fn list(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, RepoError>;
    fn delete(&self, id: PaymentId) -> Result<(), RepoError>;
    /// Sum of allocations targeting a given invoice, across all payments.
    fn allocated_for_invoice(&self, id: InvoiceId) -> Result<Money, RepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct ListPaymentsQuery {
    pub client_id: Option<ClientId>,
    pub search: Option<String>,
}
