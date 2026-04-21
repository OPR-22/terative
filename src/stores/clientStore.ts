import { create } from "zustand";
import {
  ipc,
  type ClientDto,
  type ListClientsQueryDto,
  type NewClientDto,
  type PageDto,
  type UpdateClientDto,
} from "../ipc";

interface ClientState {
  clients: ClientDto[];
  page: PageDto<ClientDto> | null;
  loading: boolean;
  error: string | null;
  query: ListClientsQueryDto;
  currentPage: number;
  setQuery: (q: ListClientsQueryDto) => void;
  setPage: (page: number) => void;
  refresh: () => Promise<void>;
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
  setQuery: (query) => {
    set({ query, currentPage: 1 });
    void get().refresh();
  },
  setPage: (currentPage) => {
    set({ currentPage });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const result = await ipc.clientList({
        ...get().query,
        pagination: { page: get().currentPage },
      });
      set({
        clients: result.data,
        page: result,
        currentPage: result.next ? result.next - 1 : result.last,
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
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
