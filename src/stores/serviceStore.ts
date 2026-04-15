import { create } from "zustand";
import {
  ipc,
  type NewServiceDto,
  type ServiceDto,
  type UpdateServiceDto,
} from "../ipc";

interface ServiceState {
  services: ServiceDto[];
  loading: boolean;
  error: string | null;
  includeInactive: boolean;
  setIncludeInactive: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewServiceDto) => Promise<ServiceDto>;
  update: (input: UpdateServiceDto) => Promise<ServiceDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

export const useServiceStore = create<ServiceState>((set, get) => ({
  services: [],
  loading: false,
  error: null,
  includeInactive: false,
  setIncludeInactive: (includeInactive) => {
    set({ includeInactive });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const services = await ipc.serviceList(get().includeInactive);
      set({ services, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const service = await ipc.serviceCreate(input);
    await get().refresh();
    return service;
  },
  update: async (input) => {
    const service = await ipc.serviceUpdate(input);
    await get().refresh();
    return service;
  },
  archive: async (id) => {
    await ipc.serviceArchive(id);
    await get().refresh();
  },
  unarchive: async (id) => {
    await ipc.serviceUnarchive(id);
    await get().refresh();
  },
}));
