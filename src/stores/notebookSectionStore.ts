import { create } from "zustand";
import { ipc, type NotebookSectionDto } from "../ipc";
import { registerOrgScopedReset } from "./orgScopedRegistry";

interface NotebookSectionState {
  sections: NotebookSectionDto[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (name: string) => Promise<NotebookSectionDto>;
  rename: (id: string, name: string) => Promise<NotebookSectionDto>;
  remove: (id: string) => Promise<void>;
  reorder: (orderedIds: string[]) => Promise<void>;
}

const INITIAL = {
  sections: [] as NotebookSectionDto[],
  loading: false,
  error: null as string | null,
};

export const useNotebookSectionStore = create<NotebookSectionState>(
  (set, get) => ({
    ...INITIAL,
    refresh: async () => {
      set({ loading: true, error: null });
      try {
        const sections = await ipc.notebookSectionList();
        set({ sections, loading: false });
      } catch (e) {
        set({ error: String(e), loading: false });
      }
    },
    create: async (name) => {
      const s = await ipc.notebookSectionCreate(name);
      await get().refresh();
      return s;
    },
    rename: async (id, name) => {
      const s = await ipc.notebookSectionRename({ id, name });
      await get().refresh();
      return s;
    },
    remove: async (id) => {
      await ipc.notebookSectionDelete(id);
      await get().refresh();
    },
    reorder: async (orderedIds) => {
      await ipc.notebookSectionReorder(orderedIds);
      await get().refresh();
    },
  }),
);

registerOrgScopedReset(() =>
  useNotebookSectionStore.setState({ ...INITIAL }),
);
