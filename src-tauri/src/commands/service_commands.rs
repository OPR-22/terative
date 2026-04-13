use tauri::State;

use crate::application::service_usecases::UpdateServiceInput;
use crate::domain::service::{NewService, Service, ServiceId};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn service_create(
    state: State<'_, AppState>,
    input: NewService,
) -> Result<Service, String> {
    state.create_service.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn service_update(
    state: State<'_, AppState>,
    input: UpdateServiceInput,
) -> Result<Service, String> {
    state.update_service.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn service_delete(state: State<'_, AppState>, id: ServiceId) -> Result<(), String> {
    state.delete_service.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn service_list(
    state: State<'_, AppState>,
    include_inactive: Option<bool>,
) -> Result<Vec<Service>, String> {
    state
        .list_services
        .execute(include_inactive.unwrap_or(false))
        .map_err(to_ipc_err)
}
