use tauri::State;

use crate::domain::invoice::{Invoice, InvoiceId};
use crate::domain::settings::EmailConfig;

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn settings_update_email_config(
    state: State<'_, AppState>,
    config: EmailConfig,
) -> Result<EmailConfig, String> {
    state.update_email_config.execute(config).map_err(to_ipc_err)
}

#[tauri::command]
pub fn settings_update_email_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    state
        .update_email_password
        .execute(&password)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn email_test_connection(state: State<'_, AppState>) -> Result<(), String> {
    state.test_email_connection.execute().map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_send(
    state: State<'_, AppState>,
    id: InvoiceId,
) -> Result<Invoice, String> {
    state.send_invoice.execute(id).map_err(to_ipc_err)
}
