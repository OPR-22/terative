import { invoke } from "@tauri-apps/api/core";
import type {
  Invoice,
  ListInvoicesQuery,
  NewInvoice,
  UpdateDraftInvoiceInput,
} from "../types/invoice";

export const invoicesApi = {
  list: (query: ListInvoicesQuery = {}) =>
    invoke<Invoice[]>("invoice_list", { query }),
  get: (id: string) => invoke<Invoice>("invoice_get", { id }),
  createDraft: (input: NewInvoice) =>
    invoke<Invoice>("invoice_create_draft", { input }),
  updateDraft: (input: UpdateDraftInvoiceInput) =>
    invoke<Invoice>("invoice_update_draft", { input }),
  finalize: (id: string) => invoke<Invoice>("invoice_finalize", { id }),
  duplicate: (id: string) => invoke<Invoice>("invoice_duplicate", { id }),
  cancel: (id: string) => invoke<Invoice>("invoice_cancel", { id }),
  send: (id: string) => invoke<Invoice>("invoice_send", { id }),
};
