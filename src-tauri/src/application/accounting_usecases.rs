use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::application::ports::{
    AccountingQueries, AgingRow, ClientBalance, DashboardSummary, InvoicePaymentRow,
    RevenueBucket, RevenueByClient, RevenueGrouping,
};
use crate::application::AppError;
use crate::domain::client::ClientId;

pub struct AccountingService {
    queries: Arc<dyn AccountingQueries>,
}

impl AccountingService {
    pub fn new(queries: Arc<dyn AccountingQueries>) -> Self {
        Self { queries }
    }

    pub fn list_outstanding(&self) -> Result<Vec<InvoicePaymentRow>, AppError> {
        Ok(self.queries.list_outstanding_invoices()?)
    }

    pub fn list_overdue(&self) -> Result<Vec<InvoicePaymentRow>, AppError> {
        Ok(self.queries.list_overdue_invoices(today())?)
    }

    pub fn revenue_by_period(
        &self,
        input: RevenueByPeriodInput,
    ) -> Result<Vec<RevenueBucket>, AppError> {
        Ok(self
            .queries
            .revenue_by_period(input.start, input.end, input.grouping)?)
    }

    pub fn revenue_by_client(
        &self,
        input: RevenueByClientInput,
    ) -> Result<Vec<RevenueByClient>, AppError> {
        Ok(self.queries.revenue_by_client(input.start, input.end)?)
    }

    pub fn client_balance(&self, client_id: ClientId) -> Result<ClientBalance, AppError> {
        Ok(self.queries.client_balance(client_id)?)
    }

    pub fn client_balances(&self) -> Result<Vec<ClientBalance>, AppError> {
        Ok(self.queries.client_balances()?)
    }

    pub fn aging_report(&self) -> Result<Vec<AgingRow>, AppError> {
        Ok(self.queries.aging_report(today())?)
    }

    pub fn dashboard_summary(&self) -> Result<DashboardSummary, AppError> {
        Ok(self.queries.dashboard_summary(today())?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueByPeriodInput {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub grouping: RevenueGrouping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueByClientInput {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}
