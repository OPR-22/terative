use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    EmailTemplateDto, NewEmailTemplateDto, UpdateEmailTemplateDto,
};
use crate::domain::email_template::EmailTemplateId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn email_template_create(
    state: State<'_, AppState>,
    input: NewEmailTemplateDto,
) -> Result<EmailTemplateDto, String> {
    state
        .create_email_template
        .execute(input.into())
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn email_template_update(
    state: State<'_, AppState>,
    input: UpdateEmailTemplateDto,
) -> Result<EmailTemplateDto, String> {
    state
        .update_email_template
        .execute(
            EmailTemplateId(input.id),
            input.name,
            input.subject_template,
            input.body_template,
        )
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn email_template_delete(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .delete_email_template
        .execute(EmailTemplateId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn email_template_set_default(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .set_default_email_template
        .execute(EmailTemplateId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn email_template_list(
    state: State<'_, AppState>,
) -> Result<Vec<EmailTemplateDto>, String> {
    state
        .list_email_templates
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}
