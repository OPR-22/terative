use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    CatalogItemDto, NewCatalogItemDto, UpdateCatalogItemDto,
};
use crate::application::AppError;
use crate::domain::catalog_item::CatalogItemId;

use super::AppState;

#[tauri::command]
#[specta::specta]
pub fn catalog_item_create(
    state: State<'_, AppState>,
    input: NewCatalogItemDto,
) -> Result<CatalogItemDto, AppError> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        AppError::from(e)
    })?;
    state.org()?
        .create_catalog_item
        .execute(input)
        .map(|s| (&s).into())
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_update(
    state: State<'_, AppState>,
    input: UpdateCatalogItemDto,
) -> Result<CatalogItemDto, AppError> {
    let input = input.try_into().map_err(|e: crate::application::dto::DtoConvertError| {
        AppError::from(e)
    })?;
    state.org()?
        .update_catalog_item
        .execute(input)
        .map(|s| (&s).into())
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .archive_catalog_item
        .execute(CatalogItemId(id))
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .unarchive_catalog_item
        .execute(CatalogItemId(id))
}

#[tauri::command]
#[specta::specta]
pub fn catalog_item_list(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> Result<Vec<CatalogItemDto>, AppError> {
    state.org()?
        .list_catalog_items
        .execute(include_archived.unwrap_or(false))
        .map(|list| list.iter().map(Into::into).collect())
}
