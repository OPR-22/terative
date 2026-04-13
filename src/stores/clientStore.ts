import { create } from "zustand";
import { clientsApi } from "../api/clients";
import type {
  Client,
  ListClientsQuery,
  NewClient,
  UpdateClientInput,
} from "../types/client";

interface ClientState {
  clients: Client[];
  loading: boolean;
  error: string | null;
  query: ListClientsQuery;
  setQuery: (q: ListClientsQuery) => void;
  refresh: () => Promise<void>;
  create: (input: NewClient) => Promise<Client>;
  update: (input: UpdateClientInput) => Promise<Client>;
  remove: (id: string) => Promise<void>;
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
      const clients = await clientsApi.list(get().query);
      set({ clients, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const client = await clientsApi.create(input);
    await get().refresh();
    return client;
  },
  update: async (input) => {
    const client = await clientsApi.update(input);
    await get().refresh();
    return client;
  },
  remove: async (id) => {
    await clientsApi.delete(id);
    await get().refresh();
  },
}));
