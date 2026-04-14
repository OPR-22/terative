import { invoke } from "@tauri-apps/api/core";
import type {
  AppPreferences,
  CurrencyConfig,
  EmailConfig,
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
  updateEmailConfig: (config: EmailConfig) =>
    invoke<EmailConfig>("settings_update_email_config", { config }),
  updateEmailPassword: (password: string) =>
    invoke<void>("settings_update_email_password", { password }),
  testEmailConnection: () => invoke<void>("email_test_connection"),
};
