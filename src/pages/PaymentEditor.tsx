import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { Field, Input, Select, Textarea } from "../components/ui/Input";
import { MoneyInput } from "../components/common/MoneyInput";
import {
  ipc,
  type InvoicePaymentRowDto,
  type NewPaymentAllocationDto,
  type NewPaymentDto,
  type PaymentMethodDto,
  type UpdatePaymentDto,
} from "../ipc";
import { useMoneyFormat } from "../lib/money";
import { useClientStore } from "../stores/clientStore";
import { useCurrencyCatalogStore } from "../stores/currencyCatalogStore";
import { usePaymentStore } from "../stores/paymentStore";
import { useSettingsStore } from "../stores/settingsStore";

type PaymentMethodKind = PaymentMethodDto["kind"];
const METHOD_KINDS: PaymentMethodKind[] = ["BankTransfer", "Cash", "Check", "Card", "Other"];

const today = () => new Date().toISOString().slice(0, 10);

export function PaymentEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { payments, refresh, record, update } = usePaymentStore();
  const { clients, refresh: refreshClients } = useClientStore();
  const { snapshot, load } = useSettingsStore();
  const {
    all: currencyCatalog,
    load: loadCurrencyCatalog,
    byCode,
  } = useCurrencyCatalogStore();
  const { formatMinor } = useMoneyFormat();

  useEffect(() => {
    if (payments.length === 0) void refresh();
    if (clients.length === 0) void refreshClients();
    if (!snapshot) void load();
    if (currencyCatalog.length === 0) void loadCurrencyCatalog();
  }, [
    payments.length,
    clients.length,
    snapshot,
    currencyCatalog.length,
    refresh,
    refreshClients,
    load,
    loadCurrencyCatalog,
  ]);

  const existing = useMemo(() => payments.find((p) => p.id === id), [payments, id]);

  const [currencyCode, setCurrencyCode] = useState<string>("");
  // Seed from the org default once the snapshot loads, the existing payment's
  // currency on edit, and let the client picker overwrite it below.
  useEffect(() => {
    if (currencyCode) return;
    if (existing) setCurrencyCode(existing.amount.currency.code);
    else if (snapshot) setCurrencyCode(snapshot.currency.code);
  }, [existing, snapshot, currencyCode]);

  const currency = byCode(currencyCode);

  const [clientId, setClientId] = useState("");
  const [date, setDate] = useState(today());
  const [amountCents, setAmountCents] = useState(0);
  const [methodKind, setMethodKind] = useState<PaymentMethodKind>("BankTransfer");
  const [methodDetail, setMethodDetail] = useState("");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");
  const [allocations, setAllocations] = useState<Record<string, number>>({});
  const [outstanding, setOutstanding] = useState<InvoicePaymentRowDto[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!existing) return;
    setClientId(existing.client_id);
    setDate(existing.date);
    setAmountCents(existing.amount.amount);
    setMethodKind(existing.method.kind);
    setMethodDetail(existing.method.kind === "Other" ? existing.method.detail : "");
    setReference(existing.reference ?? "");
    setNotes(existing.notes ?? "");
    const m: Record<string, number> = {};
    for (const a of existing.allocations) m[a.invoice_id] = a.amount.amount;
    setAllocations(m);
  }, [existing]);

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingListOutstanding()
      .then((rows) => {
        if (!cancelled) setOutstanding(rows);
      })
      .catch((e) => {
        if (!cancelled) toast.error(e);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const clientOutstanding = useMemo(
    () => outstanding.filter((r) => r.client_id === clientId),
    [outstanding, clientId],
  );

  // Split the client's open invoices by whether they match the payment's
  // currency. Strict silos means we can only allocate against same-currency
  // invoices; the rest get summarised below the table.
  const clientOutstandingInCurrency = useMemo(
    () =>
      clientOutstanding.filter(
        (r) => r.amount_due.currency.code === currencyCode,
      ),
    [clientOutstanding, currencyCode],
  );
  const otherCurrencyCount = clientOutstanding.length - clientOutstandingInCurrency.length;

  // When a client is selected (and this isn't an edit of an existing
  // payment), default the payment's currency to the client's default —
  // mirrors the invoice editor behaviour. Switching currency invalidates
  // any allocations carried over, so we reset them.
  const seededClientCurrencyRef = useRef<string | null>(null);
  useEffect(() => {
    if (existing) return;
    if (!clientId) return;
    if (seededClientCurrencyRef.current === clientId) return;
    const client = clients.find((c) => c.id === clientId);
    if (!client) return;
    seededClientCurrencyRef.current = clientId;
    if (client.default_currency === currencyCode) return;
    setCurrencyCode(client.default_currency);
    setAllocations({});
  }, [clientId, clients, existing, currencyCode]);

  const allocatedTotal = Object.values(allocations).reduce((sum, c) => sum + c, 0);
  const unallocated = amountCents - allocatedTotal;

  const toggleAllocation = (row: InvoicePaymentRowDto) => {
    setAllocations((prev) => {
      const next = { ...prev };
      if (row.invoice_id in next) {
        delete next[row.invoice_id];
      } else {
        const remaining = Math.max(0, amountCents - allocatedTotal);
        next[row.invoice_id] = Math.min(remaining, row.amount_due.amount);
      }
      return next;
    });
  };

  const updateAllocation = (invoiceId: string, cents: number) =>
    setAllocations((prev) => ({ ...prev, [invoiceId]: cents }));

  const buildMethod = (): PaymentMethodDto => {
    switch (methodKind) {
      case "Other":
        return { kind: "Other", detail: methodDetail };
      default:
        return { kind: methodKind };
    }
  };

  const buildAllocations = (
    currency: NonNullable<typeof snapshot>["currency"],
  ): NewPaymentAllocationDto[] =>
    Object.entries(allocations)
      .filter(([, c]) => c > 0)
      .map(([invoice_id, c]) => ({
        invoice_id,
        amount: { amount: c, currency },
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
      if (!currency) throw new Error(t("payments.err_currency_unknown"));

      if (editing && existing) {
        const payload: UpdatePaymentDto = {
          id: existing.id,
          date,
          amount: { amount: amountCents, currency },
          method: buildMethod(),
          reference: reference || null,
          allocations: buildAllocations(currency),
          notes: notes || null,
        };
        await update(payload);
      } else {
        const payload: NewPaymentDto = {
          client_id: clientId,
          date,
          amount: { amount: amountCents, currency },
          method: buildMethod(),
          reference: reference || null,
          allocations: buildAllocations(currency),
          notes: notes || null,
        };
        await record(payload);
      }
      navigate("/payments");
    } catch (e) {
      toast.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Page
      crumbs={[
        { label: t("payments.title"), to: "/payments" },
        editing ? t("payments.edit") : t("payments.new"),
      ]}
      title={editing ? t("payments.edit") : t("payments.new")}
    >
      <form onSubmit={submit} className="max-w-3xl">
        <Card>
          <CardHead title={t("common.details")} />
          <CardBody>
            <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-3">
              <Field label={t("invoices.client")}>
                <Select
                  value={clientId}
                  onChange={(e) => {
                    setClientId(e.target.value);
                    setAllocations({});
                  }}
                  disabled={editing}
                >
                  <option value="">—</option>
                  {clients.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label={t("common.date")}>
                <Input
                  mono
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                />
              </Field>
              <Field label={t("accounting.currency")}>
                <Select
                  value={currencyCode}
                  onChange={(e) => {
                    setCurrencyCode(e.target.value);
                    setAllocations({});
                  }}
                  disabled={editing || currencyCatalog.length === 0}
                >
                  {currencyCatalog.map((c) => (
                    <option key={c.code} value={c.code}>
                      {c.code}
                    </option>
                  ))}
                </Select>
              </Field>
              {currency ? (
                <Field label={t("payments.amount")}>
                  <MoneyInput
                    valueMinor={amountCents}
                    currency={currency}
                    onChangeMinor={setAmountCents}
                  />
                </Field>
              ) : null}
              <Field label={t("payments.method")}>
                <Select
                  value={methodKind}
                  onChange={(e) => setMethodKind(e.target.value as PaymentMethodKind)}
                >
                  {METHOD_KINDS.map((k) => (
                    <option key={k} value={k}>
                      {t(`payments.method_${k.toLowerCase()}`)}
                    </option>
                  ))}
                </Select>
              </Field>
              {methodKind === "Other" ? (
                <Field label={t("payments.method_detail")}>
                  <Input
                    value={methodDetail}
                    onChange={(e) => setMethodDetail(e.target.value)}
                  />
                </Field>
              ) : null}
              <Field label={t("payments.reference")}>
                <Input
                  mono
                  value={reference}
                  onChange={(e) => setReference(e.target.value)}
                />
              </Field>
              <Field label={t("common.notes")} className="sm:col-span-3">
                <Textarea
                  rows={2}
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                />
              </Field>
            </div>
          </CardBody>

          <CardHead title={t("payments.allocations")} className="border-t border-line" />
          <CardBody>
            {!clientId ? (
              <p className="text-[12px] text-ink-4">
                {t("payments.select_client_first")}
              </p>
            ) : clientOutstanding.length === 0 ? (
              <p className="text-[12px] text-ink-4">{t("payments.no_outstanding")}</p>
            ) : (
              <>
                {clientOutstandingInCurrency.length === 0 ? (
                  <p className="text-[12px] text-ink-4">
                    {t("payments.no_outstanding_in_currency", {
                      currency: currencyCode,
                    })}
                  </p>
                ) : (
                  <div className="border border-line rounded-card overflow-hidden">
                    <div
                      className="grid items-center px-3 py-2 text-[12px] font-medium text-ink-3 bg-paper-2 border-b border-line"
                      style={{ gridTemplateColumns: "24px 60px 1fr 100px 140px" }}
                    >
                      <span />
                      <span>N°</span>
                      <span>Date / échéance</span>
                      <span className="text-right">{t("accounting.amount_due")}</span>
                      <span className="text-right">{t("payments.allocate_amount")}</span>
                    </div>
                    {clientOutstandingInCurrency.map((row) => {
                      const checked = row.invoice_id in allocations;
                      const cents = allocations[row.invoice_id] ?? 0;
                      return (
                        <div
                          key={row.invoice_id}
                          className="grid items-center px-3 py-2.5 border-b border-line-soft last:border-b-0 text-[13px]"
                          style={{ gridTemplateColumns: "24px 60px 1fr 100px 140px" }}
                        >
                          <Checkbox checked={checked} onChange={() => toggleAllocation(row)} />
                          <span className="font-mono tabular">#{row.number ?? "—"}</span>
                          <span className="text-ink-3 text-[12px]">
                            {row.due_date ?? "—"}
                          </span>
                          <span className="text-right font-mono tabular">
                            {formatMinor(row.amount_due.amount, row.amount_due.currency.code)}
                          </span>
                          <span className="text-right">
                            {checked && currency ? (
                              <MoneyInput
                                valueMinor={cents}
                                currency={currency}
                                onChangeMinor={(c) => updateAllocation(row.invoice_id, c)}
                              />
                            ) : null}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                )}
                {otherCurrencyCount > 0 ? (
                  <p className="mt-2 text-[12px] text-ink-3 italic">
                    {t("payments.invoices_in_other_currencies", {
                      count: otherCurrencyCount,
                      currency: currencyCode,
                    })}
                  </p>
                ) : null}
              </>
            )}
            <div className="mt-3.5 flex justify-between text-[13px]">
              <span className="text-ink-3">
                {t("payments.allocated")}:{" "}
                <span className="font-mono tabular text-ink">
                  {formatMinor(allocatedTotal, currencyCode)}
                </span>
              </span>
              <span
                className={
                  unallocated < 0
                    ? "text-danger font-medium"
                    : unallocated === 0
                      ? "text-ok-ink"
                      : "text-ink-3"
                }
              >
                {t("payments.unallocated")}:{" "}
                <span className="font-mono tabular">
                  {formatMinor(unallocated, currencyCode)}
                </span>
              </span>
            </div>
            {err ? <p className="mt-3 text-[13px] text-danger">{err}</p> : null}
          </CardBody>
        </Card>
        <div className="mt-4 flex justify-end gap-2">
          <Button type="button" onClick={() => navigate("/payments")}>
            {t("common.cancel")}
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={submitting || unallocated < 0}
          >
            {t("common.save")}
          </Button>
        </div>
      </form>
    </Page>
  );
}
