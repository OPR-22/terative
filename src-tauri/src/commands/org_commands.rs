use serde::{Deserialize, Serialize};
use tauri::State;

use super::{AppState, OrgServices};
use crate::adapters::sqlite::connection::{open_org_db, OpenOrgError};
use crate::application::AppError;
use crate::domain::org::OrgCode;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OrgSummaryDto {
    /// User-supplied code — also the on-disk folder name and the picker
    /// label. Validated to `[a-z0-9_-]+` at creation.
    pub code: String,
    /// `true` once T03 lands and this org is encrypted. Always `false` in T01.
    pub has_password: bool,
    pub last_modified_at: Option<String>,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OrgInfoDto {
    pub code: String,
}

#[tauri::command]
#[specta::specta]
pub fn org_list(state: State<'_, AppState>) -> Result<Vec<OrgSummaryDto>, AppError> {
    let mut summaries = Vec::new();
    for s in state.org_registry.list()? {
        summaries.push(OrgSummaryDto {
            code: s.code.as_str().to_string(),
            has_password: false,
            last_modified_at: s.last_modified.map(|t| t.to_rfc3339()),
            file_size_bytes: s.file_size_bytes,
        });
    }
    Ok(summaries)
}

#[tauri::command]
#[specta::specta]
pub fn org_create(
    state: State<'_, AppState>,
    code: String,
) -> Result<OrgSummaryDto, AppError> {
    let parsed = OrgCode::parse(&code)?;
    let created = state.org_registry.create(parsed)?;

    let path = state.org_registry.db_path(&created);
    let meta = std::fs::metadata(&path)?;
    let last_modified_at = meta.modified().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Utc> = t.into();
        dt.to_rfc3339()
    });

    Ok(OrgSummaryDto {
        code: created.as_str().to_string(),
        has_password: false,
        last_modified_at,
        file_size_bytes: meta.len(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn org_open(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
    _password: Option<String>, // reserved for T03 encryption
) -> Result<OrgInfoDto, AppError> {
    let parsed = OrgCode::parse(&code)?;
    let db_path = state.org_registry.db_path(&parsed);

    // Bookmark webviews are bound to a per-(org, bookmark) data partition
    // at construction. Close any open ones so the next bookmark navigation
    // creates fresh webviews against the new org's storage.
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);

    let invoices_dir = state.org_registry.invoices_dir(&parsed);
    let user_backup_dir = state.org_registry.user_backup_dir(&parsed);
    let system_backup_dir = state.org_registry.system_backup_dir(&parsed);

    for d in [&invoices_dir, &user_backup_dir, &system_backup_dir] {
        std::fs::create_dir_all(d)?;
    }

    // Snapshot the on-disk file BEFORE `open_org_db` runs migrations — once
    // they've applied, we've lost the pre-migration state. The function is a
    // no-op when no migrations are pending or the file doesn't exist yet
    // (fresh org). Snapshots land in this org's `backups/system/`.
    match crate::adapters::sqlite::snapshot_pre_migration_if_pending(
        &db_path,
        &system_backup_dir,
    ) {
        Ok(Some(path)) => {
            eprintln!("pre-migration snapshot written to {}", path.display())
        }
        Ok(None) => {}
        Err(e) => {
            return Err(AppError::internal(format!(
                "pre-migration snapshot failed for org '{code}': {e}"
            )));
        }
    }

    let db = open_org_db(&db_path).map_err(|e| match e {
        OpenOrgError::NotFound => AppError::org_not_found(code.clone()),
        OpenOrgError::ForeignFile => AppError::org_not_found(code.clone()),
        OpenOrgError::Other(err) => AppError::internal(err.to_string()),
    })?;

    // Idempotent: only inserts on truly fresh orgs.
    crate::seed_default_template_if_empty(&db);
    crate::seed_default_email_templates_if_empty(&db);

    let services = OrgServices::new(
        parsed.clone(),
        db,
        db_path,
        invoices_dir,
        user_backup_dir,
        system_backup_dir,
    );
    state.open_org(services);

    Ok(OrgInfoDto {
        code: parsed.as_str().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn org_close(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);
    state.close_org();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn org_delete(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    code: String,
) -> Result<(), AppError> {
    let parsed = OrgCode::parse(&code)?;
    // If the org is currently active, close it first so the DB handle is
    // released (Windows would otherwise refuse the directory delete).
    // Tear down its bookmark webviews unconditionally — they hold open
    // file handles inside the org folder we're about to delete.
    if state
        .active_code()
        .map(|c| c.as_str() == parsed.as_str())
        .unwrap_or(false)
    {
        state.close_org();
    }
    crate::commands::bookmark_commands::close_all_bookmark_webviews(&app);
    state.org_registry.delete(&parsed)
}

#[tauri::command]
#[specta::specta]
pub fn org_get_active(state: State<'_, AppState>) -> Result<Option<OrgInfoDto>, AppError> {
    Ok(state.active_code().map(|code| OrgInfoDto {
        code: code.as_str().to_string(),
    }))
}
