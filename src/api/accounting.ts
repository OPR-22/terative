import { invoke } from "@tauri-apps/api/core";
import type {
  AgingRow,
  ClientBalance,
  DashboardSummary,
  InvoicePaymentRow,
  RevenueBucket,
  RevenueByClient,
  RevenueByClientInput,
  RevenueByPeriodInput,
} from "../types/accounting";

export const accountingApi = {
  listOutstanding: () =>
    invoke<InvoicePaymentRow[]>("accounting_list_outstanding"),
  listOverdue: () => invoke<InvoicePaymentRow[]>("accounting_list_overdue"),
  revenueByPeriod: (input: RevenueByPeriodInput) =>
    invoke<RevenueBucket[]>("accounting_revenue_by_period", { input }),
  revenueByClient: (input: RevenueByClientInput) =>
    invoke<RevenueByClient[]>("accounting_revenue_by_client", { input }),
  clientBalance: (clientId: string) =>
    invoke<ClientBalance>("accounting_client_balance", { clientId }),
  clientBalances: () =>
    invoke<ClientBalance[]>("accounting_client_balances"),
  agingReport: () => invoke<AgingRow[]>("accounting_aging_report"),
  dashboardSummary: () =>
    invoke<DashboardSummary>("accounting_dashboard_summary"),
};
