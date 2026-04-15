use tauri::State;

use crate::application::dto::{
    AppPreferencesDto, CurrencyConfigDto, SellerProfileDto, SettingsSnapshotDto,
};

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

#[tauri::command]
#[specta::specta]
pub fn settings_update_currency(
    state: State<'_, AppState>,
    currency: CurrencyConfigDto,
) -> Result<CurrencyConfigDto, String> {
    state
        .update_currency
        .execute(currency.into())
        .map(|c| (&c).into())
        .map_err(to_ipc_err)
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
