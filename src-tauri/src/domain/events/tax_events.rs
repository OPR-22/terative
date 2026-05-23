//! Events emitted by the [`TaxDefinition`](crate::domain::tax::TaxDefinition)
//! aggregate. Audit-log scope: only `created` / `updated` are tracked
//! (archive/unarchive are deliberately omitted).

use chrono::{DateTime, Utc};

use crate::domain::events::DomainEvent;
use crate::domain::field_change::FieldChange;
use crate::domain::tax::TaxId;

#[derive(Debug, Clone)]
pub struct TaxCreated {
    pub id: TaxId,
    pub name: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for TaxCreated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "tax.created"
    }
}

#[derive(Debug, Clone)]
pub struct TaxUpdated {
    pub id: TaxId,
    /// Per-field diff between the prior snapshot and the post-update state.
    /// Empty when the use case ran but nothing actually changed. The diff is
    /// the *whole* update payload — no top-level `name` snapshot, since if
    /// the name changed it appears in `changes`, and otherwise the entity
    /// can be resolved by `id`.
    pub changes: Vec<FieldChange>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for TaxUpdated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "tax.updated"
    }
}
