//! Events emitted by the [`Payment`](crate::domain::payment::Payment)
//! aggregate.
//!
//! `PaymentDeleted` is emitted at the use-case layer (the aggregate is gone
//! by the time it would otherwise `apply`), but it lives here alongside its
//! siblings for discoverability.

use chrono::{DateTime, Utc};

use crate::domain::client::ClientId;
use crate::domain::events::DomainEvent;
use crate::domain::field_change::FieldChange;
use crate::domain::money::Money;
use crate::domain::payment::{PaymentAllocation, PaymentId};

#[derive(Debug, Clone)]
pub struct PaymentRecorded {
    pub id: PaymentId,
    pub client_id: ClientId,
    pub amount: Money,
    /// Snapshot of every invoice this payment allocated to. The audit
    /// projector fans out one row per allocation so the payment surfaces on
    /// each invoice's activity strip, plus a client-scoped row when nothing
    /// is allocated.
    pub allocations: Vec<PaymentAllocation>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for PaymentRecorded {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "payment.recorded"
    }
}

#[derive(Debug, Clone)]
pub struct PaymentUpdated {
    pub id: PaymentId,
    pub client_id: ClientId,
    pub amount: Money,
    /// Snapshot of every invoice this payment now allocates to (after the
    /// update). Same fan-out semantics as `PaymentRecorded` — kept on the
    /// event so the audit handler can emit one row per allocated invoice.
    pub allocations: Vec<PaymentAllocation>,
    /// Per-field diff between the prior snapshot and the post-update state,
    /// including a per-invoice indexed-collection diff for `allocations`.
    /// Empty when the use case ran but nothing actually changed.
    pub changes: Vec<FieldChange>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for PaymentUpdated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "payment.updated"
    }
}

#[derive(Debug, Clone)]
pub struct PaymentDeleted {
    pub id: PaymentId,
    pub client_id: ClientId,
    pub at: DateTime<Utc>,
}
impl DomainEvent for PaymentDeleted {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "payment.deleted"
    }
}
