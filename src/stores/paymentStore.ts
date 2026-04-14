import { create } from "zustand";
import { paymentsApi } from "../api/payments";
import type {
  ListPaymentsQuery,
  NewPayment,
  Payment,
  UpdatePaymentInput,
} from "../types/payment";

interface PaymentState {
  payments: Payment[];
  loading: boolean;
  error: string | null;
  query: ListPaymentsQuery;
  setQuery: (q: ListPaymentsQuery) => void;
  refresh: () => Promise<void>;
  record: (input: NewPayment) => Promise<Payment>;
  update: (input: UpdatePaymentInput) => Promise<Payment>;
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
      const payments = await paymentsApi.list(get().query);
      set({ payments, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  record: async (input) => {
    const p = await paymentsApi.record(input);
    await get().refresh();
    return p;
  },
  update: async (input) => {
    const p = await paymentsApi.update(input);
    await get().refresh();
    return p;
  },
  remove: async (id) => {
    await paymentsApi.delete(id);
    await get().refresh();
  },
}));
