import { useEffect } from "react";

import { ipc } from "../ipc";
import { currentSidebarWidth } from "../stores/sidebarStore";

/// Layout constants for the bookmark windowing system. React owns these as
/// the single source of truth; the Rust side stores whatever React reports
/// and refuses to open a bookmark before the bootstrap has run.
export const TOOLBAR_HEIGHT = 50;

/// Reports initial layout dimensions to the Rust side on app mount.
/// Called once from `App.tsx`. Subsequent changes (e.g. sidebar collapse,
/// toolbar re-measure) call the IPC commands directly from where the
/// change originates.
export function useBookmarksLayoutBootstrap(): void {
  useEffect(() => {
    void ipc.bookmarkLayoutSetSidebarWidth(currentSidebarWidth());
    void ipc.bookmarkLayoutSetToolbarHeight(TOOLBAR_HEIGHT);
  }, []);
}
