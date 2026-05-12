use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;
use tauri::State;

use super::AppState;
use crate::application::AppError;
use crate::application::ports::{BackupKind, BackupMetadata, BackupScope};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "PascalCase")]
pub enum BackupKindDto {
    Manual,
    Auto,
    PreRestore,
    PreMigration,
}

impl From<BackupKind> for BackupKindDto {
    fn from(value: BackupKind) -> Self {
        match value {
            BackupKind::Manual => BackupKindDto::Manual,
            BackupKind::Auto => BackupKindDto::Auto,
            BackupKind::PreRestore => BackupKindDto::PreRestore,
            BackupKind::PreMigration => BackupKindDto::PreMigration,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "PascalCase")]
pub enum BackupScopeDto {
    User,
    System,
}

impl From<BackupScope> for BackupScopeDto {
    fn from(value: BackupScope) -> Self {
        match value {
            BackupScope::User => BackupScopeDto::User,
            BackupScope::System => BackupScopeDto::System,
        }
    }
}

#[derive(Debug, Serialize, Type)]
pub struct BackupDto {
    pub path: String,
    pub timestamp: DateTime<Utc>,
    pub kind: BackupKindDto,
    pub scope: BackupScopeDto,
    pub size_bytes: u64,
}

impl From<BackupMetadata> for BackupDto {
    fn from(value: BackupMetadata) -> Self {
        BackupDto {
            path: value.path.to_string_lossy().to_string(),
            timestamp: value.timestamp,
            scope: value.kind.scope().into(),
            kind: value.kind.into(),
            size_bytes: value.size_bytes,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn data_export(
    state: State<'_, AppState>,
    destination: String,
) -> Result<String, AppError> {
    state.org()?
        .data_management
        .export_database(&PathBuf::from(destination))
        .map(|p| p.to_string_lossy().to_string())
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_backup(state: State<'_, AppState>) -> Result<String, AppError> {
    state.org()?
        .data_management
        .create_backup()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_restore(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    source: String,
) -> Result<(), AppError> {
    state.org()?
        .data_management
        .restore_database(&PathBuf::from(source))
        .map_err(AppError::from)?;
    // Restart so the live DB connection is closed and the newly-swapped file
    // is opened cleanly by a fresh process. `restart` never returns.
    app.restart()
}

#[tauri::command]
#[specta::specta]
pub fn data_list_backups(state: State<'_, AppState>) -> Result<Vec<BackupDto>, AppError> {
    state.org()?
        .data_management
        .list_backups()
        .map(|items| items.into_iter().map(BackupDto::from).collect())
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_delete_backup(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    state.org()?
        .data_management
        .delete_backup(&PathBuf::from(path))
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_user_backup_dir(state: State<'_, AppState>) -> Result<String, AppError> {
    Ok(state.org()?.user_backup_dir.to_string_lossy().to_string())
}
