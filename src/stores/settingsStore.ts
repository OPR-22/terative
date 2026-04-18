import { create } from "zustand";
import {
  ipc,
  type AppPreferencesDto,
  type EmailConfigDto,
  type SellerProfileDto,
  type SettingsSnapshotDto,
} from "../ipc";

interface SettingsState {
  snapshot: SettingsSnapshotDto | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  saveSeller: (profile: SellerProfileDto) => Promise<void>;
  saveCurrency: (code: string) => Promise<void>;
  savePreferences: (prefs: AppPreferencesDto) => Promise<void>;
  saveEmailConfig: (config: EmailConfigDto) => Promise<void>;
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
      const snapshot = await ipc.settingsGet();
      set({ snapshot, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  saveSeller: async (profile) => {
    const seller = await ipc.settingsUpdateSellerProfile(profile);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, seller } });
  },
  saveCurrency: async (code) => {
    const updated = await ipc.settingsUpdateCurrency(code);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, currency: updated } });
  },
  savePreferences: async (prefs) => {
    const preferences = await ipc.settingsUpdateAppPreferences(prefs);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, preferences } });
  },
  saveEmailConfig: async (config) => {
    const email = await ipc.settingsUpdateEmailConfig(config);
    const current = get().snapshot;
    if (current) set({ snapshot: { ...current, email } });
  },
  saveEmailPassword: async (password) => {
    await ipc.settingsUpdateEmailPassword(password);
    const current = get().snapshot;
    if (current)
      set({
        snapshot: { ...current, has_email_password: password.length > 0 },
      });
  },
  testEmailConnection: async () => {
    await ipc.emailTestConnection();
  },
}));
