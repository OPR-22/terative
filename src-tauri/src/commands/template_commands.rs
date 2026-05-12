use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    InvoiceTemplateDto, NewInvoiceTemplateDto, PreviewTemplateInputDto, UpdateTemplateDto,
};
use crate::domain::template::TemplateId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn template_create(
    state: State<'_, AppState>,
    input: NewInvoiceTemplateDto,
) -> Result<InvoiceTemplateDto, AppError> {
    state.org()?
        .create_template
        .execute(input.into())
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn template_update(
    state: State<'_, AppState>,
    input: UpdateTemplateDto,
) -> Result<InvoiceTemplateDto, AppError> {
    state.org()?
        .update_template
        .execute(input.into())
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn template_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .delete_template
        .execute(TemplateId(id))
}

#[tauri::command]
#[specta::specta]
pub fn template_duplicate(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InvoiceTemplateDto, AppError> {
    state.org()?
        .duplicate_template
        .execute(TemplateId(id))
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn template_set_default(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .set_default_template
        .execute(TemplateId(id))
}

#[tauri::command]
#[specta::specta]
pub fn template_list(state: State<'_, AppState>) -> Result<Vec<InvoiceTemplateDto>, AppError> {
    state.org()?
        .list_templates
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn template_preview(
    state: State<'_, AppState>,
    input: PreviewTemplateInputDto,
) -> Result<Vec<u8>, AppError> {
    state.org()?
        .preview_template
        .execute(input.into())
}
