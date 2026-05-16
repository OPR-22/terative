//! Events emitted by the [`Client`](crate::domain::client::Client) aggregate.

use chrono::{DateTime, Utc};

use crate::domain::client::ClientId;
use crate::domain::events::DomainEvent;
use crate::domain::field_change::FieldChange;

#[derive(Debug, Clone)]
pub struct ClientCreated {
    pub id: ClientId,
    pub name: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for ClientCreated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "client.created"
    }
}

#[derive(Debug, Clone)]
pub struct ClientUpdated {
    pub id: ClientId,
    /// Per-field diff between the prior snapshot and the post-update state.
    /// Empty when the use case ran but nothing actually changed.
    pub changes: Vec<FieldChange>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for ClientUpdated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "client.updated"
    }
}

#[derive(Debug, Clone)]
pub struct ClientArchived {
    pub id: ClientId,
    pub name: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for ClientArchived {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "client.archived"
    }
}

#[derive(Debug, Clone)]
pub struct ClientUnarchived {
    pub id: ClientId,
    pub name: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for ClientUnarchived {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "client.unarchived"
    }
}
