import { create } from "zustand";
import { settingsApi } from "../api/settings";
import type {
  AppPreferences,
  CurrencyConfig,
  EmailConfig,
  SellerProfile,
  SettingsSnapshot,
} from "../types/settings";

interface SettingsState {
  snapshot: SettingsSnapshot | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  saveSeller: (profile: SellerProfile) => Promise<void>;
  saveCurrency: (currency: CurrencyConfig) => Promise<void>;
  savePreferences: (prefs: AppPreferences) => Promise<void>;
  saveEmailConfig: (config: EmailConfig) => Promise<void>;
  saveEmailPassword: (password: string) => Promise<void>;
  testEmailConnection: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  snapshot: null,
  loading: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const snapshot = await settingsApi.get();
      set({ snapshot, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  saveSeller: async (profile) => {
    const seller = await settingsApi.updateSellerProfile(profile);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, seller } });
  },
  saveCurrency: async (currency) => {
    const updated = await settingsApi.updateCurrency(currency);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, currency: updated } });
  },
  savePreferences: async (prefs) => {
    const preferences = await settingsApi.updateAppPreferences(prefs);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, preferences } });
  },
  saveEmailConfig: async (config) => {
    const email = await settingsApi.updateEmailConfig(config);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, email } });
  },
  saveEmailPassword: async (password) => {
    await settingsApi.updateEmailPassword(password);
    const current = get().snapshot;
    if (current)
      set({
        snapshot: { ...current, has_email_password: password.length > 0 },
      });
  },
  testEmailConnection: async () => {
    await settingsApi.testEmailConnection();
  },
}));
