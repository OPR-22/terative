import { create } from "zustand";
import {
  ipc,
  type ListPaymentsQueryDto,
  type NewPaymentDto,
  type PaymentDto,
  type UpdatePaymentDto,
} from "../ipc";

interface PaymentState {
  payments: PaymentDto[];
  loading: boolean;
  error: string | null;
  query: ListPaymentsQueryDto;
  setQuery: (q: ListPaymentsQueryDto) => void;
  refresh: () => Promise<void>;
  record: (input: NewPaymentDto) => Promise<PaymentDto>;
  update: (input: UpdatePaymentDto) => Promise<PaymentDto>;
  remove: (id: string) => Promise<void>;
}

export const usePaymentStore = create<PaymentState>((set, get) => ({
  payments: [],
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
      const payments = await ipc.paymentList(get().query);
      set({ payments, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  record: async (input) => {
    const p = await ipc.paymentRecord(input);
    await get().refresh();
    return p;
  },
  update: async (input) => {
    const p = await ipc.paymentUpdate(input);
    await get().refresh();
    return p;
  },
  remove: async (id) => {
    await ipc.paymentDelete(id);
    await get().refresh();
  },
}));
