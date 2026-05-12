use tauri::State;
use uuid::Uuid;

use crate::application::dto::{
    ClientJournalEntryDto, ClientNotebookViewDto, NewJournalEntryDto, NotebookSectionDto,
    RenameNotebookSectionDto, SaveClientNotebookDto, UpdateJournalEntryDto,
};
use crate::domain::client::ClientId;
use crate::domain::notebook::{JournalEntryId, NotebookSectionId};

use super::AppState;
use crate::application::AppError;

// ---- section management (Settings) ----

#[tauri::command]
#[specta::specta]
pub fn notebook_section_create(
    state: State<'_, AppState>,
    name: String,
) -> Result<NotebookSectionDto, AppError> {
    state.org()?
        .create_notebook_section
        .execute(name)
        .map(|s| (&s).into())
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_rename(
    state: State<'_, AppState>,
    input: RenameNotebookSectionDto,
) -> Result<NotebookSectionDto, AppError> {
    state.org()?
        .rename_notebook_section
        .execute(input.into())
        .map(|s| (&s).into())
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_delete(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), AppError> {
    state.org()?
        .delete_notebook_section
        .execute(NotebookSectionId(id))
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_count_entries(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<u64, AppError> {
    state.org()?
        .count_section_entries
        .execute(NotebookSectionId(id))
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_reorder(
    state: State<'_, AppState>,
    ordered_ids: Vec<Uuid>,
) -> Result<(), AppError> {
    let ids: Vec<NotebookSectionId> = ordered_ids.into_iter().map(NotebookSectionId).collect();
    state.org()?
        .reorder_notebook_sections
        .execute(ids)
}

#[tauri::command]
#[specta::specta]
pub fn notebook_section_list(
    state: State<'_, AppState>,
) -> Result<Vec<NotebookSectionDto>, AppError> {
    state.org()?
        .list_notebook_sections
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
}

// ---- client notebook ----

#[tauri::command]
#[specta::specta]
pub fn client_notebook_get(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<ClientNotebookViewDto, AppError> {
    state.org()?
        .get_client_notebook
        .execute(ClientId(client_id))
        .map(|v| (&v).into())
}

#[tauri::command]
#[specta::specta]
pub fn client_notebook_save(
    state: State<'_, AppState>,
    input: SaveClientNotebookDto,
) -> Result<(), AppError> {
    state.org()?
        .save_client_notebook
        .execute(input.into())
}

// ---- journal ----

#[tauri::command]
#[specta::specta]
pub fn journal_entry_create(
    state: State<'_, AppState>,
    input: NewJournalEntryDto,
) -> Result<ClientJournalEntryDto, AppError> {
    state.org()?
        .create_journal_entry
        .execute(input.into())
        .map(|e| (&e).into())
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_update(
    state: State<'_, AppState>,
    input: UpdateJournalEntryDto,
) -> Result<ClientJournalEntryDto, AppError> {
    state.org()?
        .update_journal_entry
        .execute(input.into())
        .map(|e| (&e).into())
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_delete(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), AppError> {
    state.org()?
        .delete_journal_entry
        .execute(JournalEntryId(id))
}

#[tauri::command]
#[specta::specta]
pub fn journal_list_for_client(
    state: State<'_, AppState>,
    client_id: Uuid,
) -> Result<Vec<ClientJournalEntryDto>, AppError> {
    state.org()?
        .list_client_journal
        .execute(ClientId(client_id))
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn journal_entry_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ClientJournalEntryDto, AppError> {
    state.org()?
        .get_journal_entry
        .execute(JournalEntryId(id))
        .map(|e| (&e).into())
}
