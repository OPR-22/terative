import { create } from "zustand";

import { ipc } from "../ipc";

/// Layout constants for the sidebar. React owns these; Rust receives them via
/// `set_sidebar_width` and uses them to position bookmark webviews on Linux.
export const SIDEBAR_WIDTH_EXPANDED = 224; // matches Tailwind `w-56`
export const SIDEBAR_WIDTH_COLLAPSED = 64; // matches `w-16`

const STORAGE_KEY = "terative.sidebar.collapsed";

function readInitial(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

interface SidebarState {
  collapsed: boolean;
  toggle: () => void;
}

export const useSidebarStore = create<SidebarState>((set, get) => ({
  collapsed: readInitial(),
  toggle: () => {
    const next = !get().collapsed;
    set({ collapsed: next });
    try {
      localStorage.setItem(STORAGE_KEY, next ? "1" : "0");
    } catch {
      // noop — storage unavailable, runtime state is still correct
    }
    void ipc.setSidebarWidth(
      next ? SIDEBAR_WIDTH_COLLAPSED : SIDEBAR_WIDTH_EXPANDED,
    );
  },
}));

export function currentSidebarWidth(): number {
  return useSidebarStore.getState().collapsed
    ? SIDEBAR_WIDTH_COLLAPSED
    : SIDEBAR_WIDTH_EXPANDED;
}
