import { create } from "zustand";
import { notebookApi } from "../api/notebook";
import type { NotebookSection } from "../types/notebook";

interface NotebookSectionState {
  sections: NotebookSection[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (name: string) => Promise<NotebookSection>;
  rename: (id: string, name: string) => Promise<NotebookSection>;
  remove: (id: string) => Promise<void>;
  reorder: (orderedIds: string[]) => Promise<void>;
}

export const useNotebookSectionStore = create<NotebookSectionState>(
  (set, get) => ({
    sections: [],
    loading: false,
    error: null,
    refresh: async () => {
      set({ loading: true, error: null });
      try {
        const sections = await notebookApi.listSections();
        set({ sections, loading: false });
      } catch (e) {
        set({ error: String(e), loading: false });
      }
    },
    create: async (name) => {
      const s = await notebookApi.createSection(name);
      await get().refresh();
      return s;
    },
    rename: async (id, name) => {
      const s = await notebookApi.renameSection({ id, name });
      await get().refresh();
      return s;
    },
    remove: async (id) => {
      await notebookApi.deleteSection(id);
      await get().refresh();
    },
    reorder: async (orderedIds) => {
      await notebookApi.reorderSections(orderedIds);
      await get().refresh();
    },
  }),
);
