import { create } from "zustand";
import {
  ipc,
  type InvoiceDto,
  type ListInvoicesQueryDto,
  type NewInvoiceDto,
  type UpdateDraftInvoiceDto,
} from "../ipc";

interface InvoiceState {
  invoices: InvoiceDto[];
  loading: boolean;
  error: string | null;
  query: ListInvoicesQueryDto;
  setQuery: (q: ListInvoicesQueryDto) => void;
  refresh: () => Promise<void>;
  get: (id: string) => Promise<InvoiceDto>;
  createDraft: (input: NewInvoiceDto) => Promise<InvoiceDto>;
  updateDraft: (input: UpdateDraftInvoiceDto) => Promise<InvoiceDto>;
  finalize: (id: string) => Promise<InvoiceDto>;
  duplicate: (id: string) => Promise<InvoiceDto>;
  cancel: (id: string) => Promise<InvoiceDto>;
  send: (id: string) => Promise<InvoiceDto>;
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
      const invoices = await ipc.invoiceList(get().query);
      set({ invoices, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  get: (id) => ipc.invoiceGet(id),
  createDraft: async (input) => {
    const i = await ipc.invoiceCreateDraft(input);
    await get().refresh();
    return i;
  },
  updateDraft: async (input) => {
    const i = await ipc.invoiceUpdateDraft(input);
    await get().refresh();
    return i;
  },
  finalize: async (id) => {
    const i = await ipc.invoiceFinalize(id);
    await get().refresh();
    return i;
  },
  duplicate: async (id) => {
    const i = await ipc.invoiceDuplicate(id);
    await get().refresh();
    return i;
  },
  cancel: async (id) => {
    const i = await ipc.invoiceCancel(id);
    await get().refresh();
    return i;
  },
  send: async (id) => {
    const i = await ipc.invoiceSend(id);
    await get().refresh();
    return i;
  },
}));
