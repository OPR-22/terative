use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::application::ports::{
    ClientAttributeValues, ClientRepository, ListClientsQuery, Page,
};
use crate::application::AppError;
use crate::domain::client::{
    Client, ClientId, ClientKind, NewClient, NewClientAddress, NewContactEntry,
};

#[derive(Clone)]
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
    pub kind: ClientKind,
    pub name: String,
    pub contact_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub emails: Vec<NewContactEntry>,
    pub phones: Vec<NewContactEntry>,
    pub addresses: Vec<NewClientAddress>,
    pub notes: Option<String>,
    pub referred_by: Option<ClientId>,
    pub date_of_birth: Option<NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
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
        if let Some(d) = input.date_of_birth {
            if d > Utc::now().date_naive() {
                return Err(crate::domain::client::ClientError::FutureDateOfBirth.into());
            }
        }
        client.kind = input.kind;
        client.name = name;
        client.contact_name = normalize(input.contact_name);
        client.tax_id = normalize(input.tax_id);
        client.registration_number = normalize(input.registration_number);
        client.replace_emails(input.emails)?;
        client.replace_phones(input.phones)?;
        client.replace_addresses(input.addresses)?;
        client.notes = normalize(input.notes);
        client.set_referred_by(input.referred_by)?;
        client.date_of_birth = input.date_of_birth;
        client.sex = normalize(input.sex);
        client.gender = normalize(input.gender);
        client.pronouns = normalize(input.pronouns);
        client.occupation = normalize(input.occupation);
        client.language = normalize(input.language);
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
        client.archive(Utc::now());
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
        client.unarchive();
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

    pub fn execute(&self, query: ListClientsQuery) -> Result<Page<Client>, AppError> {
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

/// Returns the sets of values currently used on existing clients for the
/// free-form attribute fields (sex, gender, pronouns, occupation). The UI
/// uses this as a "previously used" autocomplete catalogue — new entries
/// grow the list automatically without requiring a separate management UI.
pub struct ListClientAttributeValues {
    repo: Arc<dyn ClientRepository>,
}

impl ListClientAttributeValues {
    pub fn new(repo: Arc<dyn ClientRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self) -> Result<ClientAttributeValues, AppError> {
        Ok(self.repo.distinct_attribute_values()?)
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
    use crate::application::ports::PaginationParams;
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
        fn list(&self, query: ListClientsQuery) -> Result<Page<Client>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Client> = g
                .values()
                .filter(|c| query.include_archived || !c.is_archived())
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
            let total = v.len() as u64;
            Ok(Page::new(v, total, &PaginationParams::default()))
        }
        fn names_for(
            &self,
            ids: &[ClientId],
        ) -> Result<HashMap<ClientId, String>, RepoError> {
            let g = self.inner.lock();
            Ok(ids
                .iter()
                .filter_map(|id| g.get(id).map(|c| (*id, c.name.clone())))
                .collect())
        }
        fn distinct_attribute_values(
            &self,
        ) -> Result<crate::application::ports::ClientAttributeValues, RepoError> {
            let g = self.inner.lock();
            let collect = |f: fn(&Client) -> Option<&str>| -> Vec<String> {
                let mut v: Vec<String> = g
                    .values()
                    .filter_map(|c| f(c).map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect();
                v.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                v.dedup();
                v
            };
            Ok(crate::application::ports::ClientAttributeValues {
                gender: collect(|c| c.gender.as_deref()),
                pronouns: collect(|c| c.pronouns.as_deref()),
                occupation: collect(|c| c.occupation.as_deref()),
            })
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
                kind: ClientKind::Individual,
                contact_name: None,
                tax_id: None,
                registration_number: None,
                emails: vec![email("new@x.com", true)],
                phones: vec![],
                addresses: vec![],
                notes: None,
                referred_by: None,
                date_of_birth: None,
                sex: None,
                gender: None,
                pronouns: None,
                occupation: None,
                language: None,
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
                kind: ClientKind::Individual,
                contact_name: None,
                tax_id: None,
                registration_number: None,
                emails: vec![],
                phones: vec![],
                addresses: vec![],
                notes: None,
                referred_by: None,
                date_of_birth: None,
                sex: None,
                gender: None,
                pronouns: None,
                occupation: None,
                language: None,
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
                kind: ClientKind::Individual,
                contact_name: None,
                tax_id: None,
                registration_number: None,
                emails: vec![],
                phones: vec![],
                addresses: vec![],
                notes: None,
                referred_by: None,
                date_of_birth: None,
                sex: None,
                gender: None,
                pronouns: None,
                occupation: None,
                language: None,
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
                kind: ClientKind::Individual,
                contact_name: None,
                tax_id: None,
                registration_number: None,
                emails: vec![],
                phones: vec![],
                addresses: vec![],
                notes: None,
                referred_by: Some(c.id),
                date_of_birth: None,
                sex: None,
                gender: None,
                pronouns: None,
                occupation: None,
                language: None,
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
        assert!(stored.is_archived());
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
        assert!(!stored.is_archived());
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
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].name, "Beta");

        let all = ListClients::new(repo.clone())
            .execute(ListClientsQuery {
                include_archived: true,
                search: None,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(all.data.len(), 2);
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
                include_archived: false,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].name, "Acme Corp");
    }

    #[test]
    fn get_client_detail_returns_not_found() {
        let repo = make_repo();
        let err = GetClientDetail::new(repo)
            .execute(ClientId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn list_attribute_values_dedupes_and_sorts_case_insensitively() {
        let repo = make_repo();
        // Two clients with overlapping pronouns and one new gender value.
        {
            let mut g = repo.inner.lock();
            let mut a = Client::create(
                NewClient {
                    name: "A".into(),
                    pronouns: Some("she/her".into()),
                    gender: Some("woman".into()),
                    ..Default::default()
                },
                Utc::now(),
            )
            .unwrap();
            a.occupation = Some("Architect".into());
            g.insert(a.id, a);

            let mut b = Client::create(
                NewClient {
                    name: "B".into(),
                    pronouns: Some("she/her".into()),
                    gender: Some("man".into()),
                    ..Default::default()
                },
                Utc::now(),
            )
            .unwrap();
            b.occupation = Some("Architect".into());
            g.insert(b.id, b);
        }
        let values = ListClientAttributeValues::new(repo).execute().unwrap();
        assert_eq!(values.pronouns, vec!["she/her"]);
        assert_eq!(values.gender, vec!["man", "woman"]);
        assert_eq!(values.occupation, vec!["Architect"]);
    }
}
