//! Read-only paginated use cases over the audit log — one per UI surface.
//! Thin wrappers around [`AuditRepository`]; all ordering, paging and
//! limiting is the repository's job.
//!
//! Plus the lone write operation that isn't a projection: [`CleanupAudits`],
//! the user-triggered "delete everything older than X" maintenance command.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::application::ports::{AuditRepository, Page, PaginationParams};
use crate::application::AppError;
use crate::domain::audit::Audit;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

/// Dashboard "Recent audit" card and the dedicated Audit page.
pub struct PaginateRecentAudit {
    repo: Arc<dyn AuditRepository>,
}

impl PaginateRecentAudit {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, params: PaginationParams) -> Result<Page<Audit>, AppError> {
        Ok(self.repo.paginate_recent(&params)?)
    }
}

/// Per-client audit tab.
pub struct PaginateAuditForClient {
    repo: Arc<dyn AuditRepository>,
}

impl PaginateAuditForClient {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(
        &self,
        client_id: ClientId,
        params: PaginationParams,
    ) -> Result<Page<Audit>, AppError> {
        Ok(self.repo.paginate_by_client(client_id, &params)?)
    }
}

/// Per-invoice audit strip.
/// Manual maintenance command: delete every audit row older than `cutoff`.
/// The FE constrains `cutoff` to "today minus N years" for some
/// `N ∈ 1..=5`, but the use case itself accepts any timestamp — callers
/// past the FE are trusted (this is a single-user local app).
pub struct CleanupAudits {
    repo: Arc<dyn AuditRepository>,
}

impl CleanupAudits {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    /// Returns the number of rows removed.
    pub fn execute(&self, cutoff: DateTime<Utc>) -> Result<u64, AppError> {
        Ok(self.repo.delete_older_than(cutoff)?)
    }
}

pub struct PaginateAuditForInvoice {
    repo: Arc<dyn AuditRepository>,
}

impl PaginateAuditForInvoice {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(
        &self,
        invoice_id: InvoiceId,
        params: PaginationParams,
    ) -> Result<Page<Audit>, AppError> {
        Ok(self.repo.paginate_by_invoice(invoice_id, &params)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::audit_handlers::test_support::InMemoryAuditRepo;
    use crate::domain::audit::{Audit, NewAudit};
    use chrono::{Duration, Utc};

    fn page(per_page: u32) -> PaginationParams {
        PaginationParams { page: 1, per_page }
    }

    fn seed(repo: &InMemoryAuditRepo, event_type: &str, client: Option<ClientId>, ago_h: i64) {
        let a = Audit::record(NewAudit {
            event_type: event_type.into(),
            entity_type: "test".into(),
            entity_id: None,
            client_id: client,
            invoice_id: None,
            metadata_json: "{}".into(),
            occurred_at: Utc::now() - Duration::hours(ago_h),
        })
        .unwrap();
        repo.insert(&a).unwrap();
    }

    #[test]
    fn paginate_recent_returns_newest_first_with_total() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        seed(&repo, "a.old", None, 2);
        seed(&repo, "a.new", None, 0);
        let result = PaginateRecentAudit::new(repo).execute(page(10)).unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.data[0].event_type, "a.new");
        assert_eq!(result.data[1].event_type, "a.old");
    }

    #[test]
    fn paginate_for_client_scopes_to_that_client() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        let alice = ClientId::new();
        let bob = ClientId::new();
        seed(&repo, "a.1", Some(alice), 1);
        seed(&repo, "b.1", Some(bob), 0);
        let result = PaginateAuditForClient::new(repo)
            .execute(alice, page(10))
            .unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.data[0].event_type, "a.1");
    }

    #[test]
    fn cleanup_audits_deletes_rows_older_than_cutoff_and_returns_count() {
        let repo = Arc::new(InMemoryAuditRepo::default());
        // Two rows older than 24h, one newer.
        seed(&repo, "old.1", None, 48);
        seed(&repo, "old.2", None, 30);
        seed(&repo, "fresh", None, 1);

        let cutoff = Utc::now() - chrono::Duration::hours(24);
        let removed = CleanupAudits::new(repo.clone()).execute(cutoff).unwrap();
        assert_eq!(removed, 2);

        // Only the fresh row remains.
        let remaining = PaginateRecentAudit::new(repo).execute(page(10)).unwrap();
        assert_eq!(remaining.total, 1);
        assert_eq!(remaining.data[0].event_type, "fresh");
    }
}
