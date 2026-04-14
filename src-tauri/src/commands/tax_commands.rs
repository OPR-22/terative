use tauri::State;

use crate::application::tax_usecases::UpdateTaxInput;
use crate::domain::tax::{NewTaxDefinition, TaxDefinition, TaxId};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn tax_create(
    state: State<'_, AppState>,
    input: NewTaxDefinition,
) -> Result<TaxDefinition, String> {
    state.create_tax.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn tax_update(
    state: State<'_, AppState>,
    input: UpdateTaxInput,
) -> Result<TaxDefinition, String> {
    state.update_tax.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn tax_delete(state: State<'_, AppState>, id: TaxId) -> Result<(), String> {
    state.delete_tax.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn tax_list(
    state: State<'_, AppState>,
    include_inactive: Option<bool>,
) -> Result<Vec<TaxDefinition>, String> {
    state
        .list_taxes
        .execute(include_inactive.unwrap_or(false))
        .map_err(to_ipc_err)
}
