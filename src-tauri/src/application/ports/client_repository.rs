use crate::application::RepoError;
use crate::application::ports::pagination::{Page, PaginationParams};
use crate::domain::client::{Client, ClientId};

pub trait ClientRepository: Send + Sync {
    fn insert(&self, client: &Client) -> Result<(), RepoError>;
    fn update(&self, client: &Client) -> Result<(), RepoError>;
    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError>;
    fn list(&self, query: ListClientsQuery) -> Result<Page<Client>, RepoError>;
    /// Distinct, non-null, alphabetically sorted values currently used on
    /// existing clients. Lets the UI show "previously used" suggestions
    /// (autocomplete) for free-form fields without forcing a fixed list —
    /// new entries grow the catalogue automatically.
    fn distinct_attribute_values(&self) -> Result<ClientAttributeValues, RepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct ListClientsQuery {
    pub search: Option<String>,
    pub include_archived: bool,
    pub pagination: PaginationParams,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientAttributeValues {
    pub gender: Vec<String>,
    pub pronouns: Vec<String>,
    pub occupation: Vec<String>,
}
