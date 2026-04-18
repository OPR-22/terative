use tauri::State;

use crate::application::dto::{
    AppPreferencesDto, CurrencyConfigDto, SellerProfileDto, SettingsSnapshotDto,
};
use crate::application::AppError;
use crate::domain::money::Currency;
use crate::domain::settings::CurrencyConfig;

use super::{to_ipc_err, AppState};

#[tauri::command]
#[specta::specta]
pub fn settings_get(state: State<'_, AppState>) -> Result<SettingsSnapshotDto, String> {
    state
        .get_settings
        .execute()
        .map(|s| (&s).into())
        .map_err(to_ipc_err)
}

#[tauri::command]
#[specta::specta]
pub fn settings_update_seller_profile(
    state: State<'_, AppState>,
    profile: SellerProfileDto,
) -> Result<SellerProfileDto, String> {
    state
        .update_seller_profile
        .execute(profile.into())
        .map(|p| (&p).into())
        .map_err(to_ipc_err)
}

/// Updates the app's display currency by ISO 4217 code. The full metadata
/// (symbol, fraction digits, unit names) is derived server-side and returned
/// in the response so the frontend can keep its cache fresh.
#[tauri::command]
#[specta::specta]
pub fn settings_update_currency(
    state: State<'_, AppState>,
    code: String,
) -> Result<CurrencyConfigDto, String> {
    let config = CurrencyConfig::from_code(&code).map_err(|e| to_ipc_err(AppError::Currency(e)))?;
    state
        .update_currency
        .execute(config)
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
}

/// Returns the full list of supported currencies with their display metadata.
/// The frontend calls this once at boot to populate its catalog for amount
/// formatting and the Settings currency dropdown.
#[tauri::command]
#[specta::specta]
pub fn settings_supported_currencies() -> Result<Vec<CurrencyConfigDto>, String> {
    Ok(Currency::all().iter().copied().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn settings_update_app_preferences(
    state: State<'_, AppState>,
    preferences: AppPreferencesDto,
) -> Result<AppPreferencesDto, String> {
    state
        .update_app_preferences
        .execute(preferences.into())
        .map(|p| (&p).into())
        .map_err(to_ipc_err)
}
