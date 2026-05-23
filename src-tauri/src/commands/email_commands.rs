use tauri::State;
use uuid::Uuid;

use crate::application::dto::{EmailConfigDto, EmailLogDto, InvoiceDto};
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn settings_update_email_config(
    state: State<'_, AppState>,
    config: EmailConfigDto,
) -> Result<EmailConfigDto, AppError> {
    state.org()?
        .update_email_config
        .execute(config.into())
        .map(|c| (&c).into())
}

#[tauri::command]
#[specta::specta]
pub fn settings_update_email_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), AppError> {
    state.org()?
        .update_email_password
        .execute(&password)
}

#[tauri::command]
#[specta::specta]
pub fn email_test_connection(state: State<'_, AppState>) -> Result<(), AppError> {
    state.org()?.test_email_connection.execute()
}

#[tauri::command]
#[specta::specta]
pub fn invoice_send(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, AppError> {
    let (invoice, logs) = state.org()?
        .send_invoice
        .execute(InvoiceId(id))?;
    Ok(InvoiceDto::from_invoice_with_logs(&invoice, &logs))
}

#[tauri::command]
#[specta::specta]
pub fn email_log_list_for_client(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<Vec<EmailLogDto>, AppError> {
    state.org()?
        .list_email_logs_for_client
        .execute(ClientId(client_id))
        .map(|logs| logs.iter().map(Into::into).collect())
}
