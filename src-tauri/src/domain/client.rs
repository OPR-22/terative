use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::aggregate_root::AggregateRoot;
use crate::domain::events::client_events::{ClientArchived, ClientCreated, ClientUnarchived};
use crate::domain::events::EventBuffer;
use crate::domain::field_change::FieldChange;
use crate::domain::money::Currency;

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

/// Whether this client is a natural person or a legal entity. Drives UI
/// rendering (which fields to show); the domain layer stores Individual-
/// only and Company-only fields side-by-side and doesn't reject mixed
/// usage — sole traders frequently fill both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKind {
    Individual,
    Company,
}

impl ClientKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ClientKind::Individual => "Individual",
            ClientKind::Company => "Company",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Individual" => Some(Self::Individual),
            "Company" => Some(Self::Company),
            _ => None,
        }
    }
}

impl Default for ClientKind {
    fn default() -> Self {
        Self::Individual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientAddressId(pub Uuid);

impl ClientAddressId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClientAddressId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientAddressId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A single postal address, structured. `is_billing` / `is_shipping` flag
/// what the address is used for — at least one must be true. A client
/// has at most one billing address and at most one shipping address;
/// the same row may carry both flags (the common case for individuals).
/// The DB enforces this with partial unique indexes on each flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAddress {
    pub id: ClientAddressId,
    /// Free-form label. Optional, kept mostly for legacy data; with at
    /// most one billing + one shipping per client the label is rarely
    /// load-bearing for disambiguation.
    pub label: Option<String>,
    /// Line 1 — building/street number plus street name.
    pub street: String,
    /// Line 2 — apartment, suite, building, floor, "c/o", etc.
    pub apt_suite: Option<String>,
    pub city: String,
    /// Optional — many countries don't use a state/province subdivision.
    pub state_province: Option<String>,
    pub postal_code: String,
    /// Free-form. ISO 3166-1 alpha-2 ("FR", "BE") recommended.
    pub country: String,
    pub is_billing: bool,
    pub is_shipping: bool,
}

impl ClientAddress {
    /// Joins the structured fields into a multi-line representation suitable
    /// for invoice/PDF rendering. Skips empty optional lines so we don't
    /// emit blank ones.
    pub fn formatted(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(4);
        lines.push(self.street.clone());
        if let Some(apt) = self.apt_suite.as_deref() {
            if !apt.trim().is_empty() {
                lines.push(apt.trim().to_string());
            }
        }
        // City line: "Postal City" or "Postal City, State".
        let mut city_line = format!("{} {}", self.postal_code, self.city);
        if let Some(state) = self.state_province.as_deref() {
            if !state.trim().is_empty() {
                city_line.push_str(", ");
                city_line.push_str(state.trim());
            }
        }
        lines.push(city_line);
        lines.push(self.country.clone());
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Default)]
pub struct NewClientAddress {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub kind: ClientKind,
    /// Display name. For Individuals: the person's full name. For Companies:
    /// the trading / public-facing name.
    pub name: String,
    /// Optional human contact at a company (e.g. accounts payable contact).
    /// The UI only shows / writes this for Companies.
    pub contact_name: Option<String>,
    /// VAT number for B2B invoicing.
    pub tax_id: Option<String>,
    /// Companies-house / SIREN / KBIS registration number.
    pub registration_number: Option<String>,
    pub emails: Vec<ContactEntry>,
    pub phones: Vec<ContactEntry>,
    pub addresses: Vec<ClientAddress>,
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
    /// Pre-fills the currency on new invoices for this client. Does not
    /// restrict — the user may invoice in any currency regardless of this
    /// preference. Defaults to the org's currency at creation time.
    pub default_currency: Currency,
    /// `None` = active client. `Some(timestamp)` = archived client; the
    /// timestamp records when the user clicked "archive". Lets the UI
    /// sort archived items by when they were retired and gives an audit
    /// trail without a separate event log.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Domain events buffered by mutating methods, drained by the use case
    /// after persistence. Not persisted; a row loaded from SQLite always has
    /// this empty. See [`EventBuffer`] for why this keeps the `derive`s intact.
    pub pending_events: EventBuffer,
}

impl AggregateRoot for Client {
    fn pending_events_mut(&mut self) -> &mut EventBuffer {
        &mut self.pending_events
    }

    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        // `archived_at` and `created_at` are intentionally omitted: archive
        // / unarchive have their own dedicated events, and `created_at` is
        // immutable. `id` and the event buffer are not user-visible state.
        [
            FieldChange::scalar("name", &before.name, &self.name),
            FieldChange::scalar("kind", before.kind.as_str(), self.kind.as_str()),
            FieldChange::opt("contact_name", &before.contact_name, &self.contact_name),
            FieldChange::opt("tax_id", &before.tax_id, &self.tax_id),
            FieldChange::opt(
                "registration_number",
                &before.registration_number,
                &self.registration_number,
            ),
            FieldChange::opt("notes", &before.notes, &self.notes),
            FieldChange::opt("referred_by", &before.referred_by, &self.referred_by),
            FieldChange::opt("date_of_birth", &before.date_of_birth, &self.date_of_birth),
            FieldChange::opt("sex", &before.sex, &self.sex),
            FieldChange::opt("gender", &before.gender, &self.gender),
            FieldChange::opt("pronouns", &before.pronouns, &self.pronouns),
            FieldChange::opt("occupation", &before.occupation, &self.occupation),
            FieldChange::opt("language", &before.language, &self.language),
            FieldChange::scalar(
                "default_currency",
                before.default_currency.code(),
                self.default_currency.code(),
            ),
            // Contact entries and addresses have no stable identity (the user
            // can edit any of them), so a count-only summary is the honest
            // v1 — element-level diffs would require synthetic IDs the
            // domain doesn't model today.
            FieldChange::collection("emails", &before.emails, &self.emails),
            FieldChange::collection("phones", &before.phones, &self.phones),
            FieldChange::collection("addresses", &before.addresses, &self.addresses),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClientError {
    #[error("client name cannot be empty")]
    EmptyName,
    #[error("contact entry value cannot be empty")]
    EmptyContactValue,
    #[error("address street cannot be empty")]
    EmptyAddressStreet,
    #[error("address city cannot be empty")]
    EmptyAddressCity,
    #[error("address postal code cannot be empty")]
    EmptyAddressPostalCode,
    #[error("address country cannot be empty")]
    EmptyAddressCountry,
    #[error("only one address can be the active billing address")]
    DuplicateBillingAddress,
    #[error("only one address can be the active shipping address")]
    DuplicateShippingAddress,
    #[error("a client cannot refer itself")]
    SelfReferral,
    #[error("date of birth cannot be in the future")]
    FutureDateOfBirth,
}

#[derive(Debug, Clone, Default)]
pub struct NewClient {
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
    /// The currency to use as the default on this client's new invoices.
    /// Callers (the use case layer) pass the org's currency here when the
    /// user hasn't explicitly picked one.
    pub default_currency: Currency,
}

impl Client {
    pub fn create(input: NewClient, now: DateTime<Utc>) -> Result<Self, ClientError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ClientError::EmptyName);
        }
        let emails = sanitize_contacts(input.emails)?;
        let phones = sanitize_contacts(input.phones)?;
        let addresses = sanitize_addresses(input.addresses)?;
        let date_of_birth = validate_dob(input.date_of_birth, now)?;
        let mut client = Self {
            id: ClientId::new(),
            kind: input.kind,
            name,
            contact_name: input.contact_name.and_then(non_empty),
            tax_id: input.tax_id.and_then(non_empty),
            registration_number: input.registration_number.and_then(non_empty),
            emails,
            phones,
            addresses,
            notes: input.notes.and_then(non_empty),
            referred_by: input.referred_by,
            date_of_birth,
            sex: input.sex.and_then(non_empty),
            gender: input.gender.and_then(non_empty),
            pronouns: input.pronouns.and_then(non_empty),
            occupation: input.occupation.and_then(non_empty),
            language: input.language.and_then(non_empty),
            default_currency: input.default_currency,
            archived_at: None,
            created_at: now,
            pending_events: EventBuffer::default(),
        };
        client.apply(ClientCreated {
            id: client.id,
            name: client.name.clone(),
            at: now,
        });
        Ok(client)
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    pub fn replace_emails(&mut self, new_emails: Vec<NewContactEntry>) -> Result<(), ClientError> {
        self.emails = sanitize_contacts(new_emails)?;
        Ok(())
    }

    pub fn replace_phones(&mut self, new_phones: Vec<NewContactEntry>) -> Result<(), ClientError> {
        self.phones = sanitize_contacts(new_phones)?;
        Ok(())
    }

    pub fn replace_addresses(
        &mut self,
        new_addresses: Vec<NewClientAddress>,
    ) -> Result<(), ClientError> {
        self.addresses = sanitize_addresses(new_addresses)?;
        Ok(())
    }

    /// The client's billing address, if any. By construction (validated by
    /// `sanitize_addresses` and enforced at the DB layer) there is at
    /// most one billing-flagged row per client.
    pub fn billing_address(&self) -> Option<&ClientAddress> {
        self.addresses.iter().find(|a| a.is_billing)
    }

    /// The client's shipping address, if any. May be the same row as the
    /// billing address when that row carries both flags.
    pub fn shipping_address(&self) -> Option<&ClientAddress> {
        self.addresses.iter().find(|a| a.is_shipping)
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

    pub fn archive(&mut self, now: DateTime<Utc>) {
        self.archived_at = Some(now);
        self.apply(ClientArchived {
            id: self.id,
            name: self.name.clone(),
            at: now,
        });
    }

    pub fn unarchive(&mut self, now: DateTime<Utc>) {
        self.archived_at = None;
        self.apply(ClientUnarchived {
            id: self.id,
            name: self.name.clone(),
            at: now,
        });
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

/// Trims structured fields, validates the required ones, and rejects
/// having more than one active billing or active shipping row. Addresses
/// with neither flag set are valid — they're stored-but-not-currently-
/// active addresses. The DB partial unique indexes are the ultimate
/// enforcer; this check surfaces a clean domain error before the round-
/// trip.
fn sanitize_addresses(
    input: Vec<NewClientAddress>,
) -> Result<Vec<ClientAddress>, ClientError> {
    let mut out: Vec<ClientAddress> = Vec::with_capacity(input.len());
    let mut billing_seen = false;
    let mut shipping_seen = false;
    for a in input {
        if a.is_billing {
            if billing_seen {
                return Err(ClientError::DuplicateBillingAddress);
            }
            billing_seen = true;
        }
        if a.is_shipping {
            if shipping_seen {
                return Err(ClientError::DuplicateShippingAddress);
            }
            shipping_seen = true;
        }
        let street = a.street.trim().to_string();
        if street.is_empty() {
            return Err(ClientError::EmptyAddressStreet);
        }
        let city = a.city.trim().to_string();
        if city.is_empty() {
            return Err(ClientError::EmptyAddressCity);
        }
        let postal_code = a.postal_code.trim().to_string();
        if postal_code.is_empty() {
            return Err(ClientError::EmptyAddressPostalCode);
        }
        let country = a.country.trim().to_string();
        if country.is_empty() {
            return Err(ClientError::EmptyAddressCountry);
        }
        out.push(ClientAddress {
            id: ClientAddressId::new(),
            label: a.label.and_then(non_empty),
            street,
            apt_suite: a.apt_suite.and_then(non_empty),
            city,
            state_province: a.state_province.and_then(non_empty),
            postal_code,
            country,
            is_billing: a.is_billing,
            is_shipping: a.is_shipping,
        });
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
        assert!(!c.is_archived());
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
    fn archive_stamps_timestamp() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert!(!c.is_archived());
        c.archive(now());
        assert_eq!(c.archived_at, Some(now()));
        assert!(c.is_archived());
    }

    #[test]
    fn unarchive_clears_timestamp() {
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        c.archive(now());
        c.unarchive(now());
        assert!(c.archived_at.is_none());
        assert!(!c.is_archived());
    }

    // === Domain event emission ===

    fn acme() -> Client {
        Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap()
    }

    #[test]
    fn create_buffers_client_created_event() {
        let mut c = acme();
        let events = c.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<ClientCreated>()
            .expect("ClientCreated");
        assert_eq!(ev.id, c.id);
        assert_eq!(ev.name, "Acme");
    }

    #[test]
    fn archive_buffers_client_archived_event() {
        let mut c = acme();
        let _ = c.take_events(); // discard the created event
        c.archive(now());
        let events = c.take_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].downcast_ref::<ClientArchived>().is_some());
    }

    #[test]
    fn unarchive_buffers_client_unarchived_event() {
        let mut c = acme();
        c.archive(now());
        let _ = c.take_events(); // discard created + archived
        c.unarchive(now());
        let events = c.take_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].downcast_ref::<ClientUnarchived>().is_some());
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
    fn create_records_explicit_default_currency() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                default_currency: Currency::Usd,
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.default_currency, Currency::Usd);
    }

    #[test]
    fn create_falls_back_to_default_currency_when_unspecified() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        // NewClient::default() picks Currency::default() — i.e. EUR. This is
        // the "caller didn't care" case; the use case layer is expected to
        // override with the org's actual currency.
        assert_eq!(c.default_currency, Currency::Eur);
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

    fn new_address(street: &str, billing: bool, shipping: bool) -> NewClientAddress {
        NewClientAddress {
            label: None,
            street: street.into(),
            apt_suite: None,
            city: "Brussels".into(),
            state_province: None,
            postal_code: "1000".into(),
            country: "BE".into(),
            is_billing: billing,
            is_shipping: shipping,
        }
    }

    #[test]
    fn create_with_addresses_exposes_billing_and_shipping() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![
                    new_address("1 HQ St", true, false),
                    new_address("2 Warehouse Way", false, true),
                ],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.billing_address().unwrap().street, "1 HQ St");
        assert_eq!(c.shipping_address().unwrap().street, "2 Warehouse Way");
    }

    #[test]
    fn create_with_combined_address_returns_same_row_for_both() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![new_address("1 Office Rd", true, true)],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.billing_address().unwrap().street, "1 Office Rd");
        assert_eq!(c.shipping_address().unwrap().street, "1 Office Rd");
        // Same row, not a duplicate.
        assert_eq!(c.addresses.len(), 1);
    }

    #[test]
    fn create_rejects_duplicate_billing() {
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![
                    new_address("1 First", true, false),
                    new_address("2 Second", true, false),
                ],
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::DuplicateBillingAddress);
    }

    #[test]
    fn create_rejects_duplicate_shipping() {
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![
                    new_address("1 First", false, true),
                    new_address("2 Second", false, true),
                ],
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::DuplicateShippingAddress);
    }

    #[test]
    fn create_accepts_address_with_no_active_role() {
        // An address with neither flag is valid — it's stored on file
        // but not currently the active billing or shipping address.
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![new_address("inactive", false, false)],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.addresses.len(), 1);
        assert!(c.billing_address().is_none());
        assert!(c.shipping_address().is_none());
    }

    #[test]
    fn create_rejects_empty_required_address_fields() {
        let mut a = new_address("1 Way", true, false);
        a.street = "  ".into();
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![a],
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::EmptyAddressStreet);

        let mut a = new_address("1 Way", true, false);
        a.country = "".into();
        let err = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![a],
                ..Default::default()
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, ClientError::EmptyAddressCountry);
    }

    #[test]
    fn formatted_address_skips_empty_optional_lines() {
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![new_address("1 Way", true, true)],
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        let f = c.addresses[0].formatted();
        assert!(f.contains("1 Way"));
        assert!(f.contains("1000 Brussels"));
        assert!(f.ends_with("BE"));
        // No blank middle line.
        assert!(!f.contains("\n\n"));
    }

    #[test]
    fn create_company_kind_with_tax_fields() {
        let c = Client::create(
            NewClient {
                kind: ClientKind::Company,
                name: "Acme SARL".into(),
                contact_name: Some("Marie Dupont".into()),
                tax_id: Some("FR12345678901".into()),
                registration_number: Some("123 456 789".into()),
                ..Default::default()
            },
            now(),
        )
        .unwrap();
        assert_eq!(c.kind, ClientKind::Company);
        assert_eq!(c.tax_id.as_deref(), Some("FR12345678901"));
        assert_eq!(c.registration_number.as_deref(), Some("123 456 789"));
        assert_eq!(c.contact_name.as_deref(), Some("Marie Dupont"));
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

    // === diff_against ===

    #[test]
    fn diff_against_identical_returns_empty() {
        let a = acme();
        let b = a.clone();
        assert!(b.diff_against(&a).is_empty());
    }

    #[test]
    fn diff_against_reports_scalars_opts_and_collections() {
        let before = acme();
        let mut after = before.clone();

        after.name = "Acme Corp".into();                      // scalar
        after.kind = ClientKind::Company;                     // scalar (enum)
        after.tax_id = Some("BE0123456".into());              // opt None → Some
        after.notes = Some("VIP".into());                     // opt None → Some
        after.default_currency = Currency::Usd;               // scalar (enum code)
        after
            .replace_emails(vec![NewContactEntry {
                value: "billing@acme.example".into(),
                label: None,
                is_default: true,
            }])
            .unwrap();                                        // collection 0 → 1

        let changes = after.diff_against(&before);
        let fields: Vec<&str> = changes.iter().map(FieldChange::field).collect();

        assert!(fields.contains(&"name"));
        assert!(fields.contains(&"kind"));
        assert!(fields.contains(&"tax_id"));
        assert!(fields.contains(&"notes"));
        assert!(fields.contains(&"default_currency"));
        assert!(fields.contains(&"emails"));
        // Untouched fields must not appear:
        assert!(!fields.contains(&"phones"));
        assert!(!fields.contains(&"addresses"));
        assert!(!fields.contains(&"date_of_birth"));
        assert!(!fields.contains(&"language"));

        // Email collection diff: count went 0 → 1.
        let emails = changes.iter().find(|c| c.field() == "emails").unwrap();
        match emails {
            FieldChange::Collection { from_count, to_count, .. } => {
                assert_eq!(*from_count, 0);
                assert_eq!(*to_count, 1);
            }
            _ => panic!("expected Collection for emails"),
        }
    }
}
