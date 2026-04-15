use std::sync::Arc;

use chrono::Utc;

use crate::application::ports::{ClientRepository, ListClientsQuery};
use crate::application::AppError;
use crate::domain::client::{Client, ClientId, NewClient, NewContactEntry};

pub struct CreateClient {
    repo: Arc<dyn ClientRepository>,
}

impl CreateClient {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: NewClient) -> Result<Client, AppError> {
        let client = Client::create(input, Utc::now())?;
        self.repo.insert(&client)?;
        Ok(client)
    }
}

pub struct UpdateClient {
    repo: Arc<dyn ClientRepository>,
}

#[derive(Debug, Clone)]
pub struct UpdateClientInput {
    pub id: ClientId,
    pub name: String,
    pub emails: Vec<NewContactEntry>,
    pub phones: Vec<NewContactEntry>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<ClientId>,
}

impl UpdateClient {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: UpdateClientInput) -> Result<Client, AppError> {
        let mut client = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(crate::domain::client::ClientError::EmptyName.into());
        }
        client.name = name;
        client.replace_emails(input.emails)?;
        client.replace_phones(input.phones)?;
        client.address = normalize(input.address);
        client.notes = normalize(input.notes);
        client.set_referred_by(input.referred_by)?;
        self.repo.update(&client)?;
        Ok(client)
    }
}

pub struct ArchiveClient {
    repo: Arc<dyn ClientRepository>,
}

impl ArchiveClient {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: ClientId) -> Result<(), AppError> {
        let mut client = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        client.deactivate();
        self.repo.update(&client)?;
        Ok(())
    }
}

pub struct UnarchiveClient {
    repo: Arc<dyn ClientRepository>,
}

impl UnarchiveClient {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: ClientId) -> Result<(), AppError> {
        let mut client = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        client.reactivate();
        self.repo.update(&client)?;
        Ok(())
    }
}

pub struct ListClients {
    repo: Arc<dyn ClientRepository>,
}

impl ListClients {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, query: ListClientsQuery) -> Result<Vec<Client>, AppError> {
        Ok(self.repo.list(query)?)
    }
}

pub struct GetClientDetail {
    repo: Arc<dyn ClientRepository>,
}

impl GetClientDetail {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: ClientId) -> Result<Client, AppError> {
        self.repo.get(id)?.ok_or(AppError::NotFound)
    }
}

fn normalize(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryClientRepo {
        inner: Mutex<HashMap<ClientId, Client>>,
    }

    impl ClientRepository for InMemoryClientRepo {
        fn insert(&self, client: &Client) -> Result<(), RepoError> {
            self.inner.lock().insert(client.id, client.clone());
            Ok(())
        }
        fn update(&self, client: &Client) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&client.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(client.id, client.clone());
            Ok(())
        }
        fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, query: ListClientsQuery) -> Result<Vec<Client>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Client> = g
                .values()
                .filter(|c| query.include_inactive || c.active)
                .filter(|c| {
                    query
                        .search
                        .as_deref()
                        .map(|s| c.name.to_lowercase().contains(&s.to_lowercase()))
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(v)
        }
    }

    fn make_repo() -> Arc<InMemoryClientRepo> {
        Arc::new(InMemoryClientRepo::default())
    }

    #[test]
    fn create_client_persists_and_returns_entity() {
        let repo = make_repo();
        let uc = CreateClient::new(repo.clone());
        let c = uc
            .execute(NewClient {
                name: "Acme".into(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(c.name, "Acme");
        assert_eq!(repo.inner.lock().len(), 1);
    }

    fn email(value: &str, is_default: bool) -> NewContactEntry {
        NewContactEntry {
            value: value.into(),
            label: None,
            is_default,
        }
    }

    #[test]
    fn update_client_changes_fields() {
        let repo = make_repo();
        let created = CreateClient::new(repo.clone())
            .execute(NewClient {
                name: "Old".into(),
                emails: vec![email("old@x.com", true)],
                ..Default::default()
            })
            .unwrap();
        let updated = UpdateClient::new(repo.clone())
            .execute(UpdateClientInput {
                id: created.id,
                name: "New Name".into(),
                emails: vec![email("new@x.com", true)],
                phones: vec![],
                address: None,
                notes: None,
                referred_by: None,
            })
            .unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.default_email(), Some("new@x.com"));
    }

    #[test]
    fn update_client_rejects_missing_id() {
        let repo = make_repo();
        let err = UpdateClient::new(repo)
            .execute(UpdateClientInput {
                id: ClientId::new(),
                name: "X".into(),
                emails: vec![],
                phones: vec![],
                address: None,
                notes: None,
                referred_by: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn update_client_rejects_empty_name() {
        let repo = make_repo();
        let c = CreateClient::new(repo.clone())
            .execute(NewClient {
                name: "Acme".into(),
                ..Default::default()
            })
            .unwrap();
        let err = UpdateClient::new(repo)
            .execute(UpdateClientInput {
                id: c.id,
                name: "   ".into(),
                emails: vec![],
                phones: vec![],
                address: None,
                notes: None,
                referred_by: None,
            })
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Client(crate::domain::client::ClientError::EmptyName)
        ));
    }

    #[test]
    fn update_client_rejects_self_referral() {
        let repo = make_repo();
        let c = CreateClient::new(repo.clone())
            .execute(NewClient {
                name: "Acme".into(),
                ..Default::default()
            })
            .unwrap();
        let err = UpdateClient::new(repo)
            .execute(UpdateClientInput {
                id: c.id,
                name: "Acme".into(),
                emails: vec![],
                phones: vec![],
                address: None,
                notes: None,
                referred_by: Some(c.id),
            })
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Client(crate::domain::client::ClientError::SelfReferral)
        ));
    }

    #[test]
    fn archive_client_deactivates_entity() {
        let repo = make_repo();
        let c = CreateClient::new(repo.clone())
            .execute(NewClient {
                name: "Acme".into(),
                ..Default::default()
            })
            .unwrap();
        ArchiveClient::new(repo.clone()).execute(c.id).unwrap();
        let stored = repo.inner.lock().get(&c.id).cloned().unwrap();
        assert!(!stored.active);
    }

    #[test]
    fn unarchive_client_reactivates_entity() {
        let repo = make_repo();
        let c = CreateClient::new(repo.clone())
            .execute(NewClient {
                name: "Acme".into(),
                ..Default::default()
            })
            .unwrap();
        ArchiveClient::new(repo.clone()).execute(c.id).unwrap();
        UnarchiveClient::new(repo.clone()).execute(c.id).unwrap();
        let stored = repo.inner.lock().get(&c.id).cloned().unwrap();
        assert!(stored.active);
    }

    #[test]
    fn list_clients_filters_inactive_by_default() {
        let repo = make_repo();
        let create = CreateClient::new(repo.clone());
        let a = create
            .execute(NewClient {
                name: "Alpha".into(),
                ..Default::default()
            })
            .unwrap();
        let _b = create
            .execute(NewClient {
                name: "Beta".into(),
                ..Default::default()
            })
            .unwrap();
        ArchiveClient::new(repo.clone()).execute(a.id).unwrap();

        let list = ListClients::new(repo.clone())
            .execute(ListClientsQuery::default())
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Beta");

        let all = ListClients::new(repo.clone())
            .execute(ListClientsQuery {
                include_inactive: true,
                search: None,
            })
            .unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn list_clients_search_by_substring() {
        let repo = make_repo();
        let create = CreateClient::new(repo.clone());
        create
            .execute(NewClient {
                name: "Acme Corp".into(),
                ..Default::default()
            })
            .unwrap();
        create
            .execute(NewClient {
                name: "Globex".into(),
                ..Default::default()
            })
            .unwrap();
        let list = ListClients::new(repo.clone())
            .execute(ListClientsQuery {
                search: Some("acm".into()),
                include_inactive: false,
            })
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Acme Corp");
    }

    #[test]
    fn get_client_detail_returns_not_found() {
        let repo = make_repo();
        let err = GetClientDetail::new(repo)
            .execute(ClientId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }
}
