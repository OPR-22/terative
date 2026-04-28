use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContactEntryId(pub Uuid);

impl ContactEntryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContactEntryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContactEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactEntry {
    pub id: ContactEntryId,
    pub value: String,
    pub label: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NewContactEntry {
    pub value: String,
    pub label: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub emails: Vec<ContactEntry>,
    pub phones: Vec<ContactEntry>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<ClientId>,
    pub date_of_birth: Option<NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    /// Preferred contact language as an ISO 639-1 code (e.g. "fr", "en",
    /// "nl"). Free-form to accommodate languages outside the UI's two
    /// supported locales.
    pub language: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClientError {
    #[error("client name cannot be empty")]
    EmptyName,
    #[error("contact entry value cannot be empty")]
    EmptyContactValue,
    #[error("a client cannot refer itself")]
    SelfReferral,
    #[error("date of birth cannot be in the future")]
    FutureDateOfBirth,
}

#[derive(Debug, Clone, Default)]
pub struct NewClient {
    pub name: String,
    pub emails: Vec<NewContactEntry>,
    pub phones: Vec<NewContactEntry>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub referred_by: Option<ClientId>,
    pub date_of_birth: Option<NaiveDate>,
    pub sex: Option<String>,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub occupation: Option<String>,
    pub language: Option<String>,
}

impl Client {
    pub fn create(input: NewClient, now: DateTime<Utc>) -> Result<Self, ClientError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ClientError::EmptyName);
        }
        let emails = sanitize_contacts(input.emails)?;
        let phones = sanitize_contacts(input.phones)?;
        let date_of_birth = validate_dob(input.date_of_birth, now)?;
        Ok(Self {
            id: ClientId::new(),
            name,
            emails,
            phones,
            address: input.address.and_then(non_empty),
            notes: input.notes.and_then(non_empty),
            referred_by: input.referred_by,
            date_of_birth,
            sex: input.sex.and_then(non_empty),
            gender: input.gender.and_then(non_empty),
            pronouns: input.pronouns.and_then(non_empty),
            occupation: input.occupation.and_then(non_empty),
            language: input.language.and_then(non_empty),
            active: true,
            created_at: now,
        })
    }

    pub fn replace_emails(&mut self, new_emails: Vec<NewContactEntry>) -> Result<(), ClientError> {
        self.emails = sanitize_contacts(new_emails)?;
        Ok(())
    }

    pub fn replace_phones(&mut self, new_phones: Vec<NewContactEntry>) -> Result<(), ClientError> {
        self.phones = sanitize_contacts(new_phones)?;
        Ok(())
    }

    pub fn set_referred_by(&mut self, referrer: Option<ClientId>) -> Result<(), ClientError> {
        if referrer == Some(self.id) {
            return Err(ClientError::SelfReferral);
        }
        self.referred_by = referrer;
        Ok(())
    }

    pub fn default_email(&self) -> Option<&str> {
        default_value(&self.emails)
    }

    pub fn default_phone(&self) -> Option<&str> {
        default_value(&self.phones)
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn reactivate(&mut self) {
        self.active = true;
    }
}

/// Trims values, drops empty ones, and normalizes the default flag so that
/// exactly one entry is marked default when the list is non-empty.
fn sanitize_contacts(input: Vec<NewContactEntry>) -> Result<Vec<ContactEntry>, ClientError> {
    let mut out: Vec<ContactEntry> = Vec::with_capacity(input.len());
    for entry in input {
        let value = entry.value.trim().to_string();
        if value.is_empty() {
            return Err(ClientError::EmptyContactValue);
        }
        out.push(ContactEntry {
            id: ContactEntryId::new(),
            value,
            label: entry.label.and_then(non_empty),
            is_default: entry.is_default,
        });
    }
    if out.is_empty() {
        return Ok(out);
    }
    // At most one default; if none flagged, the first entry becomes default.
    let mut seen_default = false;
    for e in out.iter_mut() {
        if e.is_default && !seen_default {
            seen_default = true;
        } else {
            e.is_default = false;
        }
    }
    if !seen_default {
        out[0].is_default = true;
    }
    Ok(out)
}

fn default_value(list: &[ContactEntry]) -> Option<&str> {
    list.iter()
        .find(|e| e.is_default)
        .or_else(|| list.first())
        .map(|e| e.value.as_str())
}

fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Rejects a DOB strictly after `now`'s date. A DOB equal to today is allowed
/// (newborns happen). No lower bound — the application doesn't care if the
/// user types a clearly impossible date like 1700-01-01; that's a UI concern.
fn validate_dob(
    dob: Option<NaiveDate>,
    now: DateTime<Utc>,
) -> Result<Option<NaiveDate>, ClientError> {
    match dob {
        Some(d) if d > now.date_naive() => Err(ClientError::FutureDateOfBirth),
        other => Ok(other),
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

    fn new_email(value: &str, is_default: bool) -> NewContactEntry {
        NewContactEntry {
            value: value.into(),
            label: None,
            is_default,
        }
    }

    #[test]
    fn create_client_with_valid_name() {
        let c = Client::create(
            NewClient {
                name: "Acme Corp".into(),
                emails: vec![new_email("billing@acme.example", true)],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.name, "Acme Corp");
        assert_eq!(c.default_email(), Some("billing@acme.example"));
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
    fn create_client_rejects_empty_contact_value() {
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                emails: vec![new_email("  ", false)],
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::EmptyContactValue);
    }

    #[test]
    fn first_entry_becomes_default_if_none_flagged() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                emails: vec![
                    new_email("a@x.com", false),
                    new_email("b@x.com", false),
                ],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.emails.len(), 2);
        assert!(c.emails[0].is_default);
        assert!(!c.emails[1].is_default);
        assert_eq!(c.default_email(), Some("a@x.com"));
    }

    #[test]
    fn only_first_flagged_entry_is_kept_as_default() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                emails: vec![
                    new_email("a@x.com", false),
                    new_email("b@x.com", true),
                    new_email("c@x.com", true),
                ],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert!(!c.emails[0].is_default);
        assert!(c.emails[1].is_default);
        assert!(!c.emails[2].is_default);
        assert_eq!(c.default_email(), Some("b@x.com"));
    }

    #[test]
    fn empty_contact_lists_have_no_default() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert!(c.emails.is_empty());
        assert!(c.phones.is_empty());
        assert_eq!(c.default_email(), None);
        assert_eq!(c.default_phone(), None);
    }

    #[test]
    fn set_referred_by_rejects_self_reference() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        let own_id = c.id;
        let err = c.set_referred_by(Some(own_id)).unwrap_err();
        assert_eq!(err, ClientError::SelfReferral);
    }

    #[test]
    fn set_referred_by_accepts_other_client() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        let other = ClientId::new();
        c.set_referred_by(Some(other)).unwrap();
        assert_eq!(c.referred_by, Some(other));
        c.set_referred_by(None).unwrap();
        assert_eq!(c.referred_by, None);
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

    #[test]
    fn reactivate_restores_active_flag() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        c.deactivate();
        c.reactivate();
        assert!(c.active);
    }

    #[test]
    fn create_accepts_past_date_of_birth() {
        let dob = NaiveDate::from_ymd_opt(1990, 5, 14).unwrap();
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                date_of_birth: Some(dob),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.date_of_birth, Some(dob));
    }

    #[test]
    fn create_accepts_today_as_date_of_birth() {
        let today = now().date_naive();
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                date_of_birth: Some(today),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.date_of_birth, Some(today));
    }

    #[test]
    fn create_rejects_future_date_of_birth() {
        let future = now().date_naive().succ_opt().unwrap();
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                date_of_birth: Some(future),
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::FutureDateOfBirth);
    }

    #[test]
    fn create_normalizes_pronouns_occupation_language() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                pronouns: Some("  she/her  ".into()),
                occupation: Some("  Architect  ".into()),
                language: Some("  fr  ".into()),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.pronouns.as_deref(), Some("she/her"));
        assert_eq!(c.occupation.as_deref(), Some("Architect"));
        assert_eq!(c.language.as_deref(), Some("fr"));
    }

    #[test]
    fn create_drops_empty_pronouns_occupation_language() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                pronouns: Some("   ".into()),
                occupation: Some("".into()),
                language: Some("  ".into()),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert!(c.pronouns.is_none());
        assert!(c.occupation.is_none());
        assert!(c.language.is_none());
    }
}
