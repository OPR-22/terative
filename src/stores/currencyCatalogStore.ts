import { create } from "zustand";

import { ipc, type CurrencyConfigDto } from "../ipc";

interface CatalogState {
  /** The full list of supported currencies, loaded from the backend once. */
  all: CurrencyConfigDto[];
  loading: boolean;
  error: string | null;
  /** Fetch the list. Idempotent: re-calling while loaded is a no-op. */
  load: () => Promise<void>;
  /**
   * Look up a currency's metadata by ISO code. Returns `undefined` if the
   * catalog hasn't loaded yet or the code is unknown.
   */
  byCode: (code: string) => CurrencyConfigDto | undefined;
}

export const useCurrencyCatalogStore = create<CatalogState>((set, get) => ({
  all: [],
  loading: false,
  error: null,
  load: async () => {
    if (get().all.length > 0 || get().loading) return;
    set({ loading: true, error: null });
    try {
      const all = await ipc.settingsSupportedCurrencies();
      set({ all, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  byCode: (code) => get().all.find((c) => c.code === code),
}));
