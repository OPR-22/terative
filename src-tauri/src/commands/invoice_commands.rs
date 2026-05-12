use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    InvoiceDto, ListInvoicesQueryDto, NewInvoiceDto, PageDto, UpdateDraftInvoiceDto,
};
use crate::application::AppError;
use crate::domain::invoice::InvoiceId;

use super::AppState;

#[tauri::command]
#[specta::specta]
pub fn invoice_create_draft(
    state: State<'_, AppState>,
    input: NewInvoiceDto,
) -> Result<InvoiceDto, AppError> {
    let domain = input.try_into()?;
    state.org()?
        .create_draft_invoice
        .execute(domain)
        .map(|i| InvoiceDto::from_invoice_basic(&i))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_update_draft(
    state: State<'_, AppState>,
    input: UpdateDraftInvoiceDto,
) -> Result<InvoiceDto, AppError> {
    let domain = input.try_into()?;
    state.org()?
        .update_draft_invoice
        .execute(domain)
        .map(|i| InvoiceDto::from_invoice_basic(&i))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_finalize(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, AppError> {
    state.org()?
        .finalize_invoice
        .execute(InvoiceId(id))
        .map(|i| InvoiceDto::from_invoice_basic(&i))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_duplicate(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, AppError> {
    state.org()?
        .duplicate_invoice
        .execute(InvoiceId(id))
        .map(|i| InvoiceDto::from_invoice_basic(&i))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_cancel(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, AppError> {
    state.org()?
        .cancel_invoice
        .execute(InvoiceId(id))
        .map(|i| InvoiceDto::from_invoice_basic(&i))
}

#[tauri::command]
#[specta::specta]
pub fn invoice_list(
    state: State<'_, AppState>,
    query: Option<ListInvoicesQueryDto>,
) -> Result<PageDto<InvoiceDto>, AppError> {
    let today = Utc::now().date_naive();
    let page = state.org()?
        .list_invoices
        .execute(query.unwrap_or_default().into())?;
    Ok(page
        .map(|(i, paid, client_name, logs)| {
            InvoiceDto::from_invoice_enriched(&i, paid, today, client_name, &logs)
        })
        .into())
}

#[tauri::command]
#[specta::specta]
pub fn invoice_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceDto, AppError> {
    let today = Utc::now().date_naive();
    state.org()?
        .get_invoice
        .execute(InvoiceId(id))
        .map(|(i, paid, client_name, logs)| {
            InvoiceDto::from_invoice_enriched(&i, paid, today, client_name, &logs)
        })
}

/// Returns the rendered PDF bytes for an invoice that has already been
/// finalized (or sent / cancelled). Errors with `NotFound` for drafts or
/// when the file is missing — the UI renders an empty state in that case.
#[tauri::command]
#[specta::specta]
pub fn invoice_pdf_bytes(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<Vec<u8>, AppError> {
    state.org()?
        .get_invoice_pdf
        .execute(InvoiceId(id))
}

/// Sends the invoice's rendered PDF to the OS default printer. No print
/// dialog — users pick their default printer at the OS level.
#[tauri::command]
#[specta::specta]
pub fn invoice_print(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .print_invoice
        .execute(InvoiceId(id))
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
) -> Result<(), AppError> {
    state.org()?
        .open_invoice_externally
        .execute(InvoiceId(id))
}
