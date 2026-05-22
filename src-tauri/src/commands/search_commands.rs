use tauri::State;

use crate::application::dto::SearchHitDto;
use crate::application::AppError;

use super::AppState;

/// Global full-text search across clients, invoices and catalog items
/// (T1.07). Backs the ⌘K search palette. A blank or punctuation-only
/// `query` yields an empty list.
#[tauri::command]
#[specta::specta]
pub fn global_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchHitDto>, AppError> {
    state
        .org()?
        .global_search
        .execute(&query)
        .map(|hits| hits.iter().map(SearchHitDto::from).collect())
}
