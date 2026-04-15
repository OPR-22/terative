use chrono::NaiveDate;

use crate::application::RepoError;
use crate::domain::client::ClientId;
pub use crate::domain::invoice::DerivedPaymentStatus;
use crate::domain::invoice::{InvoiceId, InvoiceStatus};
use crate::domain::money::Money;

/// Read-only projections backed by the SQLite views (`v_invoice_payment_status`,
/// `v_client_balance`, `v_aging_report`) plus a few aggregate sums. Kept as a
/// dedicated port so the use cases don't reach past the invoice/payment ports
/// into raw SQL for reporting.
pub trait AccountingQueries: Send + Sync {
    fn list_outstanding_invoices(&self) -> Result<Vec<InvoicePaymentRow>, RepoError>;
    fn list_overdue_invoices(&self, today: NaiveDate)
        -> Result<Vec<InvoicePaymentRow>, RepoError>;
    fn revenue_by_period(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        grouping: RevenueGrouping,
    ) -> Result<Vec<RevenueBucket>, RepoError>;
    fn revenue_by_client(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByClient>, RepoError>;
    fn client_balance(&self, client_id: ClientId) -> Result<ClientBalance, RepoError>;
    fn client_balances(&self) -> Result<Vec<ClientBalance>, RepoError>;
    fn aging_report(&self, today: NaiveDate) -> Result<Vec<AgingRow>, RepoError>;
    fn dashboard_summary(&self, today: NaiveDate) -> Result<DashboardSummary, RepoError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoicePaymentRow {
    pub invoice_id: InvoiceId,
    pub number: Option<u64>,
    pub client_id: ClientId,
    pub client_name: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total: Money,
    pub amount_paid: Money,
    pub amount_due: Money,
    pub status: InvoiceStatus,
    pub payment_status: DerivedPaymentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevenueGrouping {
    Day,
    Month,
    Year,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevenueBucket {
    /// Start of the bucket (e.g. first day of the month for Month grouping).
    pub bucket_start: NaiveDate,
    pub amount: Money,
    pub invoice_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevenueByClient {
    pub client_id: ClientId,
    pub client_name: String,
    pub total_invoiced: Money,
    pub invoice_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBalance {
    pub client_id: ClientId,
    pub client_name: String,
    pub total_invoiced: Money,
    pub total_paid: Money,
    pub outstanding: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgingRow {
    pub invoice_id: InvoiceId,
    pub number: Option<u64>,
    pub client_id: ClientId,
    pub client_name: String,
    pub total: Money,
    pub amount_due: Money,
    pub due_date: Option<NaiveDate>,
    pub bucket: AgingBucket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgingBucket {
    Current,
    Days1To30,
    Days31To60,
    Days61To90,
    Days91Plus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummary {
    pub revenue_this_year: Money,
    pub outstanding_total: Money,
    pub overdue_count: u64,
    pub draft_count: u64,
    pub finalized_count: u64,
    pub sent_count: u64,
}
