import { create } from "zustand";
import {
  ipc,
  type InvoiceDto,
  type ListInvoicesQueryDto,
  type NewInvoiceDto,
  type PageDto,
  type UpdateDraftInvoiceDto,
} from "../ipc";

interface InvoiceState {
  invoices: InvoiceDto[];
  page: PageDto<InvoiceDto> | null;
  loading: boolean;
  error: string | null;
  query: ListInvoicesQueryDto;
  currentPage: number;
  setQuery: (q: ListInvoicesQueryDto) => void;
  setPage: (page: number) => void;
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
      const result = await ipc.invoiceList({
        ...get().query,
        pagination: { page: get().currentPage },
      });
      set({
        invoices: result.data,
        page: result,
        currentPage: result.next ? result.next - 1 : result.last,
        loading: false,
      });
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
