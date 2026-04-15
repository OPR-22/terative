use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::{Invoice, InvoiceId, InvoiceStatus};

pub trait InvoiceRepository: Send + Sync {
    fn insert(&self, invoice: &Invoice) -> Result<(), RepoError>;
    fn update(&self, invoice: &Invoice) -> Result<(), RepoError>;
    fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError>;
    fn list(&self, query: ListInvoicesQuery) -> Result<Vec<Invoice>, RepoError>;
    fn delete(&self, id: InvoiceId) -> Result<(), RepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct ListInvoicesQuery {
    pub status: Option<InvoiceStatus>,
    pub client_id: Option<ClientId>,
    pub search: Option<String>,
}
