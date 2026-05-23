/**
 * Registry of reset functions for org-scoped Zustand stores.
 *
 * Zustand stores are module-level singletons that survive React remounts —
 * keying the route on `activeOrg.code` re-mounts components, but the
 * stores themselves keep their cached data from the previous org. Without
 * explicit resets, switching orgs would show stale clients/invoices/
 * settings until each page's useEffect re-fetches.
 *
 * Each org-scoped store calls `registerOrgScopedReset(...)` at module
 * load time. `orgStore.open` / `orgStore.close` invoke `resetAll()` to
 * wipe every store back to its initial state in a single sweep.
 *
 * Stores that hold app-wide data (sidebar collapse state, toasts,
 * currency catalog) DO NOT register — they outlive org switches.
 */

const resetFns = new Set<() => void>();

export function registerOrgScopedReset(fn: () => void): void {
  resetFns.add(fn);
}

export function resetAllOrgScopedStores(): void {
  for (const fn of resetFns) fn();
}
