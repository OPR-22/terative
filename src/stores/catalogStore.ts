import { create } from "zustand";
import {
  ipc,
  type CatalogItemDto,
  type NewCatalogItemDto,
  type UpdateCatalogItemDto,
} from "../ipc";
import { registerOrgScopedReset } from "./orgScopedRegistry";

interface CatalogState {
  items: CatalogItemDto[];
  loading: boolean;
  error: string | null;
  includeArchived: boolean;
  setIncludeArchived: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewCatalogItemDto) => Promise<CatalogItemDto>;
  update: (input: UpdateCatalogItemDto) => Promise<CatalogItemDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

const INITIAL = {
  items: [] as CatalogItemDto[],
  loading: false,
  error: null as string | null,
  includeArchived: false,
};

export const useCatalogStore = create<CatalogState>((set, get) => ({
  ...INITIAL,
  setIncludeArchived: (includeArchived) => {
    set({ includeArchived });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const items = await ipc.catalogItemList(get().includeArchived);
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

registerOrgScopedReset(() => useCatalogStore.setState({ ...INITIAL }));
