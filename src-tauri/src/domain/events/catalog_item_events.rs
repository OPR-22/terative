//! Events emitted by the [`CatalogItem`](crate::domain::catalog_item::CatalogItem)
//! aggregate. Audit-log scope: only `created` / `updated` are tracked
//! (archive/unarchive are deliberately omitted).

use chrono::{DateTime, Utc};

use crate::domain::catalog_item::CatalogItemId;
use crate::domain::events::DomainEvent;
use crate::domain::field_change::FieldChange;

#[derive(Debug, Clone)]
pub struct CatalogItemCreated {
    pub id: CatalogItemId,
    pub name: String,
    pub at: DateTime<Utc>,
}
impl DomainEvent for CatalogItemCreated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "catalog_item.created"
    }
}

#[derive(Debug, Clone)]
pub struct CatalogItemUpdated {
    pub id: CatalogItemId,
    /// Per-field diff between the prior snapshot and the post-update state.
    /// Empty when the use case ran but nothing actually changed.
    pub changes: Vec<FieldChange>,
    pub at: DateTime<Utc>,
}
impl DomainEvent for CatalogItemUpdated {
    fn occurred_at(&self) -> DateTime<Utc> {
        self.at
    }
    fn event_name(&self) -> &'static str {
        "catalog_item.updated"
    }
}
