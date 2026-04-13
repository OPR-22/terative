import { create } from "zustand";
import { servicesApi } from "../api/services";
import type {
  NewService,
  Service,
  UpdateServiceInput,
} from "../types/service";

interface ServiceState {
  services: Service[];
  loading: boolean;
  error: string | null;
  includeInactive: boolean;
  setIncludeInactive: (v: boolean) => void;
  refresh: () => Promise<void>;
  create: (input: NewService) => Promise<Service>;
  update: (input: UpdateServiceInput) => Promise<Service>;
  remove: (id: string) => Promise<void>;
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
      const services = await servicesApi.list(get().includeInactive);
      set({ services, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const service = await servicesApi.create(input);
    await get().refresh();
    return service;
  },
  update: async (input) => {
    const service = await servicesApi.update(input);
    await get().refresh();
    return service;
  },
  remove: async (id) => {
    await servicesApi.delete(id);
    await get().refresh();
  },
}));
