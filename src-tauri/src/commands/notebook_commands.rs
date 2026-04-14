use tauri::State;

use crate::application::notebook_usecases::{
    ClientNotebookView, RenameNotebookSectionInput, SaveClientNotebookInput,
    UpdateJournalEntryInput,
};
use crate::domain::client::ClientId;
use crate::domain::notebook::{
    ClientJournalEntry, JournalEntryId, NewJournalEntry, NotebookSection, NotebookSectionId,
};

use super::{to_ipc_err, AppState};

// ---- section management (Settings) ----

#[tauri::command]
pub fn notebook_section_create(
    state: State<'_, AppState>,
    name: String,
) -> Result<NotebookSection, String> {
    state
        .create_notebook_section
        .execute(name)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn notebook_section_rename(
    state: State<'_, AppState>,
    input: RenameNotebookSectionInput,
) -> Result<NotebookSection, String> {
    state
        .rename_notebook_section
        .execute(input)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn notebook_section_delete(
    state: State<'_, AppState>,
    id: NotebookSectionId,
) -> Result<(), String> {
    state
        .delete_notebook_section
        .execute(id)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn notebook_section_count_entries(
    state: State<'_, AppState>,
    id: NotebookSectionId,
) -> Result<u64, String> {
    state.count_section_entries.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn notebook_section_reorder(
    state: State<'_, AppState>,
    ordered_ids: Vec<NotebookSectionId>,
) -> Result<(), String> {
    state
        .reorder_notebook_sections
        .execute(ordered_ids)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn notebook_section_list(
    state: State<'_, AppState>,
) -> Result<Vec<NotebookSection>, String> {
    state.list_notebook_sections.execute().map_err(to_ipc_err)
}

// ---- client notebook ----

#[tauri::command]
pub fn client_notebook_get(
    state: State<'_, AppState>,
    client_id: ClientId,
) -> Result<ClientNotebookView, String> {
    state
        .get_client_notebook
        .execute(client_id)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn client_notebook_save(
    state: State<'_, AppState>,
    input: SaveClientNotebookInput,
) -> Result<(), String> {
    state.save_client_notebook.execute(input).map_err(to_ipc_err)
}

// ---- journal ----

#[tauri::command]
pub fn journal_entry_create(
    state: State<'_, AppState>,
    input: NewJournalEntry,
) -> Result<ClientJournalEntry, String> {
    state.create_journal_entry.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn journal_entry_update(
    state: State<'_, AppState>,
    input: UpdateJournalEntryInput,
) -> Result<ClientJournalEntry, String> {
    state.update_journal_entry.execute(input).map_err(to_ipc_err)
}

#[tauri::command]
pub fn journal_entry_delete(
    state: State<'_, AppState>,
    id: JournalEntryId,
) -> Result<(), String> {
    state.delete_journal_entry.execute(id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn journal_list_for_client(
    state: State<'_, AppState>,
    client_id: ClientId,
) -> Result<Vec<ClientJournalEntry>, String> {
    state.list_client_journal.execute(client_id).map_err(to_ipc_err)
}

#[tauri::command]
pub fn journal_entry_get(
    state: State<'_, AppState>,
    id: JournalEntryId,
) -> Result<ClientJournalEntry, String> {
    state.get_journal_entry.execute(id).map_err(to_ipc_err)
}
