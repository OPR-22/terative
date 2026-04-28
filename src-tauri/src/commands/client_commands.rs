use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ClientAttributeValuesDto, ClientDto, ListClientsQueryDto, NewClientDto, PageDto,
    UpdateClientDto,
};
use crate::domain::client::ClientId;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn client_create(
    state: State<'_, AppState>,
    input: NewClientDto,
) -> Result<ClientDto, String> {
    state
        .create_client
        .execute(input.into())
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_update(
    state: State<'_, AppState>,
    input: UpdateClientDto,
) -> Result<ClientDto, String> {
    state
        .update_client
        .execute(input.into())
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_archive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .archive_client
        .execute(ClientId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_unarchive(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state
        .unarchive_client
        .execute(ClientId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_list(
    state: State<'_, AppState>,
    query: Option<ListClientsQueryDto>,
) -> Result<PageDto<ClientDto>, String> {
    let page = state
        .list_clients
        .execute(query.unwrap_or_default().into())
        .map_err(to_ipc_err)?;
    Ok(page.map(|c| ClientDto::from(&c)).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_get(state: State<'_, AppState>, id: Uuid) -> Result<ClientDto, String> {
    state
        .get_client_detail
        .execute(ClientId(id))
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_attribute_values(
    state: State<'_, AppState>,
) -> Result<ClientAttributeValuesDto, String> {
    state
        .list_client_attribute_values
        .execute()
        .map(Into::into)
        .map_err(to_ipc_err)
}
