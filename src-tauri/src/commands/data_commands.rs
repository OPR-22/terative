use std::path::PathBuf;

use tauri::State;

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn data_export(
    state: State<'_, AppState>,
    destination: String,
) -> Result<String, String> {
    state
        .data_management
        .export_database(&PathBuf::from(destination))
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| to_ipc_err(crate::application::AppError::Repo(e)))
}

#[tauri::command]
pub fn data_backup(
    state: State<'_, AppState>,
    backup_dir: Option<String>,
) -> Result<String, String> {
    let dir = backup_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| state.default_backup_dir.clone());
    state
        .data_management
        .create_backup(&dir)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| to_ipc_err(crate::application::AppError::Repo(e)))
}

#[tauri::command]
pub fn data_restore(
    state: State<'_, AppState>,
    source: String,
) -> Result<(), String> {
    state
        .data_management
        .restore_database(&PathBuf::from(source))
        .map_err(|e| to_ipc_err(crate::application::AppError::Repo(e)))
}

#[tauri::command]
pub fn data_default_backup_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.default_backup_dir.to_string_lossy().to_string())
}
