use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    AgingRowDto, ClientBalanceDto, DashboardSummaryDto, InvoicePaymentRowDto,
    RevenueBucketDto, RevenueByClientDto, RevenueByClientInputDto, RevenueByPeriodInputDto,
};
use crate::domain::client::ClientId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn accounting_list_outstanding(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRowDto>, AppError> {
    state.org()?
        .accounting
        .list_outstanding()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_list_overdue(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRowDto>, AppError> {
    state.org()?
        .accounting
        .list_overdue()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_revenue_by_period(
    state: State<'_, AppState>,
    input: RevenueByPeriodInputDto,
) -> Result<Vec<RevenueBucketDto>, AppError> {
    state.org()?
        .accounting
        .revenue_by_period(input.into())
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_revenue_by_client(
    state: State<'_, AppState>,
    input: RevenueByClientInputDto,
) -> Result<Vec<RevenueByClientDto>, AppError> {
    state.org()?
        .accounting
        .revenue_by_client(input.into())
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_client_balance(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<Vec<ClientBalanceDto>, AppError> {
    state.org()?
        .accounting
        .client_balance(ClientId(client_id))
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_client_balances(
    state: State<'_, AppState>,
) -> Result<Vec<ClientBalanceDto>, AppError> {
    state.org()?
        .accounting
        .client_balances()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_aging_report(
    state: State<'_, AppState>,
) -> Result<Vec<AgingRowDto>, AppError> {
    state.org()?
        .accounting
        .aging_report()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn accounting_dashboard_summary(
    state: State<'_, AppState>,
) -> Result<DashboardSummaryDto, AppError> {
    state.org()?
        .accounting
        .dashboard_summary()
        .map(|s| (&s).into())
}
