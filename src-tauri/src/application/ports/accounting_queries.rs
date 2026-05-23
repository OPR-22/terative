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
///
/// Strict-silos multi-currency: per-currency totals are never summed across
/// currencies. Aggregate queries return `Vec` keyed by currency (and other
/// dimensions); the UI is responsible for rendering one row/chart/table per
/// currency.
pub trait AccountingQueries: Send + Sync {
    fn list_outstanding_invoices(&self) -> Result<Vec<InvoicePaymentRow>, RepoError>;
    fn list_overdue_invoices(&self, today: NaiveDate)
        -> Result<Vec<InvoicePaymentRow>, RepoError>;
    /// One row per (bucket, currency). A bucket with two currencies of activity
    /// produces two rows that share the same `bucket_start`.
    fn revenue_by_period(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        grouping: RevenueGrouping,
    ) -> Result<Vec<RevenueBucket>, RepoError>;
    /// One row per (client, currency).
    fn revenue_by_client(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByClient>, RepoError>;
    /// One row per currency the client has activity in. Empty when there are
    /// no invoices and no payments for the client (yet).
    fn client_balance(&self, client_id: ClientId) -> Result<Vec<ClientBalance>, RepoError>;
    /// One row per (client, currency). Clients with no activity are omitted.
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

/// One per-currency row for the dashboard "revenue" card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardRevenueRow {
    pub amount: Money,
    /// Number of finalized/sent invoices in this currency this year.
    pub invoice_count: u64,
}

/// One per-currency row for the dashboard "outstanding" card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardOutstandingRow {
    /// Total still due (finalized/sent invoices, total - amount_paid > 0).
    pub outstanding: Money,
    /// Subset of `outstanding` that is past its due date today.
    pub overdue: Money,
    /// Number of open invoices in this currency.
    pub open_count: u64,
    /// Subset of `open_count` that is overdue.
    pub overdue_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSummary {
    /// Per-currency rows for the revenue card. Empty when no finalized/sent
    /// invoice exists this year.
    pub revenue_this_year: Vec<DashboardRevenueRow>,

    /// Per-currency rows for the outstanding card.
    pub outstanding: Vec<DashboardOutstandingRow>,

    /// Global overdue count across all currencies (footer of the
    /// outstanding card).
    pub overdue_count: u64,
    /// Max days past due across overdue invoices. `0` when nothing is
    /// overdue.
    pub overdue_max_days: u64,

    /// Average days between issue date and payment date for invoices paid
    /// in the last 12 months. `None` when no payments were recorded in
    /// that window.
    pub avg_payment_delay_days: Option<f64>,
    /// User-configured target for `avg_payment_delay_days`, read from
    /// `app_preferences.default_invoice_due_days`.
    pub avg_payment_delay_target_days: u64,

    /// Clients with `archived_at IS NULL`.
    pub active_clients_count: u64,
    /// Clients with `created_at` in the current calendar year.
    pub new_clients_this_year_count: u64,

    /// Finalized or Sent invoices issued this year.
    pub finalized_this_year_count: u64,
    /// Drafts created this year (`created_at`, not `date`).
    pub drafts_this_year_count: u64,
}
