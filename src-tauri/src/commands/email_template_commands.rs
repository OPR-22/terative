use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    EmailTemplateDto, NewEmailTemplateDto, UpdateEmailTemplateDto,
};
use crate::domain::email_template::EmailTemplateId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn email_template_create(
    state: State<'_, AppState>,
    input: NewEmailTemplateDto,
) -> Result<EmailTemplateDto, AppError> {
    state.org()?
        .create_email_template
        .execute(input.into())
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn email_template_update(
    state: State<'_, AppState>,
    input: UpdateEmailTemplateDto,
) -> Result<EmailTemplateDto, AppError> {
    state.org()?
        .update_email_template
        .execute(
            EmailTemplateId(input.id),
            input.name,
            input.subject_template,
            input.body_template,
        )
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn email_template_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .delete_email_template
        .execute(EmailTemplateId(id))
}

#[tauri::command]
#[specta::specta]
pub fn email_template_set_default(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .set_default_email_template
        .execute(EmailTemplateId(id))
}

#[tauri::command]
#[specta::specta]
pub fn email_template_list(
    state: State<'_, AppState>,
) -> Result<Vec<EmailTemplateDto>, AppError> {
    state.org()?
        .list_email_templates
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
}
