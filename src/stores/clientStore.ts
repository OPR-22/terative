import { create } from "zustand";
import {
  ipc,
  type ClientAttributeValuesDto,
  type ClientDto,
  type ListClientsQueryDto,
  type NewClientDto,
  type PageDto,
  type UpdateClientDto,
} from "../ipc";

const emptyAttributeValues: ClientAttributeValuesDto = {
  gender: [],
  pronouns: [],
  occupation: [],
};

interface ClientState {
  clients: ClientDto[];
  page: PageDto<ClientDto> | null;
  loading: boolean;
  error: string | null;
  query: ListClientsQueryDto;
  currentPage: number;
  perPage: number;
  /// Distinct values currently used by existing clients, for autocomplete
  /// suggestions on free-form attribute fields. Refreshed on every list
  /// refresh, create, and update so newly-typed values become suggestions
  /// for the next client immediately.
  attributeValues: ClientAttributeValuesDto;
  setQuery: (q: ListClientsQueryDto) => void;
  setPage: (page: number) => void;
  setPerPage: (perPage: number) => void;
  refresh: () => Promise<void>;
  refreshAttributeValues: () => Promise<void>;
  create: (input: NewClientDto) => Promise<ClientDto>;
  update: (input: UpdateClientDto) => Promise<ClientDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

export const useClientStore = create<ClientState>((set, get) => ({
  clients: [],
  page: null,
  loading: false,
  error: null,
  query: {},
  currentPage: 1,
  perPage: 25,
  attributeValues: emptyAttributeValues,
  setQuery: (query) => {
    set({ query, currentPage: 1 });
    void get().refresh();
  },
  setPage: (currentPage) => {
    set({ currentPage });
    void get().refresh();
  },
  setPerPage: (perPage) => {
    set({ perPage, currentPage: 1 });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const result = await ipc.clientList({
        ...get().query,
        pagination: { page: get().currentPage, per_page: get().perPage },
      });
      set({
        clients: result.data,
        page: result,
        currentPage: result.next ? result.next - 1 : result.last,
        loading: false,
      });
      void get().refreshAttributeValues();
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  refreshAttributeValues: async () => {
    try {
      const attributeValues = await ipc.clientAttributeValues();
      set({ attributeValues });
    } catch {
      // Suggestions are non-critical; swallow errors so the form still works.
    }
  },
  create: async (input) => {
    const client = await ipc.clientCreate(input);
    await get().refresh();
    return client;
  },
  update: async (input) => {
    const client = await ipc.clientUpdate(input);
    await get().refresh();
    return client;
  },
  archive: async (id) => {
    await ipc.clientArchive(id);
    await get().refresh();
  },
  unarchive: async (id) => {
    await ipc.clientUnarchive(id);
    await get().refresh();
  },
}));
