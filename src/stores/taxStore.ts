import { create } from "zustand";
import { taxesApi } from "../api/taxes";
import type { NewTaxDefinition, TaxDefinition, UpdateTaxInput } from "../types/tax";

interface TaxState {
  taxes: TaxDefinition[];
  includeInactive: boolean;
  loading: boolean;
  error: string | null;
  setIncludeInactive: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewTaxDefinition) => Promise<TaxDefinition>;
  update: (input: UpdateTaxInput) => Promise<TaxDefinition>;
  remove: (id: string) => Promise<void>;
}

export const useTaxStore = create<TaxState>((set, get) => ({
  taxes: [],
  includeInactive: false,
  loading: false,
  error: null,
  setIncludeInactive: (includeInactive) => {
    set({ includeInactive });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const taxes = await taxesApi.list(get().includeInactive);
      set({ taxes, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const tax = await taxesApi.create(input);
    await get().refresh();
    return tax;
  },
  update: async (input) => {
    const tax = await taxesApi.update(input);
    await get().refresh();
    return tax;
  },
  remove: async (id) => {
    await taxesApi.delete(id);
    await get().refresh();
  },
}));
