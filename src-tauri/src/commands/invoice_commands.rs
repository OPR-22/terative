use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    InvoiceDto, ListInvoicesQueryDto, NewInvoiceDto, UpdateDraftInvoiceDto,
};
use crate::application::AppError;
use crate::domain::invoice::InvoiceId;

use super::{to_ipc_err, AppState};

fn dto_err(e: crate::application::dto::DtoConvertError) -> String {
    to_ipc_err(AppError::from(e))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_create_draft(
    state: State<'_, AppState>,
    input: NewInvoiceDto,
) -> Result<InvoiceDto, String> {
    let domain = input.try_into().map_err(dto_err)?;
    state
        .create_draft_invoice
        .execute(domain)
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_update_draft(
    state: State<'_, AppState>,
    input: UpdateDraftInvoiceDto,
) -> Result<InvoiceDto, String> {
    let domain = input.try_into().map_err(dto_err)?;
    state
        .update_draft_invoice
        .execute(domain)
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_finalize(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    state
        .finalize_invoice
        .execute(InvoiceId(id))
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_duplicate(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    state
        .duplicate_invoice
        .execute(InvoiceId(id))
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_cancel(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    state
        .cancel_invoice
        .execute(InvoiceId(id))
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_list(
    state: State<'_, AppState>,
    query: Option<ListInvoicesQueryDto>,
) -> Result<Vec<InvoiceDto>, String> {
    state
        .list_invoices
        .execute(query.unwrap_or_default().into())
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    state
        .get_invoice
        .execute(InvoiceId(id))
        .map(|i| (&i).into())
        .map_err(to_ipc_err)
}
