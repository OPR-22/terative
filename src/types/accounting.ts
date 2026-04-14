import type { Money } from "./money";
import type { InvoiceStatus } from "./invoice";

export type DerivedPaymentStatus =
  | "Draft"
  | "Unpaid"
  | "Partial"
  | "Paid"
  | "Overdue"
  | "Cancelled";

export interface InvoicePaymentRow {
  invoice_id: string;
  number: number | null;
  client_id: string;
  client_name: string;
  date: string;
  due_date: string | null;
  total: Money;
  amount_paid: Money;
  amount_due: Money;
  status: InvoiceStatus;
  payment_status: DerivedPaymentStatus;
}

export type RevenueGrouping = "Day" | "Month" | "Year";

export interface RevenueBucket {
  bucket_start: string;
  amount: Money;
  invoice_count: number;
}

export interface RevenueByClient {
  client_id: string;
  client_name: string;
  total_invoiced: Money;
  invoice_count: number;
}

export interface ClientBalance {
  client_id: string;
  client_name: string;
  total_invoiced: Money;
  total_paid: Money;
  outstanding: Money;
}

export type AgingBucket =
  | "Current"
  | "Days1To30"
  | "Days31To60"
  | "Days61To90"
  | "Days91Plus";

export interface AgingRow {
  invoice_id: string;
  number: number | null;
  client_id: string;
  client_name: string;
  total: Money;
  amount_due: Money;
  due_date: string | null;
  bucket: AgingBucket;
}

export interface DashboardSummary {
  revenue_this_year: Money;
  outstanding_total: Money;
  overdue_count: number;
  draft_count: number;
  finalized_count: number;
  sent_count: number;
}

export interface RevenueByPeriodInput {
  start: string;
  end: string;
  grouping: RevenueGrouping;
}

export interface RevenueByClientInput {
  start: string;
  end: string;
}
