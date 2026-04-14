import type { Money } from "./money";

export type InvoiceStatus = "Draft" | "Finalized" | "Sent" | "Cancelled";

export interface AppliedTax {
  tax_definition_id: string | null;
  tax_name: string;
  percentage: string;
  tax_id_number: string | null;
  computed_amount: Money;
}

export interface LineItem {
  id: string;
  description: string;
  quantity: string;
  unit_price: Money;
  total: Money;
}

export interface NewLineItem {
  description: string;
  quantity: string;
  unit_price: Money;
}

export interface Invoice {
  id: string;
  number: number | null;
  client_id: string;
  template_id: string | null;
  date: string; // YYYY-MM-DD
  due_date: string | null;
  line_items: LineItem[];
  taxes_applied: AppliedTax[];
  subtotal: Money;
  tax_total: Money;
  total: Money;
  currency: string;
  status: InvoiceStatus;
  pdf_path: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewInvoice {
  client_id: string;
  template_id: string | null;
  date: string;
  due_date: string | null;
  line_items: NewLineItem[];
  tax_ids: string[];
  notes: string | null;
  currency: string;
}

export interface UpdateDraftInvoiceInput {
  id: string;
  template_id: string | null;
  date: string;
  due_date: string | null;
  line_items: NewLineItem[];
  tax_ids: string[];
  notes: string | null;
}

export interface ListInvoicesQuery {
  status?: InvoiceStatus | null;
  client_id?: string | null;
  search?: string | null;
}
