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
  /// Full, non-paginated index of clients keyed by id. Used by every
  /// page that displays a client *name* for a foreign-key id (invoice
  /// list, payment list, dashboard, etc.) — those views must resolve
  /// every id, including ids that fall outside the current ClientList
  /// page or the active search filter, so we keep this lookup separate
  /// from the paginated `clients` array.
  directory: Record<string, ClientDto>;
  directoryLoaded: boolean;
  /// Distinct values currently used by existing clients, for autocomplete
  /// suggestions on free-form attribute fields. Refreshed on every list
  /// refresh, create, and update so newly-typed values become suggestions
  /// for the next client immediately.
  attributeValues: ClientAttributeValuesDto;
  setQuery: (q: ListClientsQueryDto) => void;
  setPage: (page: number) => void;
  setPerPage: (perPage: number) => void;
  refresh: () => Promise<void>;
  /// Loads all clients (archived included) into `directory`. Cheap to
  /// call repeatedly — only the first call hits IPC unless `force` is set.
  ensureDirectory: (force?: boolean) => Promise<void>;
  /// Resolves a client id to its display name. Returns the raw id if the
  /// directory hasn't loaded yet or the client doesn't exist (defensive —
  /// shouldn't happen but better than crashing).
  clientName: (id: string) => string;
  refreshAttributeValues: () => Promise<void>;
  create: (input: NewClientDto) => Promise<ClientDto>;
  update: (input: UpdateClientDto) => Promise<ClientDto>;
  archive: (id: string) => Promise<void>;
  unarchive: (id: string) => Promise<void>;
}

/// `per_page` for the directory fetch. Chosen large enough that any
/// realistic single-tenant dataset fits in one round trip; revisit if we
/// ever ship multi-tenant or B2B accounts with >10k clients.
const DIRECTORY_PAGE_SIZE = 10_000;

export const useClientStore = create<ClientState>((set, get) => ({
  clients: [],
  page: null,
  loading: false,
  error: null,
  query: {},
  currentPage: 1,
  perPage: 25,
  directory: {},
  directoryLoaded: false,
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
      // The list page changes whenever clients are created/archived/etc;
      // refresh the directory too so name lookups elsewhere stay current.
      void get().ensureDirectory(true);
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  ensureDirectory: async (force = false) => {
    if (!force && get().directoryLoaded) return;
    try {
      const result = await ipc.clientList({
        include_archived: true,
        pagination: { page: 1, per_page: DIRECTORY_PAGE_SIZE },
      });
      const directory: Record<string, ClientDto> = {};
      for (const c of result.data) directory[c.id] = c;
      set({ directory, directoryLoaded: true });
    } catch {
      // Lookups will fall back to raw ids until the next refresh.
    }
  },
  clientName: (id) => get().directory[id]?.name ?? id,
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
