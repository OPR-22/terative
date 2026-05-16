use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::audit::Audit;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AuditDto {
    pub id: Uuid,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub client_id: Option<Uuid>,
    pub invoice_id: Option<Uuid>,
    /// Event-type-specific fields as a JSON string. Kept as a string (rather
    /// than a structured type) because the shape varies per `event_type`; the
    /// frontend `JSON.parse`s it and switches on `event_type`.
    pub metadata_json: String,
    pub occurred_at: DateTime<Utc>,
}

impl From<&Audit> for AuditDto {
    fn from(a: &Audit) -> Self {
        Self {
            id: a.id.0,
            event_type: a.event_type.clone(),
            entity_type: a.entity_type.clone(),
            entity_id: a.entity_id.clone(),
            client_id: a.client_id.map(|c| c.0),
            invoice_id: a.invoice_id.map(|i| i.0),
            metadata_json: a.metadata_json.clone(),
            occurred_at: a.occurred_at,
        }
    }
}
