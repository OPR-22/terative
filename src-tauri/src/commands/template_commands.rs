use tauri::State;

use crate::application::template_usecases::{PreviewTemplateInput, UpdateTemplateInput};
use crate::domain::template::{InvoiceTemplate, NewInvoiceTemplate, TemplateId};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn template_create(
    state: State<'_, AppState>,
    input: NewInvoiceTemplate,
) -> Result<InvoiceTemplate, String> {
    state.create_template.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_update(
    state: State<'_, AppState>,
    input: UpdateTemplateInput,
) -> Result<InvoiceTemplate, String> {
    state.update_template.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_delete(state: State<'_, AppState>, id: TemplateId) -> Result<(), String> {
    state.delete_template.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_duplicate(
    state: State<'_, AppState>,
    id: TemplateId,
) -> Result<InvoiceTemplate, String> {
    state.duplicate_template.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_set_default(state: State<'_, AppState>, id: TemplateId) -> Result<(), String> {
    state.set_default_template.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_list(state: State<'_, AppState>) -> Result<Vec<InvoiceTemplate>, String> {
    state.list_templates.execute().map_err(to_ipc_err)
}

#[tauri::command]
pub fn template_preview(
    state: State<'_, AppState>,
    input: PreviewTemplateInput,
) -> Result<Vec<u8>, String> {
    state.preview_template.execute(input).map_err(to_ipc_err)
}
