import { invoke } from "@tauri-apps/api/core";
import type {
  AppPreferences,
  CurrencyConfig,
  SellerProfile,
  SettingsSnapshot,
} from "../types/settings";

export const settingsApi = {
  get: () => invoke<SettingsSnapshot>("settings_get"),
  updateSellerProfile: (profile: SellerProfile) =>
    invoke<SellerProfile>("settings_update_seller_profile", { profile }),
  updateCurrency: (currency: CurrencyConfig) =>
    invoke<CurrencyConfig>("settings_update_currency", { currency }),
  updateAppPreferences: (preferences: AppPreferences) =>
    invoke<AppPreferences>("settings_update_app_preferences", { preferences }),
};
