import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import i18next from "i18next";

import { toast } from "../stores/toastStore";

/**
 * Check the updater endpoint configured in `tauri.conf.json → plugins.updater`.
 *
 * **Silent mode (`{ silent: true }`)** — used by the startup check in
 * `App.tsx`. No toast on "up to date"; errors are swallowed (the user did not
 * ask, so we don't nag if GitHub is down).
 *
 * **Loud mode (`{ silent: false }`)** — used by the manual "Check for updates"
 * button in Settings. Toasts on every outcome including success.
 *
 * When an update *is* available, both modes raise a persistent toast with an
 * "Install & restart" action that runs `downloadAndInstall()` then
 * `relaunch()`. The action is what actually performs the install — we never
 * install automatically. This is deliberate: the app handles invoicing and
 * we don't want to restart in the middle of editing one.
 */
export async function checkForUpdates(opts: { silent: boolean }): Promise<void> {
  const t = i18next.t.bind(i18next);
  let update: Update | null;
  try {
    update = await check();
  } catch (err) {
    // Common causes: no network, endpoint 404 (placeholder pubkey/URL during
    // local dev before the releases repo exists), CSP. Loud only if asked.
    if (!opts.silent) {
      toast.error(t("updates.check_failed"), describeError(err));
    } else {
      // eslint-disable-next-line no-console
      console.warn("[updater] silent check failed:", err);
    }
    return;
  }

  if (!update) {
    if (!opts.silent) toast.success(t("updates.up_to_date"));
    return;
  }

  toast.info(t("updates.available"), update.version, {
    persistent: true,
    action: {
      label: t("updates.install_and_restart"),
      onClick: () => {
        void runInstall(update!);
      },
    },
  });
}

async function runInstall(update: Update): Promise<void> {
  const t = i18next.t.bind(i18next);
  try {
    toast.neutral(t("updates.downloading"));
    // Progress callback is per-chunk; we deliberately don't surface every tick
    // — too noisy for a toast. A future enhancement could pipe this to a
    // dedicated progress UI.
    await update.downloadAndInstall(() => {});
    // On Linux (AppImage) `downloadAndInstall` replaces the AppImage in place
    // and the relaunch is required. On macOS/Windows the installer takes over;
    // `relaunch` is harmless if the OS already restarted the process.
    await relaunch();
  } catch (err) {
    toast.error(t("updates.install_failed"), describeError(err));
  }
}

function describeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
