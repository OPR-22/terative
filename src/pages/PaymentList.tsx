import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { MoneyInput } from "../components/common/MoneyInput";
import { useMoneyFormat } from "../lib/money";
import {
  ipc,
  type CurrencyConfigDto,
  type InvoicePaymentRowDto,
  type NewPaymentAllocationDto,
  type NewPaymentDto,
  type PaymentDto,
  type PaymentMethodDto,
  type UpdatePaymentDto,
} from "../ipc";
import { usePaymentStore } from "../stores/paymentStore";
import { useClientStore } from "../stores/clientStore";
import { useSettingsStore } from "../stores/settingsStore";

type PaymentMethodKind = PaymentMethodDto["kind"];

function paymentMethodLabel(method: PaymentMethodDto): string {
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

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; payment: PaymentDto };

const METHOD_KINDS: PaymentMethodKind[] = [
  "BankTransfer",
  "Cash",
  "Check",
  "Card",
  "Other",
];

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

export function PaymentList() {
  const { t } = useTranslation();
  const { payments, loading, error, refresh, remove } = usePaymentStore();
  const { clients, refresh: refreshClients } = useClientStore();
  const { snapshot, load } = useSettingsStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
    void refreshClients();
    if (!snapshot) void load();
  }, [refresh, refreshClients, load, snapshot]);

  const { formatMinor } = useMoneyFormat();
  const currency = snapshot?.currency;
  const currencyCode = snapshot?.currency.code ?? "EUR";
  const clientName = (id: string) =>
    clients.find((c) => c.id === id)?.name ?? id;

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("payments.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("payments.new")}
        </Button>
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : payments.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("payments.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("common.date")}</th>
              <th className="py-2 pr-3 font-medium">{t("payments.client")}</th>
              <th className="py-2 pr-3 font-medium">{t("payments.method")}</th>
              <th className="py-2 pr-3 font-medium">{t("payments.reference")}</th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("common.price")}
              </th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("payments.allocated")}
              </th>
              <th className="py-2 pr-3"></th>
            </tr>
          </thead>
          <tbody>
            {payments.map((p) => {
              const allocated = p.allocations.reduce(
                (sum, a) => sum + a.amount.amount_minor,
                0,
              );
              return (
                <tr key={p.id} className="border-b border-border">
                  <td className="py-2 pr-3 text-fg-muted">{p.date}</td>
                  <td className="py-2 pr-3 text-fg">{clientName(p.client_id)}</td>
                  <td className="py-2 pr-3 text-fg-muted">
                    {paymentMethodLabel(p.method)}
                  </td>
                  <td className="py-2 pr-3 text-fg-muted">
                    {p.reference ?? "—"}
                  </td>
                  <td className="py-2 pr-3 text-right font-medium text-fg">
                    {formatMinor(p.amount.amount_minor, p.amount.currency)}
                  </td>
                  <td className="py-2 pr-3 text-right text-fg-muted">
                    {formatMinor(allocated, currencyCode)}
                  </td>
                  <td className="flex justify-end gap-2 py-2 pr-3">
                    <Button
                      variant="secondary"
                      onClick={() => setEditor({ mode: "edit", payment: p })}
                    >
                      {t("common.edit")}
                    </Button>
                    <Button
                      variant="danger"
                      onClick={() => {
                        if (confirm(t("common.confirm_delete"))) {
                          void remove(p.id);
                        }
                      }}
                    >
                      {t("common.delete")}
                    </Button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {editor.mode !== "closed" && currency ? (
        <PaymentEditor
          key={editor.mode === "edit" ? editor.payment.id : "new"}
          initial={editor.mode === "edit" ? editor.payment : null}
          currencyCode={currencyCode}
          currency={currency}
          onClose={() => setEditor({ mode: "closed" })}
        />
      ) : null}
    </div>
  );
}

interface EditorProps {
  initial: PaymentDto | null;
  currencyCode: string;
  currency: CurrencyConfigDto;
  onClose: () => void;
}

function PaymentEditor({
  initial,
  currencyCode,
  currency,
  onClose,
}: EditorProps) {
  const { t } = useTranslation();
  const { formatMinor } = useMoneyFormat();
  const { record, update } = usePaymentStore();
  const { clients } = useClientStore();

  const [clientId, setClientId] = useState(initial?.client_id ?? "");
  const [date, setDate] = useState(initial?.date ?? today());
  const [amountCents, setAmountCents] = useState(initial?.amount.amount_minor ?? 0);
  const [methodKind, setMethodKind] = useState<PaymentMethodKind>(
    initial?.method.kind ?? "BankTransfer",
  );
  const [methodDetail, setMethodDetail] = useState(
    initial?.method.kind === "Other" ? initial.method.detail : "",
  );
  const [reference, setReference] = useState(initial?.reference ?? "");
  const [notes, setNotes] = useState(initial?.notes ?? "");
  // Map of invoice_id -> allocation cents
  const [allocations, setAllocations] = useState<Record<string, number>>(() => {
    if (!initial) return {};
    const m: Record<string, number> = {};
    for (const a of initial.allocations) m[a.invoice_id] = a.amount.amount_minor;
    return m;
  });
  const [outstanding, setOutstanding] = useState<InvoicePaymentRowDto[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingListOutstanding()
      .then((rows) => {
        if (!cancelled) setOutstanding(rows);
      })
      .catch((e) => {
        if (!cancelled) setErr(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const clientOutstanding = useMemo(
    () => outstanding.filter((r) => r.client_id === clientId),
    [outstanding, clientId],
  );

  const allocatedTotal = Object.values(allocations).reduce(
    (sum, cents) => sum + cents,
    0,
  );
  const unallocated = amountCents - allocatedTotal;

  const toggleAllocation = (row: InvoicePaymentRowDto) => {
    setAllocations((prev) => {
      const next = { ...prev };
      if (row.invoice_id in next) {
        delete next[row.invoice_id];
      } else {
        // default to whichever is smaller: remaining unallocated or invoice due
        const remaining = Math.max(0, amountCents - allocatedTotal);
        next[row.invoice_id] = Math.min(remaining, row.amount_due.amount_minor);
      }
      return next;
    });
  };

  const updateAllocation = (invoiceId: string, cents: number) => {
    setAllocations((prev) => ({ ...prev, [invoiceId]: cents }));
  };

  const buildMethod = (): PaymentMethodDto => {
    switch (methodKind) {
      case "BankTransfer":
        return { kind: "BankTransfer" };
      case "Cash":
        return { kind: "Cash" };
      case "Check":
        return { kind: "Check" };
      case "Card":
        return { kind: "Card" };
      case "Other":
        return { kind: "Other", detail: methodDetail };
    }
  };

  const buildAllocations = (): NewPaymentAllocationDto[] =>
    Object.entries(allocations)
      .filter(([, cents]) => cents > 0)
      .map(([invoice_id, cents]) => ({
        invoice_id,
        amount: { amount_minor: cents, currency: currencyCode },
      }));

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setSubmitting(true);
    try {
      if (!clientId) throw new Error(t("invoices.err_no_client"));
      if (amountCents <= 0) throw new Error(t("payments.err_non_positive"));
      if (methodKind === "Other" && methodDetail.trim() === "")
        throw new Error(t("payments.err_method_detail"));
      if (unallocated < 0) throw new Error(t("payments.err_over_allocated"));

      if (initial) {
        const payload: UpdatePaymentDto = {
          id: initial.id,
          date,
          amount: { amount_minor: amountCents, currency: currencyCode },
          method: buildMethod(),
          reference: reference || null,
          allocations: buildAllocations(),
          notes: notes || null,
        };
        await update(payload);
      } else {
        const payload: NewPaymentDto = {
          client_id: clientId,
          date,
          amount: { amount_minor: amountCents, currency: currencyCode },
          method: buildMethod(),
          reference: reference || null,
          allocations: buildAllocations(),
          notes: notes || null,
        };
        await record(payload);
      }
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-10 flex items-start justify-center overflow-y-auto bg-overlay p-4">
      <form
        className="my-8 w-full max-w-2xl rounded-card bg-surface p-6 shadow-card"
        onSubmit={submit}
      >
        <h2 className="mb-4 text-lg font-bold text-fg">
          {initial ? t("payments.edit") : t("payments.new")}
        </h2>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("invoices.client")}
            <select
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm disabled:opacity-60"
              value={clientId}
              onChange={(e) => {
                setClientId(e.target.value);
                setAllocations({});
              }}
              disabled={initial !== null}
            >
              <option value="">—</option>
              {clients.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>
          <Input
            type="date"
            label={t("common.date") ?? ""}
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
          <MoneyInput
            label={t("payments.amount") ?? ""}
            valueMinor={amountCents}
            currency={currency}
            onChangeMinor={setAmountCents}
          />
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("payments.method")}
            <select
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={methodKind}
              onChange={(e) => setMethodKind(e.target.value as PaymentMethodKind)}
            >
              {METHOD_KINDS.map((k) => (
                <option key={k} value={k}>
                  {t(`payments.method_${k.toLowerCase()}`)}
                </option>
              ))}
            </select>
          </label>
          {methodKind === "Other" ? (
            <Input
              label={t("payments.method_detail") ?? ""}
              value={methodDetail}
              onChange={(e) => setMethodDetail(e.target.value)}
              className="sm:col-span-2"
            />
          ) : null}
          <Input
            label={t("payments.reference") ?? ""}
            value={reference}
            onChange={(e) => setReference(e.target.value)}
          />
          <Input
            label={t("common.notes") ?? ""}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
          />
        </div>

        <div className="mt-5 border-t border-border pt-4">
          <h3 className="mb-2 text-sm font-semibold text-fg-muted">
            {t("payments.allocations")}
          </h3>
          {!clientId ? (
            <p className="text-xs text-fg-subtle">
              {t("payments.select_client_first")}
            </p>
          ) : clientOutstanding.length === 0 ? (
            <p className="text-xs text-fg-subtle">
              {t("payments.no_outstanding")}
            </p>
          ) : (
            <div className="flex flex-col gap-2">
              {clientOutstanding.map((row) => {
                const checked = row.invoice_id in allocations;
                const cents = allocations[row.invoice_id] ?? 0;
                return (
                  <div
                    key={row.invoice_id}
                    className="grid grid-cols-12 items-end gap-2 rounded-field border border-border p-2"
                  >
                    <label className="col-span-6 flex items-center gap-2 text-sm text-fg-muted">
                      <input
                        type="checkbox"
                        checked={checked}
                        onChange={() => toggleAllocation(row)}
                      />
                      <span className="font-medium text-fg">
                        #{row.number ?? "—"}
                      </span>
                      <span className="text-xs text-fg-subtle">
                        {t("accounting.amount_due")}:{" "}
                        {formatMinor(
                          row.amount_due.amount_minor,
                          row.amount_due.currency,
                        )}
                      </span>
                    </label>
                    <div className="col-span-6">
                      {checked ? (
                        <MoneyInput
                          label={t("payments.allocate_amount") ?? ""}
                          valueMinor={cents}
                          currency={currency}
                          onChangeMinor={(c) =>
                            updateAllocation(row.invoice_id, c)
                          }
                        />
                      ) : null}
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          <div className="mt-3 flex justify-between text-sm">
            <span className="text-fg-muted">
              {t("payments.allocated")}: {formatMinor(allocatedTotal, currencyCode)}
            </span>
            <span
              className={
                unallocated < 0
                  ? "text-danger font-semibold"
                  : "text-fg-muted"
              }
            >
              {t("payments.unallocated")}: {formatMinor(unallocated, currencyCode)}
            </span>
          </div>
        </div>

        {err ? <p className="mt-3 text-sm text-danger">{err}</p> : null}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={submitting || unallocated < 0}>
            {t("common.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
