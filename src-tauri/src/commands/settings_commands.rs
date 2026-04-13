use tauri::State;

use crate::application::settings_usecases::SettingsSnapshot;
use crate::domain::settings::{AppPreferences, CurrencyConfig, SellerProfile};

use super::{to_ipc_err, AppState};

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Result<SettingsSnapshot, String> {
    state.get_settings.execute().map_err(to_ipc_err)
}

#[tauri::command]
pub fn settings_update_seller_profile(
    state: State<'_, AppState>,
    profile: SellerProfile,
) -> Result<SellerProfile, String> {
    state
        .update_seller_profile
        .execute(profile)
        .map_err(to_ipc_err)
}

#[tauri::command]
pub fn settings_update_currency(
    state: State<'_, AppState>,
    currency: CurrencyConfig,
) -> Result<CurrencyConfig, String> {
    state.update_currency.execute(currency).map_err(to_ipc_err)
}

#[tauri::command]
pub fn settings_update_app_preferences(
    state: State<'_, AppState>,
    preferences: AppPreferences,
) -> Result<AppPreferences, String> {
    state
        .update_app_preferences
        .execute(preferences)
        .map_err(to_ipc_err)
}
