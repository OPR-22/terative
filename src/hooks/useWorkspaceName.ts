import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useSettingsStore } from "../stores/settingsStore";

/**
 * Returns the user's workspace display name — the seller's `name` from
 * settings — used as the first breadcrumb segment everywhere and in the
 * sidebar footer. Falls back to a translated placeholder until settings
 * are loaded so we never render an empty string.
 *
 * Loads settings on first call so consumers don't each have to remember.
 */
export function useWorkspaceName(): string {
  const { t } = useTranslation();
  const snapshot = useSettingsStore((s) => s.snapshot);
  const load = useSettingsStore((s) => s.load);

  useEffect(() => {
    if (!snapshot) void load();
  }, [snapshot, load]);

  const name = snapshot?.seller.name?.trim();
  return name && name.length > 0 ? name : t("app.workspace_fallback");
}
