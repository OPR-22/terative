use tauri::State;
use uuid::Uuid;

use crate::application::dto::{ClientDto, ListClientsQueryDto, NewClientDto, UpdateClientDto};
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
) -> Result<Vec<ClientDto>, String> {
    state
        .list_clients
        .execute(query.unwrap_or_default().into())
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
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
