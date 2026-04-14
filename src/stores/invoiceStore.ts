import { create } from "zustand";
import { invoicesApi } from "../api/invoices";
import type {
  Invoice,
  ListInvoicesQuery,
  NewInvoice,
  UpdateDraftInvoiceInput,
} from "../types/invoice";

interface InvoiceState {
  invoices: Invoice[];
  loading: boolean;
  error: string | null;
  query: ListInvoicesQuery;
  setQuery: (q: ListInvoicesQuery) => void;
  refresh: () => Promise<void>;
  get: (id: string) => Promise<Invoice>;
  createDraft: (input: NewInvoice) => Promise<Invoice>;
  updateDraft: (input: UpdateDraftInvoiceInput) => Promise<Invoice>;
  finalize: (id: string) => Promise<Invoice>;
  duplicate: (id: string) => Promise<Invoice>;
  cancel: (id: string) => Promise<Invoice>;
  send: (id: string) => Promise<Invoice>;
}

export const useInvoiceStore = create<InvoiceState>((set, get) => ({
  invoices: [],
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
      const invoices = await invoicesApi.list(get().query);
      set({ invoices, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  get: (id) => invoicesApi.get(id),
  createDraft: async (input) => {
    const i = await invoicesApi.createDraft(input);
    await get().refresh();
    return i;
  },
  updateDraft: async (input) => {
    const i = await invoicesApi.updateDraft(input);
    await get().refresh();
    return i;
  },
  finalize: async (id) => {
    const i = await invoicesApi.finalize(id);
    await get().refresh();
    return i;
  },
  duplicate: async (id) => {
    const i = await invoicesApi.duplicate(id);
    await get().refresh();
    return i;
  },
  cancel: async (id) => {
    const i = await invoicesApi.cancel(id);
    await get().refresh();
    return i;
  },
  send: async (id) => {
    const i = await invoicesApi.send(id);
    await get().refresh();
    return i;
  },
}));
