import { invoke } from "@tauri-apps/api/core";
import type {
  ListPaymentsQuery,
  NewPayment,
  Payment,
  UpdatePaymentInput,
} from "../types/payment";

export const paymentsApi = {
  list: (query: ListPaymentsQuery = {}) =>
    invoke<Payment[]>("payment_list", { query }),
  get: (id: string) => invoke<Payment>("payment_get", { id }),
  record: (input: NewPayment) => invoke<Payment>("payment_record", { input }),
  update: (input: UpdatePaymentInput) =>
    invoke<Payment>("payment_update", { input }),
  delete: (id: string) => invoke<void>("payment_delete", { id }),
};
