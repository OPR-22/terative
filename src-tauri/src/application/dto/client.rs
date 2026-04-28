use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use crate::application::client_usecases::UpdateClientInput;
use crate::application::ports::{ClientAttributeValues, ListClientsQuery};
use crate::domain::client::{Client, ClientId, ContactEntry, NewClient, NewContactEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ContactEntryDto {
    pub id: Option<Uuid>,
    pub value: String,
    pub label: Option<String>,
    pub is_default: bool,
}

impl From<&ContactEntry> for ContactEntryDto {
    fn from(e: &ContactEntry) -> Self {
        Self {
            id: Some(e.id.0),
            value: e.value.clone(),
            label: e.label.clone(),
            is_default: e.is_default,
        }
    }
}

impl From<ContactEntryDto> for NewContactEntry {
    fn from(dto: ContactEntryDto) -> Self {
        NewContactEntry {
            value: dto.value,
            label: dto.label,
            is_default: dto.is_default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ClientDto {
    pub id: Uuid,
    pub name: String,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&Client> for ClientDto {
    fn from(c: &Client) -> Self {
        Self {
            id: c.id.0,
            name: c.name.clone(),
            emails: c.emails.iter().map(Into::into).collect(),
            phones: c.phones.iter().map(Into::into).collect(),
            address: c.address.clone(),
            notes: c.notes.clone(),
            referred_by: c.referred_by.map(|r| r.0),
            date_of_birth: c.date_of_birth,
            sex: c.sex.clone(),
            gender: c.gender.clone(),
            pronouns: c.pronouns.clone(),
            occupation: c.occupation.clone(),
            language: c.language.clone(),
            active: c.active,
            created_at: c.created_at,
        }
    }
}

impl From<Client> for ClientDto {
    fn from(c: Client) -> Self {
        (&c).into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct NewClientDto {
    pub name: String,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
}

impl From<NewClientDto> for NewClient {
    fn from(dto: NewClientDto) -> Self {
        NewClient {
            name: dto.name,
            emails: dto.emails.into_iter().map(Into::into).collect(),
            phones: dto.phones.into_iter().map(Into::into).collect(),
            address: dto.address,
            notes: dto.notes,
            referred_by: dto.referred_by.map(ClientId),
            date_of_birth: dto.date_of_birth,
            sex: dto.sex,
            gender: dto.gender,
            pronouns: dto.pronouns,
            occupation: dto.occupation,
            language: dto.language,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateClientDto {
    pub id: Uuid,
    pub name: String,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
}

impl From<UpdateClientDto> for UpdateClientInput {
    fn from(dto: UpdateClientDto) -> Self {
        UpdateClientInput {
            id: ClientId(dto.id),
            name: dto.name,
            emails: dto.emails.into_iter().map(Into::into).collect(),
            phones: dto.phones.into_iter().map(Into::into).collect(),
            address: dto.address,
            notes: dto.notes,
            referred_by: dto.referred_by.map(ClientId),
            date_of_birth: dto.date_of_birth,
            sex: dto.sex,
            gender: dto.gender,
            pronouns: dto.pronouns,
            occupation: dto.occupation,
            language: dto.language,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ListClientsQueryDto {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub include_inactive: bool,
    #[serde(default)]
    pub pagination: Option<super::PaginationParamsDto>,
}

impl ListClientsQueryDto {
    pub fn pagination_params(&self) -> crate::application::ports::PaginationParams {
        self.pagination.clone().into()
    }
}

impl From<ListClientsQueryDto> for ListClientsQuery {
    fn from(dto: ListClientsQueryDto) -> Self {
        ListClientsQuery {
            search: dto.search,
            include_inactive: dto.include_inactive,
            pagination: dto.pagination.into(),
        }
    }
}

pub fn uuid_to_client_id(id: Uuid) -> ClientId {
    ClientId(id)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ClientAttributeValuesDto {
    pub gender: Vec<String>,
    pub pronouns: Vec<String>,
    pub occupation: Vec<String>,
}

impl From<ClientAttributeValues> for ClientAttributeValuesDto {
    fn from(v: ClientAttributeValues) -> Self {
        Self {
            gender: v.gender,
            pronouns: v.pronouns,
            occupation: v.occupation,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn _unused_convert_error() -> DtoConvertError {
    DtoConvertError::InvalidUuid("placeholder".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::client::ContactEntryId;
    use chrono::TimeZone;

    fn sample_client() -> Client {
        Client {
            id: ClientId::new(),
            name: "Acme Corp".into(),
            emails: vec![ContactEntry {
                id: ContactEntryId::new(),
                value: "billing@acme.example".into(),
                label: Some("Billing".into()),
                is_default: true,
            }],
            phones: vec![ContactEntry {
                id: ContactEntryId::new(),
                value: "555-0100".into(),
                label: None,
                is_default: true,
            }],
            address: Some("1 Way".into()),
            notes: None,
            referred_by: None,
            date_of_birth: None,
            sex: None,
            gender: None,
            pronouns: None,
            occupation: None,
            language: None,
            active: true,
            created_at: Utc.with_ymd_and_hms(2026, 4, 14, 9, 0, 0).unwrap(),
        }
    }

    #[test]
    fn client_to_dto_preserves_all_fields() {
        let client = sample_client();
        let dto: ClientDto = (&client).into();
        assert_eq!(dto.id, client.id.0);
        assert_eq!(dto.name, client.name);
        assert_eq!(dto.emails.len(), 1);
        assert_eq!(dto.emails[0].value, "billing@acme.example");
        assert_eq!(dto.emails[0].label.as_deref(), Some("Billing"));
        assert!(dto.emails[0].is_default);
        assert_eq!(dto.phones.len(), 1);
        assert_eq!(dto.phones[0].value, "555-0100");
        assert_eq!(dto.address, client.address);
        assert_eq!(dto.notes, client.notes);
        assert_eq!(dto.referred_by, None);
        assert_eq!(dto.active, client.active);
        assert_eq!(dto.created_at, client.created_at);
    }

    #[test]
    fn new_client_dto_maps_into_domain_input() {
        let dto = NewClientDto {
            name: "Acme".into(),
            emails: vec![ContactEntryDto {
                id: None,
                value: "a@b.c".into(),
                label: None,
                is_default: true,
            }],
            phones: vec![],
            address: None,
            notes: None,
            referred_by: None,
            date_of_birth: None,
            sex: None,
            gender: None,
            pronouns: None,
            occupation: None,
            language: None,
        };
        let input: NewClient = dto.clone().into();
        assert_eq!(input.name, dto.name);
        assert_eq!(input.emails.len(), 1);
        assert_eq!(input.emails[0].value, "a@b.c");
        assert!(input.emails[0].is_default);
    }

    #[test]
    fn update_client_dto_maps_into_use_case_input() {
        let id = Uuid::new_v4();
        let referrer = Uuid::new_v4();
        let dto = UpdateClientDto {
            id,
            name: "New".into(),
            emails: vec![],
            phones: vec![],
            address: None,
            notes: Some("hi".into()),
            referred_by: Some(referrer),
            date_of_birth: None,
            sex: None,
            gender: None,
            pronouns: None,
            occupation: None,
            language: None,
        };
        let input: UpdateClientInput = dto.into();
        assert_eq!(input.id.0, id);
        assert_eq!(input.name, "New");
        assert_eq!(input.notes.as_deref(), Some("hi"));
        assert_eq!(input.referred_by.map(|r| r.0), Some(referrer));
    }

    #[test]
    fn list_clients_query_dto_defaults() {
        let dto: ListClientsQueryDto =
            serde_json::from_str("{}").expect("empty object must deserialize");
        let q: ListClientsQuery = dto.into();
        assert_eq!(q.search, None);
        assert!(!q.include_inactive);
    }

    #[test]
    fn list_clients_query_dto_partial_deserialize() {
        let dto: ListClientsQueryDto = serde_json::from_str(r#"{"search":"acme"}"#).unwrap();
        let q: ListClientsQuery = dto.into();
        assert_eq!(q.search.as_deref(), Some("acme"));
        assert!(!q.include_inactive);
    }
}
