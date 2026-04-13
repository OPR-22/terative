import { create } from "zustand";
import { settingsApi } from "../api/settings";
import type {
  AppPreferences,
  CurrencyConfig,
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
}));
