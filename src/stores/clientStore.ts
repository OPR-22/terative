import { create } from "zustand";
import {
  ipc,
  type ClientDto,
  type ListClientsQueryDto,
  type NewClientDto,
  type UpdateClientDto,
} from "../ipc";

interface ClientState {
  clients: ClientDto[];
  loading: boolean;
  error: string | null;
  query: ListClientsQueryDto;
  setQuery: (q: ListClientsQueryDto) => void;
  refresh: () => Promise<void>;
  create: (input: NewClientDto) => Promise<ClientDto>;
  update: (input: UpdateClientDto) => Promise<ClientDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

export const useClientStore = create<ClientState>((set, get) => ({
  clients: [],
  loading: false,
  error: null,
  query: {},
  setQuery: (query) => {
    set({ query });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const clients = await ipc.clientList(get().query);
      set({ clients, loading: false });
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
