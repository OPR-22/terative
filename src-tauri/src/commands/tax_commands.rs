use tauri::State;
use uuid::Uuid;

use crate::application::dto::{NewTaxDefinitionDto, TaxDefinitionDto, UpdateTaxDto};
use crate::domain::tax::TaxId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn tax_create(
    state: State<'_, AppState>,
    input: NewTaxDefinitionDto,
) -> Result<TaxDefinitionDto, AppError> {
    state.org()?
        .create_tax
        .execute(input.into())
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn tax_update(
    state: State<'_, AppState>,
    input: UpdateTaxDto,
) -> Result<TaxDefinitionDto, AppError> {
    state.org()?
        .update_tax
        .execute(input.into())
        .map(|t| (&t).into())
}

#[tauri::command]
#[specta::specta]
pub fn tax_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?.archive_tax.execute(TaxId(id))
}

#[tauri::command]
#[specta::specta]
pub fn tax_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?.unarchive_tax.execute(TaxId(id))
}

#[tauri::command]
#[specta::specta]
pub fn tax_list(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> Result<Vec<TaxDefinitionDto>, AppError> {
    state.org()?
        .list_taxes
        .execute(include_archived.unwrap_or(false))
        .map(|list| list.iter().map(Into::into).collect())
}
