import { create } from "zustand";

import {
  ipc,
  type BookmarkDto,
  type NewBookmarkDto,
  type UpdateBookmarkDto,
} from "../ipc";

interface BookmarkState {
  bookmarks: BookmarkDto[];
  loaded: boolean;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  /// Refresh on first call; subsequent calls are no-ops.
  ensureLoaded: () => Promise<void>;
  byId: (id: string) => BookmarkDto | undefined;
  create: (input: NewBookmarkDto) => Promise<BookmarkDto>;
  update: (input: UpdateBookmarkDto) => Promise<BookmarkDto>;
  remove: (id: string) => Promise<void>;
  reorder: (orderedIds: string[]) => Promise<void>;
}

export const useBookmarkStore = create<BookmarkState>((set, get) => ({
  bookmarks: [],
  loaded: false,
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const bookmarks = await ipc.bookmarkList();
      set({ bookmarks, loaded: true, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  ensureLoaded: async () => {
    if (get().loaded || get().loading) return;
    await get().refresh();
  },
  byId: (id) => get().bookmarks.find((b) => b.id === id),
  create: async (input) => {
    const b = await ipc.bookmarkCreate(input);
    await get().refresh();
    return b;
  },
  update: async (input) => {
    const b = await ipc.bookmarkUpdate(input);
    await get().refresh();
    return b;
  },
  remove: async (id) => {
    await ipc.bookmarkDelete(id);
    await get().refresh();
  },
  reorder: async (orderedIds) => {
    await ipc.bookmarkReorder(orderedIds);
    await get().refresh();
  },
}));
