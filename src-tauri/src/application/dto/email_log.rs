use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::dto::EmailTemplateTypeDto;
use crate::domain::email_log::EmailLog;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct EmailLogDto {
    pub id: Uuid,
    pub client_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub template_type: Option<EmailTemplateTypeDto>,
    pub template_name: Option<String>,
    pub to_address: String,
    pub subject: String,
    pub sent_at: DateTime<Utc>,
}

impl From<&EmailLog> for EmailLogDto {
    fn from(l: &EmailLog) -> Self {
        Self {
            id: l.id.0,
            client_id: l.client_id.0,
            invoice_id: l.invoice_id.map(|i| i.0),
            template_type: l.template_type.map(Into::into),
            template_name: l.template_name.clone(),
            to_address: l.to_address.clone(),
            subject: l.subject.clone(),
            sent_at: l.sent_at,
        }
    }
}
