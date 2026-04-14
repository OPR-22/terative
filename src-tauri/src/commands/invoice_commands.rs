use tauri::State;

use crate::application::invoice_usecases::UpdateDraftInvoiceInput;
use crate::application::ports::ListInvoicesQuery;
use crate::domain::invoice::{Invoice, InvoiceId, NewInvoice};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn invoice_create_draft(
    state: State<'_, AppState>,
    input: NewInvoice,
) -> Result<Invoice, String> {
    state.create_draft_invoice.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_update_draft(
    state: State<'_, AppState>,
    input: UpdateDraftInvoiceInput,
) -> Result<Invoice, String> {
    state.update_draft_invoice.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_finalize(
    state: State<'_, AppState>,
    id: InvoiceId,
) -> Result<Invoice, String> {
    state.finalize_invoice.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_duplicate(
    state: State<'_, AppState>,
    id: InvoiceId,
) -> Result<Invoice, String> {
    state.duplicate_invoice.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_cancel(
    state: State<'_, AppState>,
    id: InvoiceId,
) -> Result<Invoice, String> {
    state.cancel_invoice.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_list(
    state: State<'_, AppState>,
    query: Option<ListInvoicesQuery>,
) -> Result<Vec<Invoice>, String> {
    state
        .list_invoices
        .execute(query.unwrap_or_default())
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn invoice_get(
    state: State<'_, AppState>,
    id: InvoiceId,
) -> Result<Invoice, String> {
    state.get_invoice.execute(id).map_err(to_ipc_err)
}
