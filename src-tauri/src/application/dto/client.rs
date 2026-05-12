use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use crate::application::client_usecases::UpdateClientInput;
use crate::application::ports::{ClientAttributeValues, ListClientsQuery};
use crate::domain::client::{
    Client, ClientAddress, ClientId, ClientKind, ContactEntry, NewClient, NewClientAddress,
    NewContactEntry,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum ClientKindDto {
    Individual,
    Company,
}

impl Default for ClientKindDto {
    fn default() -> Self {
        Self::Individual
    }
}

impl From<ClientKind> for ClientKindDto {
    fn from(k: ClientKind) -> Self {
        match k {
            ClientKind::Individual => Self::Individual,
            ClientKind::Company => Self::Company,
        }
    }
}

impl From<ClientKindDto> for ClientKind {
    fn from(k: ClientKindDto) -> Self {
        match k {
            ClientKindDto::Individual => Self::Individual,
            ClientKindDto::Company => Self::Company,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ClientAddressDto {
    pub id: Option<Uuid>,
    pub label: Option<String>,
    pub street: String,
    pub apt_suite: Option<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub is_billing: bool,
    pub is_shipping: bool,
}

impl From<&ClientAddress> for ClientAddressDto {
    fn from(a: &ClientAddress) -> Self {
        Self {
            id: Some(a.id.0),
            label: a.label.clone(),
            street: a.street.clone(),
            apt_suite: a.apt_suite.clone(),
            city: a.city.clone(),
            state_province: a.state_province.clone(),
            postal_code: a.postal_code.clone(),
            country: a.country.clone(),
            is_billing: a.is_billing,
            is_shipping: a.is_shipping,
        }
    }
}

impl From<ClientAddressDto> for NewClientAddress {
    fn from(dto: ClientAddressDto) -> Self {
        NewClientAddress {
            label: dto.label,
            street: dto.street,
            apt_suite: dto.apt_suite,
            city: dto.city,
            state_province: dto.state_province,
            postal_code: dto.postal_code,
            country: dto.country,
            is_billing: dto.is_billing,
            is_shipping: dto.is_shipping,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ClientDto {
    pub id: Uuid,
    pub kind: ClientKindDto,
    pub name: String,
    pub contact_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub addresses: Vec<ClientAddressDto>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
    /// ISO 4217 code (e.g. "EUR"). Pre-fills the currency on new invoices
    /// for this client.
    pub default_currency: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<&Client> for ClientDto {
    fn from(c: &Client) -> Self {
        Self {
            id: c.id.0,
            kind: c.kind.into(),
            name: c.name.clone(),
            contact_name: c.contact_name.clone(),
            tax_id: c.tax_id.clone(),
            registration_number: c.registration_number.clone(),
            emails: c.emails.iter().map(Into::into).collect(),
            phones: c.phones.iter().map(Into::into).collect(),
            addresses: c.addresses.iter().map(Into::into).collect(),
            notes: c.notes.clone(),
            referred_by: c.referred_by.map(|r| r.0),
            date_of_birth: c.date_of_birth,
            sex: c.sex.clone(),
            gender: c.gender.clone(),
            pronouns: c.pronouns.clone(),
            occupation: c.occupation.clone(),
            language: c.language.clone(),
            default_currency: c.default_currency.code().to_string(),
            archived_at: c.archived_at,
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
    #[serde(default)]
    pub kind: ClientKindDto,
    pub name: String,
    pub contact_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub addresses: Vec<ClientAddressDto>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
    /// ISO 4217 code. `None` lets the use case fall back to the org's
    /// currency at creation time.
    #[serde(default)]
    pub default_currency: Option<String>,
}

impl TryFrom<NewClientDto> for NewClient {
    type Error = DtoConvertError;
    fn try_from(dto: NewClientDto) -> Result<Self, Self::Error> {
        let default_currency = match dto.default_currency.as_deref() {
            Some(code) => crate::domain::money::Currency::new(code)
                .map_err(|_| DtoConvertError::InvalidCurrency(code.to_string()))?,
            None => crate::domain::money::Currency::Eur,
        };
        Ok(NewClient {
            kind: dto.kind.into(),
            name: dto.name,
            contact_name: dto.contact_name,
            tax_id: dto.tax_id,
            registration_number: dto.registration_number,
            emails: dto.emails.into_iter().map(Into::into).collect(),
            phones: dto.phones.into_iter().map(Into::into).collect(),
            addresses: dto.addresses.into_iter().map(Into::into).collect(),
            notes: dto.notes,
            referred_by: dto.referred_by.map(ClientId),
            date_of_birth: dto.date_of_birth,
            sex: dto.sex,
            gender: dto.gender,
            pronouns: dto.pronouns,
            occupation: dto.occupation,
            language: dto.language,
            default_currency,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateClientDto {
    pub id: Uuid,
    #[serde(default)]
    pub kind: ClientKindDto,
    pub name: String,
    pub contact_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub emails: Vec<ContactEntryDto>,
    pub phones: Vec<ContactEntryDto>,
    pub addresses: Vec<ClientAddressDto>,
    pub notes: Option<String>,
    pub referred_by: Option<Uuid>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
    pub default_currency: String,
}

impl TryFrom<UpdateClientDto> for UpdateClientInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdateClientDto) -> Result<Self, Self::Error> {
        let default_currency = crate::domain::money::Currency::new(&dto.default_currency)
            .map_err(|_| DtoConvertError::InvalidCurrency(dto.default_currency.clone()))?;
        Ok(UpdateClientInput {
            id: ClientId(dto.id),
            kind: dto.kind.into(),
            name: dto.name,
            contact_name: dto.contact_name,
            tax_id: dto.tax_id,
            registration_number: dto.registration_number,
            emails: dto.emails.into_iter().map(Into::into).collect(),
            phones: dto.phones.into_iter().map(Into::into).collect(),
            addresses: dto.addresses.into_iter().map(Into::into).collect(),
            notes: dto.notes,
            referred_by: dto.referred_by.map(ClientId),
            date_of_birth: dto.date_of_birth,
            sex: dto.sex,
            gender: dto.gender,
            pronouns: dto.pronouns,
            occupation: dto.occupation,
            language: dto.language,
            default_currency,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ListClientsQueryDto {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
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
            include_archived: dto.include_archived,
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
            kind: ClientKind::Individual,
            name: "Acme Corp".into(),
            contact_name: None,
            tax_id: None,
            registration_number: None,
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
            addresses: vec![],
            notes: None,
            referred_by: None,
            date_of_birth: None,
            sex: None,
            gender: None,
            pronouns: None,
            occupation: None,
            language: None,
            default_currency: crate::domain::money::Currency::Eur,
            archived_at: None,
            created_at: Utc.with_ymd_and_hms(2026, 4, 14, 9, 0, 0).unwrap(),
        }
    }

    #[test]
    fn client_to_dto_preserves_all_fields() {
        let client = sample_client();
        let dto: ClientDto = (&client).into();
        assert_eq!(dto.id, client.id.0);
        assert_eq!(dto.name, client.name);
        assert_eq!(dto.kind, ClientKindDto::Individual);
        assert_eq!(dto.emails.len(), 1);
        assert_eq!(dto.emails[0].value, "billing@acme.example");
        assert_eq!(dto.emails[0].label.as_deref(), Some("Billing"));
        assert!(dto.emails[0].is_default);
        assert_eq!(dto.phones.len(), 1);
        assert_eq!(dto.phones[0].value, "555-0100");
        assert!(dto.addresses.is_empty());
        assert_eq!(dto.notes, client.notes);
        assert_eq!(dto.referred_by, None);
        assert_eq!(dto.archived_at, client.archived_at);
        assert_eq!(dto.created_at, client.created_at);
    }

    #[test]
    fn new_client_dto_maps_into_domain_input() {
        let dto = NewClientDto {
            kind: ClientKindDto::Company,
            name: "Acme".into(),
            tax_id: Some("FR12345".into()),
            emails: vec![ContactEntryDto {
                id: None,
                value: "a@b.c".into(),
                label: None,
                is_default: true,
            }],
            ..Default::default()
        };
        let input: NewClient = dto.clone().try_into().unwrap();
        assert_eq!(input.name, dto.name);
        assert_eq!(input.kind, ClientKind::Company);
        assert_eq!(input.tax_id.as_deref(), Some("FR12345"));
        assert_eq!(input.emails.len(), 1);
        assert!(input.emails[0].is_default);
    }

    #[test]
    fn update_client_dto_maps_into_use_case_input() {
        let id = Uuid::new_v4();
        let referrer = Uuid::new_v4();
        let dto = UpdateClientDto {
            id,
            kind: ClientKindDto::Individual,
            name: "New".into(),
            contact_name: None,
            tax_id: None,
            registration_number: None,
            emails: vec![],
            phones: vec![],
            addresses: vec![],
            notes: Some("hi".into()),
            referred_by: Some(referrer),
            date_of_birth: None,
            sex: None,
            gender: None,
            pronouns: None,
            occupation: None,
            language: None,
            default_currency: "USD".into(),
        };
        let input: UpdateClientInput = dto.try_into().unwrap();
        assert_eq!(input.id.0, id);
        assert_eq!(input.name, "New");
        assert_eq!(input.notes.as_deref(), Some("hi"));
        assert_eq!(input.referred_by.map(|r| r.0), Some(referrer));
        assert_eq!(input.default_currency, crate::domain::money::Currency::Usd);
    }

    #[test]
    fn list_clients_query_dto_defaults() {
        let dto: ListClientsQueryDto =
            serde_json::from_str("{}").expect("empty object must deserialize");
        let q: ListClientsQuery = dto.into();
        assert_eq!(q.search, None);
        assert!(!q.include_archived);
    }

    #[test]
    fn list_clients_query_dto_partial_deserialize() {
        let dto: ListClientsQueryDto = serde_json::from_str(r#"{"search":"acme"}"#).unwrap();
        let q: ListClientsQuery = dto.into();
        assert_eq!(q.search.as_deref(), Some("acme"));
        assert!(!q.include_archived);
    }
}
