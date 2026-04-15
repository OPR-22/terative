import { create } from "zustand";
import {
  ipc,
  type NewTaxDefinitionDto,
  type TaxDefinitionDto,
  type UpdateTaxDto,
} from "../ipc";

interface TaxState {
  taxes: TaxDefinitionDto[];
  includeInactive: boolean;
  loading: boolean;
  error: string | null;
  setIncludeInactive: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewTaxDefinitionDto) => Promise<TaxDefinitionDto>;
  update: (input: UpdateTaxDto) => Promise<TaxDefinitionDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
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
      const taxes = await ipc.taxList(get().includeInactive);
      set({ taxes, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const tax = await ipc.taxCreate(input);
    await get().refresh();
    return tax;
  },
  update: async (input) => {
    const tax = await ipc.taxUpdate(input);
    await get().refresh();
    return tax;
  },
  archive: async (id) => {
    await ipc.taxArchive(id);
    await get().refresh();
  },
  unarchive: async (id) => {
    await ipc.taxUnarchive(id);
    await get().refresh();
  },
}));
