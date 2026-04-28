use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    InvoiceDto, ListInvoicesQueryDto, NewInvoiceDto, PageDto, UpdateDraftInvoiceDto,
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
        .map(|i| InvoiceDto::from_invoice_basic(&i))
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
        .map(|i| InvoiceDto::from_invoice_basic(&i))
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
        .map(|i| InvoiceDto::from_invoice_basic(&i))
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
        .map(|i| InvoiceDto::from_invoice_basic(&i))
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
        .map(|i| InvoiceDto::from_invoice_basic(&i))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn invoice_list(
    state: State<'_, AppState>,
    query: Option<ListInvoicesQueryDto>,
) -> Result<PageDto<InvoiceDto>, String> {
    let today = Utc::now().date_naive();
    let page = state
        .list_invoices
        .execute(query.unwrap_or_default().into())
        .map_err(to_ipc_err)?;
    Ok(page
        .map(|(i, paid, client_name)| {
            InvoiceDto::from_invoice_enriched(&i, paid, today, client_name)
        })
        .into())
}

#[tauri::command]
#[specta::specta]
pub fn invoice_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, String> {
    let today = Utc::now().date_naive();
    state
        .get_invoice
        .execute(InvoiceId(id))
        .map(|(i, paid, client_name)| {
            InvoiceDto::from_invoice_enriched(&i, paid, today, client_name)
        })
        .map_err(to_ipc_err)
}

/// Returns the rendered PDF bytes for an invoice that has already been
/// finalized (or sent / cancelled). Errors with `NotFound` for drafts or
/// when the file is missing — the UI renders an empty state in that case.
#[tauri::command]
#[specta::specta]
pub fn invoice_pdf_bytes(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<Vec<u8>, String> {
    state
        .get_invoice_pdf
        .execute(InvoiceId(id))
        .map_err(to_ipc_err)
}

/// Sends the invoice's rendered PDF to the OS default printer. No print
/// dialog — users pick their default printer at the OS level.
#[tauri::command]
#[specta::specta]
pub fn invoice_print(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .print_invoice
        .execute(InvoiceId(id))
        .map_err(to_ipc_err)
}

/// Opens the invoice PDF in the OS default application. Unlike
/// `tauri-plugin-opener`'s `openPath`, this routes through the native
/// `open` / `xdg-open` / `cmd start` commands, which reliably brings the
/// target app to the foreground on macOS.
#[tauri::command]
#[specta::specta]
pub fn invoice_open_external(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), String> {
    state
        .open_invoice_externally
        .execute(InvoiceId(id))
        .map_err(to_ipc_err)
}
