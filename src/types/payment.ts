import type { Money } from "./money";

export type PaymentMethodKind =
  | "BankTransfer"
  | "Cash"
  | "Check"
  | "Card"
  | "Other";

export type PaymentMethod =
  | { kind: "BankTransfer" }
  | { kind: "Cash" }
  | { kind: "Check" }
  | { kind: "Card" }
  | { kind: "Other"; detail: string };

export interface PaymentAllocation {
  invoice_id: string;
  amount: Money;
}

export interface Payment {
  id: string;
  client_id: string;
  date: string;
  amount: Money;
  method: PaymentMethod;
  reference: string | null;
  allocations: PaymentAllocation[];
  notes: string | null;
  created_at: string;
}

export interface NewPaymentAllocation {
  invoice_id: string;
  amount: Money;
}

export interface NewPayment {
  client_id: string;
  date: string;
  amount: Money;
  method: PaymentMethod;
  reference: string | null;
  allocations: NewPaymentAllocation[];
  notes: string | null;
}

export interface UpdatePaymentInput {
  id: string;
  date: string;
  amount: Money;
  method: PaymentMethod;
  reference: string | null;
  allocations: NewPaymentAllocation[];
  notes: string | null;
}

export interface ListPaymentsQuery {
  client_id?: string | null;
  search?: string | null;
}

export function paymentMethodLabel(method: PaymentMethod): string {
  switch (method.kind) {
    case "BankTransfer":
      return "Bank transfer";
    case "Cash":
      return "Cash";
    case "Check":
      return "Check";
    case "Card":
      return "Card";
    case "Other":
      return method.detail || "Other";
  }
}
