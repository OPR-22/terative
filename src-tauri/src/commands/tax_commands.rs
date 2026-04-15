use tauri::State;
use uuid::Uuid;

use crate::application::dto::{NewTaxDefinitionDto, TaxDefinitionDto, UpdateTaxDto};
use crate::domain::tax::TaxId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn tax_create(
    state: State<'_, AppState>,
    input: NewTaxDefinitionDto,
) -> Result<TaxDefinitionDto, String> {
    state
        .create_tax
        .execute(input.into())
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn tax_update(
    state: State<'_, AppState>,
    input: UpdateTaxDto,
) -> Result<TaxDefinitionDto, String> {
    state
        .update_tax
        .execute(input.into())
        .map(|t| (&t).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn tax_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state.archive_tax.execute(TaxId(id)).map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn tax_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state.unarchive_tax.execute(TaxId(id)).map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn tax_list(
    state: State<'_, AppState>,
    include_inactive: Option<bool>,
) -> Result<Vec<TaxDefinitionDto>, String> {
    state
        .list_taxes
        .execute(include_inactive.unwrap_or(false))
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}
