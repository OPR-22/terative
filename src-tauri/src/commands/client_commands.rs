use tauri::State;

use crate::application::client_usecases::UpdateClientInput;
use crate::application::ports::ListClientsQuery;
use crate::domain::client::{Client, ClientId, NewClient};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn client_create(
    state: State<'_, AppState>,
    input: NewClient,
) -> Result<Client, String> {
    state.create_client.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn client_update(
    state: State<'_, AppState>,
    input: UpdateClientInput,
) -> Result<Client, String> {
    state.update_client.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn client_delete(state: State<'_, AppState>, id: ClientId) -> Result<(), String> {
    state.delete_client.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn client_list(
    state: State<'_, AppState>,
    query: Option<ListClientsQuery>,
) -> Result<Vec<Client>, String> {
    state
        .list_clients
        .execute(query.unwrap_or_default())
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn client_get(state: State<'_, AppState>, id: ClientId) -> Result<Client, String> {
    state.get_client_detail.execute(id).map_err(to_ipc_err)
}
