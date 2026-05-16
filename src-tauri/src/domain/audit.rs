//! The [`Audit`] entity — one row in the audit log, a projection of a
//! single [`DomainEvent`](crate::domain::events::DomainEvent).
//!
//! Append-only: written by the `AuditProjector` handlers, never mutated.
//! This is a read-model entity, not an aggregate root — it emits no events of
//! its own.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AuditId(pub Uuid);

impl AuditId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AuditId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AuditId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// One projected domain event.
///
/// `entity_type` / `entity_id` identify the event's *subject*. `client_id` /
/// `invoice_id` are denormalised *scope pointers* so the per-client and
/// per-invoice views are single indexed queries — they may differ from the
/// subject (a payment's audit carries the payment id as `entity_id` but
/// still points at a `client_id`, and possibly an `invoice_id`, so it surfaces
/// on those views).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audit {
    pub id: AuditId,
    /// Dotted event identifier, e.g. `"invoice.finalized"` — taken verbatim
    /// from [`DomainEvent::event_name`](crate::domain::events::DomainEvent::event_name).
    pub event_type: String,
    /// `"invoice"` | `"client"` | `"payment"` | `"backup"`.
    pub entity_type: String,
    /// The subject's id (a UUID as text), or `None` for events with no single
    /// entity (e.g. backups).
    pub entity_id: Option<String>,
    /// Denormalised scope: which client this audit belongs to.
    pub client_id: Option<ClientId>,
    /// Denormalised scope: which invoice this audit belongs to.
    pub invoice_id: Option<InvoiceId>,
    /// Small JSON blob of event-type-specific fields the UI renders without
    /// re-fetching (invoice number, amount, client name snapshot, …). The
    /// projector owns its shape; the repository treats it as opaque text.
    pub metadata_json: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuditError {
    #[error("audit event_type is empty")]
    EmptyEventType,
    #[error("audit entity_type is empty")]
    EmptyEntityType,
}

#[derive(Debug, Clone)]
pub struct NewAudit {
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub client_id: Option<ClientId>,
    pub invoice_id: Option<InvoiceId>,
    pub metadata_json: String,
    pub occurred_at: DateTime<Utc>,
}

impl Audit {
    pub fn record(input: NewAudit) -> Result<Self, AuditError> {
        let event_type = input.event_type.trim().to_string();
        if event_type.is_empty() {
            return Err(AuditError::EmptyEventType);
        }
        let entity_type = input.entity_type.trim().to_string();
        if entity_type.is_empty() {
            return Err(AuditError::EmptyEntityType);
        }
        // A blank blob is normalised to a valid empty JSON object so the
        // `metadata_json` column always parses on the frontend.
        let metadata_json = if input.metadata_json.trim().is_empty() {
            "{}".to_string()
        } else {
            input.metadata_json
        };
        Ok(Self {
            id: AuditId::new(),
            event_type,
            entity_type,
            entity_id: input.entity_id,
            client_id: input.client_id,
            invoice_id: input.invoice_id,
            metadata_json,
            occurred_at: input.occurred_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn input() -> NewAudit {
        NewAudit {
            event_type: "  invoice.finalized  ".into(),
            entity_type: "  invoice  ".into(),
            entity_id: Some("the-invoice".into()),
            client_id: Some(ClientId::new()),
            invoice_id: Some(InvoiceId::new()),
            metadata_json: r#"{"number":42}"#.into(),
            occurred_at: at(),
        }
    }

    #[test]
    fn record_trims_event_and_entity_type() {
        let a = Audit::record(input()).unwrap();
        assert_eq!(a.event_type, "invoice.finalized");
        assert_eq!(a.entity_type, "invoice");
        assert_eq!(a.metadata_json, r#"{"number":42}"#);
    }

    #[test]
    fn record_rejects_empty_event_type() {
        let mut i = input();
        i.event_type = "   ".into();
        assert_eq!(Audit::record(i), Err(AuditError::EmptyEventType));
    }

    #[test]
    fn record_rejects_empty_entity_type() {
        let mut i = input();
        i.entity_type = "".into();
        assert_eq!(Audit::record(i), Err(AuditError::EmptyEntityType));
    }

    #[test]
    fn record_defaults_blank_metadata_to_empty_object() {
        let mut i = input();
        i.metadata_json = "  ".into();
        let a = Audit::record(i).unwrap();
        assert_eq!(a.metadata_json, "{}");
    }
}
