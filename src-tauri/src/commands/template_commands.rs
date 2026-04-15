use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    InvoiceTemplateDto, NewInvoiceTemplateDto, PreviewTemplateInputDto, UpdateTemplateDto,
};
use crate::domain::template::TemplateId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn template_create(
    state: State<'_, AppState>,
    input: NewInvoiceTemplateDto,
) -> Result<InvoiceTemplateDto, String> {
    state
        .create_template
        .execute(input.into())
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_update(
    state: State<'_, AppState>,
    input: UpdateTemplateDto,
) -> Result<InvoiceTemplateDto, String> {
    state
        .update_template
        .execute(input.into())
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .delete_template
        .execute(TemplateId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_duplicate(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceTemplateDto, String> {
    state
        .duplicate_template
        .execute(TemplateId(id))
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_set_default(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .set_default_template
        .execute(TemplateId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_list(state: State<'_, AppState>) -> Result<Vec<InvoiceTemplateDto>, String> {
    state
        .list_templates
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn template_preview(
    state: State<'_, AppState>,
    input: PreviewTemplateInputDto,
) -> Result<Vec<u8>, String> {
    state
        .preview_template
        .execute(input.into())
        .map_err(to_ipc_err)
}
