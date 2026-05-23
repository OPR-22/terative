use std::collections::HashMap;

use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::email_log::EmailLog;
use crate::domain::invoice::InvoiceId;

pub trait EmailLogRepository: Send + Sync {
    fn insert(&self, log: &EmailLog) -> Result<(), RepoError>;
    /// Returns logs for the client, ordered by `sent_at` descending.
    fn list_by_client(&self, client_id: ClientId) -> Result<Vec<EmailLog>, RepoError>;
    /// Returns logs grouped by invoice id, ordered by `sent_at` ascending
    /// within each group (so the UI can render history chronologically).
    /// Invoices without any logs are absent from the map.
    fn list_by_invoices(
        &self,
        invoice_ids: &[InvoiceId],
    ) -> Result<HashMap<InvoiceId, Vec<EmailLog>>, RepoError>;
}
