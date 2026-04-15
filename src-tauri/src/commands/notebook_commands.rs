use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ClientJournalEntryDto, ClientNotebookViewDto, NewJournalEntryDto, NotebookSectionDto,
    RenameNotebookSectionDto, SaveClientNotebookDto, UpdateJournalEntryDto,
};
use crate::domain::client::ClientId;
use crate::domain::notebook::{JournalEntryId, NotebookSectionId};

use super::{to_ipc_err, AppState};

// ---- section management (Settings) ----

#[tauri::command]
#[specta::specta]
pub fn notebook_section_create(
    state: State<'_, AppState>,
    name: String,
) -> Result<NotebookSectionDto, String> {
    state
        .create_notebook_section
        .execute(name)
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_rename(
    state: State<'_, AppState>,
    input: RenameNotebookSectionDto,
) -> Result<NotebookSectionDto, String> {
    state
        .rename_notebook_section
        .execute(input.into())
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_delete(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), String> {
    state
        .delete_notebook_section
        .execute(NotebookSectionId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_count_entries(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<u64, String> {
    state
        .count_section_entries
        .execute(NotebookSectionId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_reorder(
    state: State<'_, AppState>,
    ordered_ids: Vec<Uuid>,
) -> Result<(), String> {
    let ids: Vec<NotebookSectionId> = ordered_ids.into_iter().map(NotebookSectionId).collect();
    state
        .reorder_notebook_sections
        .execute(ids)
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_list(
    state: State<'_, AppState>,
) -> Result<Vec<NotebookSectionDto>, String> {
    state
        .list_notebook_sections
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

// ---- client notebook ----

#[tauri::command]
#[specta::specta]
pub fn client_notebook_get(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<ClientNotebookViewDto, String> {
    state
        .get_client_notebook
        .execute(ClientId(client_id))
        .map(|v| (&v).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn client_notebook_save(
    state: State<'_, AppState>,
    input: SaveClientNotebookDto,
) -> Result<(), String> {
    state
        .save_client_notebook
        .execute(input.into())
        .map_err(to_ipc_err)
}

// ---- journal ----

#[tauri::command]
#[specta::specta]
pub fn journal_entry_create(
    state: State<'_, AppState>,
    input: NewJournalEntryDto,
) -> Result<ClientJournalEntryDto, String> {
    state
        .create_journal_entry
        .execute(input.into())
        .map(|e| (&e).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_update(
    state: State<'_, AppState>,
    input: UpdateJournalEntryDto,
) -> Result<ClientJournalEntryDto, String> {
    state
        .update_journal_entry
        .execute(input.into())
        .map(|e| (&e).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_delete(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), String> {
    state
        .delete_journal_entry
        .execute(JournalEntryId(id))
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn journal_list_for_client(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<Vec<ClientJournalEntryDto>, String> {
    state
        .list_client_journal
        .execute(ClientId(client_id))
        .map(|list| list.iter().map(Into::into).collect())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ClientJournalEntryDto, String> {
    state
        .get_journal_entry
        .execute(JournalEntryId(id))
        .map(|e| (&e).into())
        .map_err(to_ipc_err)
}
