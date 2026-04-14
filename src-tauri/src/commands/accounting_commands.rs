use tauri::State;

use crate::application::accounting_usecases::{RevenueByClientInput, RevenueByPeriodInput};
use crate::application::ports::{
    AgingRow, ClientBalance, DashboardSummary, InvoicePaymentRow, RevenueBucket, RevenueByClient,
};
use crate::domain::client::ClientId;

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn accounting_list_outstanding(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRow>, String> {
    state.accounting.list_outstanding().map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_list_overdue(
    state: State<'_, AppState>,
) -> Result<Vec<InvoicePaymentRow>, String> {
    state.accounting.list_overdue().map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_revenue_by_period(
    state: State<'_, AppState>,
    input: RevenueByPeriodInput,
) -> Result<Vec<RevenueBucket>, String> {
    state.accounting.revenue_by_period(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_revenue_by_client(
    state: State<'_, AppState>,
    input: RevenueByClientInput,
) -> Result<Vec<RevenueByClient>, String> {
    state.accounting.revenue_by_client(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_client_balance(
    state: State<'_, AppState>,
    client_id: ClientId,
) -> Result<ClientBalance, String> {
    state.accounting.client_balance(client_id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_client_balances(
    state: State<'_, AppState>,
) -> Result<Vec<ClientBalance>, String> {
    state.accounting.client_balances().map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_aging_report(
    state: State<'_, AppState>,
) -> Result<Vec<AgingRow>, String> {
    state.accounting.aging_report().map_err(to_ipc_err)
}

#[tauri::command]
pub fn accounting_dashboard_summary(
    state: State<'_, AppState>,
) -> Result<DashboardSummary, String> {
    state.accounting.dashboard_summary().map_err(to_ipc_err)
}
