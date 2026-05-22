use chrono::{DateTime, Utc};

use crate::application::ports::{Page, PaginationParams};
use crate::application::RepoError;
use crate::domain::audit::Audit;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

/// Append-only store for the audit log. Writes come exclusively from the
/// `AuditProjector` handlers; reads back the three UI surfaces, all
/// paginated through the standard [`Page`] / [`PaginationParams`] pair.
pub trait AuditRepository: Send + Sync {
    /// Append one audit row.
    fn insert(&self, audit: &Audit) -> Result<(), RepoError>;

    /// Most-recent audit across the whole org, newest first. Powers the
    /// dashboard "Recent audit" card and the dedicated Audit page.
    fn paginate_recent(&self, params: &PaginationParams) -> Result<Page<Audit>, RepoError>;

    /// Audit scoped to one client, newest first. Powers the per-client
    /// audit tab.
    fn paginate_by_client(
        &self,
        client_id: ClientId,
        params: &PaginationParams,
    ) -> Result<Page<Audit>, RepoError>;

    /// Audit scoped to one invoice, newest first — consistent with the
    /// other audit feeds. Powers the per-invoice audit strip.
    fn paginate_by_invoice(
        &self,
        invoice_id: InvoiceId,
        params: &PaginationParams,
    ) -> Result<Page<Audit>, RepoError>;

    /// Delete every audit row whose `occurred_at < cutoff`. Returns the
    /// number of rows removed. Used by the manual "clean up old activity"
    /// settings affordance; the FE constrains the cutoff to "today minus
    /// N years" for some `N` in 1..=5.
    fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, RepoError>;
}
