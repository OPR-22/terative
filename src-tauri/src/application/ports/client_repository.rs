use crate::application::RepoError;
use crate::application::ports::pagination::{Page, PaginationParams};
use crate::domain::client::{Client, ClientId};

pub trait ClientRepository: Send + Sync {
    fn insert(&self, client: &Client) -> Result<(), RepoError>;
    fn update(&self, client: &Client) -> Result<(), RepoError>;
    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError>;
    fn list(&self, query: ListClientsQuery) -> Result<Page<Client>, RepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct ListClientsQuery {
    pub search: Option<String>,
    pub include_inactive: bool,
    pub pagination: PaginationParams,
}
