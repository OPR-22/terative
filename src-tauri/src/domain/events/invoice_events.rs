//! Events emitted by the [`Invoice`](crate::domain::invoice::Invoice)
//! aggregate. Each carries only what an audit-log handler needs to render
//! a row without re-fetching the invoice.

use chrono::{DateTime, Utc};

use crate::domain::client::ClientId;
use crate::domain::events::DomainEvent;
use crate::domain::invoice::{InvoiceId, InvoiceNumber};
use crate::domain::money::Money;

#[derive(Debug, Clone)]
pub struct InvoiceDraftCreated {
    pub id: InvoiceId,
    pub client_id: ClientId,
    pub total: Money,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceDraftCreated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.draft_created"
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceDraftUpdated {
    pub id: InvoiceId,
    /// Kept for routing — the audit handler scopes the row to this client.
    pub client_id: ClientId,
    /// Per-field diff between the prior snapshot and the post-update state.
    /// Empty when the use case ran but nothing actually changed.
    pub changes: Vec<crate::domain::field_change::FieldChange>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceDraftUpdated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.draft_updated"
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceFinalized {
    pub id: InvoiceId,
    pub client_id: ClientId,
    pub number: InvoiceNumber,
    pub total: Money,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceFinalized {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.finalized"
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceCancelled {
    pub id: InvoiceId,
    pub client_id: ClientId,
    /// `None` only for the (impossible-by-domain-rules) case of a cancelled
    /// draft; kept optional to mirror `Invoice::number`.
    pub number: Option<InvoiceNumber>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceCancelled {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.cancelled"
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceDuplicated {
    pub source_id: InvoiceId,
    pub new_id: InvoiceId,
    pub client_id: ClientId,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceDuplicated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.duplicated"
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceSent {
    pub id: InvoiceId,
    pub client_id: ClientId,
    pub number: Option<InvoiceNumber>,
    pub to_address: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for InvoiceSent {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "invoice.sent"
    }
}
