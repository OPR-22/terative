import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import type { ThemeDto } from "../ipc";

function applyTheme(theme: ThemeDto) {
  const value = theme === "Dark" ? "dark" : "light";
  document.documentElement.setAttribute("data-theme", value);
}

/**
 * Mounts once at the app root, applies the active theme from the settings
 * snapshot to the <html> element whenever it changes. Also loads settings
 * on first mount if they haven't been loaded yet.
 */
export function useTheme() {
  const snapshot = useSettingsStore((s) => s.snapshot);
  const load = useSettingsStore((s) => s.load);

  useEffect(() => {
    if (!snapshot) {
      void load();
    }
  }, [snapshot, load]);

  useEffect(() => {
    if (snapshot) {
      applyTheme(snapshot.preferences.theme);
    }
  }, [snapshot]);
}
