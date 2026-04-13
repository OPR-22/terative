use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClientError {
    #[error("client name cannot be empty")]
    EmptyName,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewClient {
    pub name: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl Client {
    pub fn create(input: NewClient, now: DateTime<Utc>) -> Result<Self, ClientError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ClientError::EmptyName);
        }
        Ok(Self {
            id: ClientId::new(),
            name,
            email: input.email.and_then(non_empty),
            address: input.address.and_then(non_empty),
            phone: input.phone.and_then(non_empty),
            notes: input.notes.and_then(non_empty),
            active: true,
            created_at: now,
        })
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-04-14T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn create_client_with_valid_name() {
        let c = Client::create(
            NewClient {
                name: "Acme Corp".into(),
                email: Some("billing@acme.example".into()),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.name, "Acme Corp");
        assert_eq!(c.email.as_deref(), Some("billing@acme.example"));
        assert!(c.active);
        assert_eq!(c.created_at, now());
    }

    #[test]
    fn create_client_trims_whitespace_name() {
        let c = Client::create(
            NewClient {
                name: "  Acme  ".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.name, "Acme");
    }

    #[test]
    fn create_client_rejects_empty_name() {
        let err = Client::create(NewClient::default(), now()).unwrap_err();
        assert_eq!(err, ClientError::EmptyName);
    }

    #[test]
    fn create_client_rejects_whitespace_only_name() {
        let err = Client::create(
            NewClient {
                name: "   ".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::EmptyName);
    }

    #[test]
    fn create_client_normalizes_empty_optional_fields_to_none() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                email: Some("".into()),
                address: Some("  ".into()),
                phone: Some("555-0100".into()),
                notes: None,
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.email, None);
        assert_eq!(c.address, None);
        assert_eq!(c.phone.as_deref(), Some("555-0100"));
        assert_eq!(c.notes, None);
    }

    #[test]
    fn deactivate_flips_active_flag() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert!(c.active);
        c.deactivate();
        assert!(!c.active);
    }
}
