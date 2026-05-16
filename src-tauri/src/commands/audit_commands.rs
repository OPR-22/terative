use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::application::dto::{AuditDto, PageDto, PaginationParamsDto};
use crate::application::AppError;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

/// Dashboard "Recent audit" card and the dedicated Audit page —
/// newest-first across the whole org.
#[tauri::command]
#[specta::specta]
pub fn audit_paginate_recent(
    state: State<'_, AppState>,
    pagination: Option<PaginationParamsDto>,
) -> Result<PageDto<AuditDto>, AppError> {
    let page = state
        .org()?
        .paginate_recent_audit
        .execute(pagination.into())?;
    Ok(page.map(|a| AuditDto::from(&a)).into())
}

/// Per-client audit tab — newest-first, scoped to one client.
#[tauri::command]
#[specta::specta]
pub fn audit_paginate_for_client(
    state: State<'_, AppState>,
    client_id: Uuid,
    pagination: Option<PaginationParamsDto>,
) -> Result<PageDto<AuditDto>, AppError> {
    let page = state
        .org()?
        .paginate_audit_for_client
        .execute(ClientId(client_id), pagination.into())?;
    Ok(page.map(|a| AuditDto::from(&a)).into())
}

/// Per-invoice audit strip — chronological timeline for one invoice.
#[tauri::command]
#[specta::specta]
pub fn audit_paginate_for_invoice(
    state: State<'_, AppState>,
    invoice_id: Uuid,
    pagination: Option<PaginationParamsDto>,
) -> Result<PageDto<AuditDto>, AppError> {
    let page = state
        .org()?
        .paginate_audit_for_invoice
        .execute(InvoiceId(invoice_id), pagination.into())?;
    Ok(page.map(|a| AuditDto::from(&a)).into())
}
