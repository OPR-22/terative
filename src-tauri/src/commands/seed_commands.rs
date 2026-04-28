//! Dev-only Tauri command surface for the database seeder. The whole
//! file is included only in debug builds via the `#[cfg]`-gated `pub
//! mod seed_commands;` line in `commands/mod.rs`, and the command
//! itself is added to the registered command list only in the debug
//! arm of `build_specta()`'s helper macro.

use tauri::State;

use crate::application::dto::{SeedCountsDto, SeedReportDto};

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn seed_database(
    state: State<'_, AppState>,
    counts: Option<SeedCountsDto>,
) -> Result<SeedReportDto, String> {
    state
        .seed_database
        .execute(counts.unwrap_or_default().into())
        .map(Into::into)
        .map_err(to_ipc_err)
}
