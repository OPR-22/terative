use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ClientAttributeValuesDto, ClientDto, ListClientsQueryDto, NewClientDto, PageDto,
    UpdateClientDto,
};
use crate::domain::client::ClientId;

use super::AppState;
use crate::application::AppError;

#[tauri::command]
#[specta::specta]
pub fn client_create(
    state: State<'_, AppState>,
    input: NewClientDto,
) -> Result<ClientDto, AppError> {
    state.org()?
        .create_client
        .execute(input.into())
        .map(|c| (&c).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_update(
    state: State<'_, AppState>,
    input: UpdateClientDto,
) -> Result<ClientDto, AppError> {
    state.org()?
        .update_client
        .execute(input.into())
        .map(|c| (&c).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .archive_client
        .execute(ClientId(id))
}

#[tauri::command]
#[specta::specta]
pub fn client_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), AppError> {
    state.org()?
        .unarchive_client
        .execute(ClientId(id))
}

#[tauri::command]
#[specta::specta]
pub fn client_list(
    state: State<'_, AppState>,
    query: Option<ListClientsQueryDto>,
) -> Result<PageDto<ClientDto>, AppError> {
    let page = state.org()?
        .list_clients
        .execute(query.unwrap_or_default().into())?;
    Ok(page.map(|c| ClientDto::from(&c)).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_get(state: State<'_, AppState>, id: Uuid) -> Result<ClientDto, AppError> {
    state.org()?
        .get_client_detail
        .execute(ClientId(id))
        .map(|c| (&c).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_attribute_values(
    state: State<'_, AppState>,
) -> Result<ClientAttributeValuesDto, AppError> {
    state.org()?
        .list_client_attribute_values
        .execute()
        .map(Into::into)
}
