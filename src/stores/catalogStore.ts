import { create } from "zustand";
import {
  ipc,
  type CatalogItemDto,
  type NewCatalogItemDto,
  type UpdateCatalogItemDto,
} from "../ipc";

interface CatalogState {
  items: CatalogItemDto[];
  loading: boolean;
  error: string | null;
  includeInactive: boolean;
  setIncludeInactive: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewCatalogItemDto) => Promise<CatalogItemDto>;
  update: (input: UpdateCatalogItemDto) => Promise<CatalogItemDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

export const useCatalogStore = create<CatalogState>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  includeInactive: false,
  setIncludeInactive: (includeInactive) => {
    set({ includeInactive });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const items = await ipc.catalogItemList(get().includeInactive);
      set({ items, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const item = await ipc.catalogItemCreate(input);
    await get().refresh();
    return item;
  },
  update: async (input) => {
    const item = await ipc.catalogItemUpdate(input);
    await get().refresh();
    return item;
  },
  archive: async (id) => {
    await ipc.catalogItemArchive(id);
    await get().refresh();
  },
  unarchive: async (id) => {
    await ipc.catalogItemUnarchive(id);
    await get().refresh();
  },
}));
