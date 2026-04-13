use crate::application::RepoError;
use crate::domain::client::{Client, ClientId};

pub trait ClientRepository: Send + Sync {
    fn insert(&self, client: &Client) -> Result<(), RepoError>;
    fn update(&self, client: &Client) -> Result<(), RepoError>;
    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError>;
    fn list(&self, query: ListClientsQuery) -> Result<Vec<Client>, RepoError>;
    fn has_invoices(&self, id: ClientId) -> Result<bool, RepoError>;
    fn delete(&self, id: ClientId) -> Result<(), RepoError>;
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ListClientsQuery {
    pub search: Option<String>,
    pub include_inactive: bool,
}
