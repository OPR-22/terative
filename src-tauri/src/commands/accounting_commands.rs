use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    AgingRowDto, ClientBalanceDto, DashboardSummaryDto, InvoicePaymentRowDto,
    RevenueBucketDto, RevenueByClientDto, RevenueByClientInputDto, RevenueByPeriodInputDto,
};
use crate::domain::client::ClientId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn accounting_list_outstanding(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRowDto>, String> {
    state
        .accounting
        .list_outstanding()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_list_overdue(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRowDto>, String> {
    state
        .accounting
        .list_overdue()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_revenue_by_period(
    state: State<'_, AppState>,
    input: RevenueByPeriodInputDto,
) -> Result<Vec<RevenueBucketDto>, String> {
    state
        .accounting
        .revenue_by_period(input.into())
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_revenue_by_client(
    state: State<'_, AppState>,
    input: RevenueByClientInputDto,
) -> Result<Vec<RevenueByClientDto>, String> {
    state
        .accounting
        .revenue_by_client(input.into())
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_client_balance(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<ClientBalanceDto, String> {
    state
        .accounting
        .client_balance(ClientId(client_id))
        .map(|b| (&b).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_client_balances(
    state: State<'_, AppState>,
) -> Result<Vec<ClientBalanceDto>, String> {
    state
        .accounting
        .client_balances()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_aging_report(
    state: State<'_, AppState>,
) -> Result<Vec<AgingRowDto>, String> {
    state
        .accounting
        .aging_report()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn accounting_dashboard_summary(
    state: State<'_, AppState>,
) -> Result<DashboardSummaryDto, String> {
    state
        .accounting
        .dashboard_summary()
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}
