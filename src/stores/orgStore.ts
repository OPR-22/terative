import { create } from "zustand";
import { ipc, type OrgInfoDto, type OrgSummaryDto } from "../ipc";
import { resetAllOrgScopedStores } from "./orgScopedRegistry";

interface OrgState {
  /** Currently open org, or `null` when the picker should be shown. */
  activeOrg: OrgInfoDto | null;
  /** All orgs found on disk. Refreshed on picker mount and after create/delete. */
  orgs: OrgSummaryDto[];
  loading: boolean;
  /** True until the initial `org_get_active` resolves on app boot. */
  initializing: boolean;
  error: string | null;

  /** Run once on app mount to determine whether to show picker or shell. */
  bootstrap: () => Promise<void>;
  refresh: () => Promise<void>;
  create: (code: string) => Promise<OrgSummaryDto>;
  open: (code: string, password?: string) => Promise<void>;
  close: () => Promise<void>;
  delete: (code: string) => Promise<void>;
}

export const useOrgStore = create<OrgState>((set, get) => ({
  activeOrg: null,
  orgs: [],
  loading: false,
  initializing: true,
  error: null,

  bootstrap: async () => {
    set({ initializing: true, error: null });
    try {
      const active = await ipc.orgGetActive();
      const orgs = await ipc.orgList();
      set({ activeOrg: active, orgs, initializing: false });
    } catch (e) {
      set({ initializing: false, error: e instanceof Error ? e.message : String(e) });
    }
  },

  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const orgs = await ipc.orgList();
      set({ orgs, loading: false });
    } catch (e) {
      set({ loading: false, error: e instanceof Error ? e.message : String(e) });
    }
  },

  create: async (code: string) => {
    const summary = await ipc.orgCreate(code);
    set({ orgs: [...get().orgs, summary] });
    return summary;
  },

  open: async (code: string, password?: string) => {
    const info = await ipc.orgOpen(code, password ?? null);
    // Wipe every per-org store BEFORE flipping activeOrg so React mounts
    // don't briefly read stale data from the previous org while the new
    // org's `load()` calls are still in flight.
    resetAllOrgScopedStores();
    set({ activeOrg: info });
  },

  close: async () => {
    await ipc.orgClose();
    resetAllOrgScopedStores();
    set({ activeOrg: null });
  },

  delete: async (code: string) => {
    await ipc.orgDelete(code);
    set({
      orgs: get().orgs.filter((o) => o.code !== code),
      activeOrg: get().activeOrg?.code === code ? null : get().activeOrg,
    });
  },
}));
