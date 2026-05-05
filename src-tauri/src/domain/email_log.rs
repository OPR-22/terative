use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::client::ClientId;
use crate::domain::email_template::EmailTemplateType;
use crate::domain::invoice::InvoiceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmailLogId(pub Uuid);

impl EmailLogId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EmailLogId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EmailLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Append-only record of an outbound email. Currently only `invoice_send`
/// produces these, but the structure supports non-invoice contexts (the
/// `invoice_id` is optional) so future "ad-hoc reminder" or "broadcast" flows
/// can write to the same log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailLog {
    pub id: EmailLogId,
    pub client_id: ClientId,
    pub invoice_id: Option<InvoiceId>,
    pub template_type: Option<EmailTemplateType>,
    pub template_name: Option<String>,
    pub to_address: String,
    pub subject: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmailLogError {
    #[error("recipient address is empty")]
    EmptyRecipient,
    #[error("subject is empty")]
    EmptySubject,
}

#[derive(Debug, Clone)]
pub struct NewEmailLog {
    pub client_id: ClientId,
    pub invoice_id: Option<InvoiceId>,
    pub template_type: Option<EmailTemplateType>,
    pub template_name: Option<String>,
    pub to_address: String,
    pub subject: String,
    pub sent_at: DateTime<Utc>,
}

impl EmailLog {
    pub fn record(input: NewEmailLog) -> Result<Self, EmailLogError> {
        let to_address = input.to_address.trim().to_string();
        if to_address.is_empty() {
            return Err(EmailLogError::EmptyRecipient);
        }
        let subject = input.subject.trim().to_string();
        if subject.is_empty() {
            return Err(EmailLogError::EmptySubject);
        }
        Ok(Self {
            id: EmailLogId::new(),
            client_id: input.client_id,
            invoice_id: input.invoice_id,
            template_type: input.template_type,
            template_name: input.template_name.map(|s| s.trim().to_string()),
            to_address,
            subject,
            sent_at: input.sent_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> NewEmailLog {
        NewEmailLog {
            client_id: ClientId::new(),
            invoice_id: Some(InvoiceId::new()),
            template_type: Some(EmailTemplateType::InitialContact),
            template_name: Some(" Default ".into()),
            to_address: "  billing@acme.example  ".into(),
            subject: "  Invoice 1001  ".into(),
            sent_at: Utc::now(),
        }
    }

    #[test]
    fn record_trims_strings_and_returns_log() {
        let log = EmailLog::record(input()).unwrap();
        assert_eq!(log.to_address, "billing@acme.example");
        assert_eq!(log.subject, "Invoice 1001");
        assert_eq!(log.template_name.as_deref(), Some("Default"));
    }

    #[test]
    fn record_rejects_empty_recipient() {
        let mut i = input();
        i.to_address = "   ".into();
        assert_eq!(EmailLog::record(i), Err(EmailLogError::EmptyRecipient));
    }

    #[test]
    fn record_rejects_empty_subject() {
        let mut i = input();
        i.subject = "".into();
        assert_eq!(EmailLog::record(i), Err(EmailLogError::EmptySubject));
    }
}
