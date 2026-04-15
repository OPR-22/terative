use tauri::State;
use uuid::Uuid;

use crate::application::dto::{NewServiceDto, ServiceDto, UpdateServiceDto};
use crate::application::AppError;
use crate::domain::service::ServiceId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn service_create(
    state: State<'_, AppState>,
    input: NewServiceDto,
) -> Result<ServiceDto, String> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        to_ipc_err(AppError::from(e))
    })?;
    state
        .create_service
        .execute(input)
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn service_update(
    state: State<'_, AppState>,
    input: UpdateServiceDto,
) -> Result<ServiceDto, String> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        to_ipc_err(AppError::from(e))
    })?;
    state
        .update_service
        .execute(input)
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn service_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .archive_service
        .execute(ServiceId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn service_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .unarchive_service
        .execute(ServiceId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn service_list(
    state: State<'_, AppState>,
    include_inactive: Option<bool>,
) -> Result<Vec<ServiceDto>, String> {
    state
        .list_services
        .execute(include_inactive.unwrap_or(false))
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}
