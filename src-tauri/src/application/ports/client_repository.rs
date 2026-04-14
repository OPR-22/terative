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
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub include_inactive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_object_uses_defaults() {
        let q: ListClientsQuery =
            serde_json::from_str("{}").expect("empty object must deserialize");
        assert_eq!(q.search, None);
        assert!(!q.include_inactive);
    }

    #[test]
    fn deserialize_partial_object_fills_missing_fields() {
        let q: ListClientsQuery = serde_json::from_str(r#"{"search": "acme"}"#)
            .expect("partial object must deserialize");
        assert_eq!(q.search.as_deref(), Some("acme"));
        assert!(!q.include_inactive);
    }
}
