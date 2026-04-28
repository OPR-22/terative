use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    CatalogItemDto, NewCatalogItemDto, UpdateCatalogItemDto,
};
use crate::application::AppError;
use crate::domain::catalog_item::CatalogItemId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn catalog_item_create(
    state: State<'_, AppState>,
    input: NewCatalogItemDto,
) -> Result<CatalogItemDto, String> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        to_ipc_err(AppError::from(e))
    })?;
    state
        .create_catalog_item
        .execute(input)
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_update(
    state: State<'_, AppState>,
    input: UpdateCatalogItemDto,
) -> Result<CatalogItemDto, String> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        to_ipc_err(AppError::from(e))
    })?;
    state
        .update_catalog_item
        .execute(input)
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .archive_catalog_item
        .execute(CatalogItemId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .unarchive_catalog_item
        .execute(CatalogItemId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_list(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> Result<Vec<CatalogItemDto>, String> {
    state
        .list_catalog_items
        .execute(include_archived.unwrap_or(false))
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}
