use tauri::State;
use uuid::Uuid;

use crate::application::dto::{EmailConfigDto, EmailLogDto, InvoiceDto};
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn settings_update_email_config(
    state: State<'_, AppState>,
    config: EmailConfigDto,
) -> Result<EmailConfigDto, String> {
    state
        .update_email_config
        .execute(config.into())
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn email_test_connection(state: State<'_, AppState>) -> Result<(), String> {
    state.test_email_connection.execute().map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_send(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    let (invoice, logs) = state
        .send_invoice
        .execute(InvoiceId(id))
        .map_err(to_ipc_err)?;
    Ok(InvoiceDto::from_invoice_with_logs(&invoice, &logs))
}

#[tauri::command]
#[specta::specta]
pub fn email_log_list_for_client(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<Vec<EmailLogDto>, String> {
    state
        .list_email_logs_for_client
        .execute(ClientId(client_id))
        .map(|logs| logs.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}
