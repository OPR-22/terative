import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { MoneyInput } from "../components/common/MoneyInput";
import { StatusBadge } from "../components/invoice/StatusBadge";
import { MarkPaidModal } from "../components/invoice/MarkPaidModal";
import { useMoneyFormat } from "../lib/money";
import { useInvoiceStore } from "../stores/invoiceStore";
import { useClientStore } from "../stores/clientStore";
import { useCatalogStore } from "../stores/catalogStore";
import { useTaxStore } from "../stores/taxStore";
import { useTemplateStore } from "../stores/templateStore";
import { useSettingsStore } from "../stores/settingsStore";
import type {
  InvoiceDto,
  NewInvoiceDto,
  NewLineItemDto,
  UpdateDraftInvoiceDto,
} from "../ipc";

interface Props {
  invoice: InvoiceDto | null; // null = create mode
  onClose: () => void;
}

interface LineRow {
  description: string;
  quantity: string;
  unit_price_cents: number;
}

interface FormState {
  client_id: string;
  template_id: string | null;
  date: string;
  due_date: string;
  notes: string;
  lines: LineRow[];
  tax_ids: string[];
}

const today = () => new Date().toISOString().slice(0, 10);

function initialForm(invoice: InvoiceDto | null): FormState {
  if (!invoice) {
    return {
      client_id: "",
      template_id: null,
      date: today(),
      due_date: "",
      notes: "",
      lines: [{ description: "", quantity: "1", unit_price_cents: 0 }],
      tax_ids: [],
    };
  }
  return {
    client_id: invoice.client_id,
    template_id: invoice.template_id,
    date: invoice.date,
    due_date: invoice.due_date ?? "",
    notes: invoice.notes ?? "",
    lines: invoice.line_items.map((li) => ({
      description: li.description,
      quantity: li.quantity,
      unit_price_cents: li.unit_price.amount_minor,
    })),
    tax_ids: invoice.taxes_applied
      .map((t) => t.tax_definition_id)
      .filter((id): id is string => id != null),
  };
}

export function InvoiceEditor({ invoice, onClose }: Props) {
  const { t } = useTranslation();
  const { createDraft, updateDraft, finalize, cancel, send } = useInvoiceStore();
  const { clients, refresh: refreshClients } = useClientStore();
  const { items: catalogItems, refresh: refreshCatalog } = useCatalogStore();
  const { taxes, refresh: refreshTaxes } = useTaxStore();
  const { templates, refresh: refreshTemplates } = useTemplateStore();
  const { snapshot, load: loadSettings } = useSettingsStore();

  const [form, setForm] = useState<FormState>(() => initialForm(invoice));
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [markingPaid, setMarkingPaid] = useState(false);

  useEffect(() => {
    if (clients.length === 0) void refreshClients();
    if (catalogItems.length === 0) void refreshCatalog();
    if (taxes.length === 0) void refreshTaxes();
    if (templates.length === 0) void refreshTemplates();
    if (!snapshot) void loadSettings();
  }, [
    clients.length,
    catalogItems.length,
    taxes.length,
    templates.length,
    snapshot,
    refreshClients,
    refreshCatalog,
    refreshTaxes,
    refreshTemplates,
    loadSettings,
  ]);

  const currencyCode = snapshot?.currency.code ?? "EUR";
  const appCurrency = snapshot?.currency;
  const { formatMinor } = useMoneyFormat();
  const readOnly = invoice !== null && invoice.status !== "Draft";

  const subtotalCents = useMemo(
    () =>
      form.lines.reduce((sum, li) => {
        const q = parseFloat(li.quantity);
        if (Number.isNaN(q)) return sum;
        return sum + Math.round(q * li.unit_price_cents);
      }, 0),
    [form.lines],
  );

  const taxBreakdown = useMemo(() => {
    return form.tax_ids
      .map((id) => taxes.find((t) => t.id === id))
      .filter((t): t is NonNullable<typeof t> => t != null)
      .map((t) => {
        const pct = parseFloat(t.percentage);
        const amount = Number.isNaN(pct)
          ? 0
          : Math.round((subtotalCents * pct) / 100);
        return { id: t.id, name: t.name, percentage: t.percentage, amount };
      });
  }, [form.tax_ids, taxes, subtotalCents]);

  const taxTotalCents = taxBreakdown.reduce((sum, t) => sum + t.amount, 0);
  const totalCents = subtotalCents + taxTotalCents;

  const updateLine = (idx: number, patch: Partial<LineRow>) =>
    setForm((f) => ({
      ...f,
      lines: f.lines.map((li, i) => (i === idx ? { ...li, ...patch } : li)),
    }));

  const addLine = () =>
    setForm((f) => ({
      ...f,
      lines: [
        ...f.lines,
        { description: "", quantity: "1", unit_price_cents: 0 },
      ],
    }));

  const removeLine = (idx: number) =>
    setForm((f) => ({
      ...f,
      lines: f.lines.length > 1 ? f.lines.filter((_, i) => i !== idx) : f.lines,
    }));

  const toggleTax = (id: string) =>
    setForm((f) => ({
      ...f,
      tax_ids: f.tax_ids.includes(id)
        ? f.tax_ids.filter((x) => x !== id)
        : [...f.tax_ids, id],
    }));

  const buildLineItems = (): NewLineItemDto[] =>
    form.lines
      .filter((li) => li.description.trim() !== "")
      .map((li) => ({
        description: li.description,
        quantity: li.quantity || "1",
        unit_price: {
          amount_minor: li.unit_price_cents,
          currency: currencyCode,
        },
      }));

  const submitDraft = async () => {
    setError(null);
    setSubmitting(true);
    try {
      if (!form.client_id) {
        throw new Error(t("invoices.err_no_client"));
      }
      if (invoice && invoice.status === "Draft") {
        const payload: UpdateDraftInvoiceDto = {
          id: invoice.id,
          template_id: form.template_id,
          date: form.date,
          due_date: form.due_date || null,
          line_items: buildLineItems(),
          tax_ids: form.tax_ids,
          notes: form.notes || null,
        };
        await updateDraft(payload);
      } else {
        const payload: NewInvoiceDto = {
          client_id: form.client_id,
          template_id: form.template_id,
          date: form.date,
          due_date: form.due_date || null,
          line_items: buildLineItems(),
          tax_ids: form.tax_ids,
          notes: form.notes || null,
          currency: currencyCode,
        };
        await createDraft(payload);
      }
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const finalizeNow = async () => {
    if (!invoice) return;
    setError(null);
    setSubmitting(true);
    try {
      await finalize(invoice.id);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const cancelInvoice = async () => {
    if (!invoice) return;
    if (!confirm(t("invoices.confirm_cancel"))) return;
    setSubmitting(true);
    try {
      await cancel(invoice.id);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const sendInvoice = async () => {
    if (!invoice) return;
    setError(null);
    setSubmitting(true);
    try {
      await send(invoice.id);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold text-fg">
            {invoice
              ? invoice.number != null
                ? `${t("invoices.title")} #${invoice.number}`
                : t("invoices.edit")
              : t("invoices.new")}
          </h1>
          {invoice ? <StatusBadge status={invoice.status} /> : null}
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose}>
            {t("common.back")}
          </Button>
          {!readOnly ? (
            <Button onClick={submitDraft} disabled={submitting}>
              {t("invoices.save_draft")}
            </Button>
          ) : null}
          {invoice && invoice.status === "Draft" ? (
            <Button onClick={finalizeNow} disabled={submitting}>
              {t("invoices.finalize")}
            </Button>
          ) : null}
          {invoice && invoice.status === "Finalized" ? (
            <Button onClick={sendInvoice} disabled={submitting}>
              {t("invoices.send")}
            </Button>
          ) : null}
          {invoice &&
          (invoice.status === "Finalized" || invoice.status === "Sent") &&
          invoice.payment_status !== "Paid" ? (
            <Button
              variant="secondary"
              onClick={() => setMarkingPaid(true)}
              disabled={submitting}
            >
              {t("invoices.mark_paid")}
            </Button>
          ) : null}
          {invoice &&
          (invoice.status === "Finalized" || invoice.status === "Sent") ? (
            <Button variant="danger" onClick={cancelInvoice} disabled={submitting}>
              {t("invoices.cancel")}
            </Button>
          ) : null}
        </div>
      </div>

      {error ? <p className="mb-3 text-sm text-danger">{error}</p> : null}

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="rounded-card border border-border bg-surface p-4 shadow-card">
          <h2 className="mb-3 text-sm font-semibold text-fg-muted">
            {t("invoices.section_header")}
          </h2>
          <div className="flex flex-col gap-3">
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("invoices.client")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm disabled:opacity-60"
                value={form.client_id}
                disabled={readOnly}
                onChange={(e) =>
                  setForm({ ...form, client_id: e.target.value })
                }
              >
                <option value="">—</option>
                {clients.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("invoices.template")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm disabled:opacity-60"
                value={form.template_id ?? ""}
                disabled={readOnly}
                onChange={(e) =>
                  setForm({
                    ...form,
                    template_id: e.target.value || null,
                  })
                }
              >
                <option value="">{t("invoices.template_default")}</option>
                {templates.map((tpl) => (
                  <option key={tpl.id} value={tpl.id}>
                    {tpl.name}
                    {tpl.is_default ? ` · ${t("templates.default")}` : ""}
                  </option>
                ))}
              </select>
            </label>
            <div className="grid grid-cols-2 gap-3">
              <Input
                type="date"
                label={t("common.date") ?? ""}
                value={form.date}
                disabled={readOnly}
                onChange={(e) => setForm({ ...form, date: e.target.value })}
              />
              <Input
                type="date"
                label={t("invoices.due_date") ?? ""}
                value={form.due_date}
                disabled={readOnly}
                onChange={(e) =>
                  setForm({ ...form, due_date: e.target.value })
                }
              />
            </div>
            <Input
              label={t("common.notes") ?? ""}
              value={form.notes}
              disabled={readOnly}
              onChange={(e) => setForm({ ...form, notes: e.target.value })}
            />
          </div>
        </section>

        <section className="rounded-card border border-border bg-surface p-4 shadow-card">
          <h2 className="mb-3 text-sm font-semibold text-fg-muted">
            {t("invoices.section_taxes")}
          </h2>
          {taxes.length === 0 ? (
            <p className="text-xs text-fg-subtle">{t("invoices.no_taxes")}</p>
          ) : (
            <div className="flex flex-col gap-2">
              {taxes.map((tax) => (
                <label
                  key={tax.id}
                  className="flex items-center justify-between gap-2 text-sm text-fg-muted"
                >
                  <span>
                    <input
                      type="checkbox"
                      className="mr-2"
                      checked={form.tax_ids.includes(tax.id)}
                      disabled={readOnly}
                      onChange={() => toggleTax(tax.id)}
                    />
                    {tax.name}
                  </span>
                  <span className="text-fg-subtle">{tax.percentage}%</span>
                </label>
              ))}
            </div>
          )}
          <div className="mt-4 space-y-1 border-t border-border pt-3 text-sm">
            <div className="flex justify-between text-fg-muted">
              <span>{t("invoices.subtotal")}</span>
              <span>{formatMinor(subtotalCents, currencyCode)}</span>
            </div>
            {taxBreakdown.map((t) => (
              <div key={t.id} className="flex justify-between text-fg-muted">
                <span>
                  {t.name} ({t.percentage}%)
                </span>
                <span>{formatMinor(t.amount, currencyCode)}</span>
              </div>
            ))}
            <div className="flex justify-between pt-1 text-base font-semibold text-fg">
              <span>{t("invoices.total")}</span>
              <span>{formatMinor(totalCents, currencyCode)}</span>
            </div>
          </div>
        </section>

        <section className="rounded-card border border-border bg-surface p-4 shadow-card lg:col-span-2">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-fg-muted">
              {t("invoices.section_lines")}
            </h2>
            {!readOnly ? (
              <Button variant="secondary" onClick={addLine}>
                {t("invoices.add_line")}
              </Button>
            ) : null}
          </div>
          <div className="flex flex-col gap-2">
            {form.lines.map((line, idx) => {
              const q = parseFloat(line.quantity);
              const lineTotal =
                Number.isNaN(q) ? 0 : Math.round(q * line.unit_price_cents);
              return (
                <div
                  key={idx}
                  className="grid grid-cols-12 items-end gap-2 rounded-field border border-border p-2"
                >
                  <div className="col-span-12 md:col-span-6 flex flex-col gap-1">
                    {!readOnly && catalogItems.length > 0 ? (
                      <select
                        className="block w-full rounded-field border border-border bg-surface px-2 py-1 text-xs text-fg-muted shadow-sm"
                        value=""
                        onChange={(e) => {
                          const item = catalogItems.find(
                            (c) => c.id === e.target.value,
                          );
                          if (!item) return;
                          updateLine(idx, {
                            description: item.name,
                            unit_price_cents: item.default_price.amount_minor,
                          });
                        }}
                      >
                        <option value="">
                          {t("invoices.pick_catalog_item")}
                        </option>
                        {(["Service", "Product"] as const).map((kind) => {
                          const group = catalogItems.filter(
                            (c) => c.kind === kind,
                          );
                          if (group.length === 0) return null;
                          return (
                            <optgroup
                              key={kind}
                              label={t(`catalog.kind_${kind.toLowerCase()}_plural`)}
                            >
                              {group.map((c) => (
                                <option key={c.id} value={c.id}>
                                  {c.reference ? `[${c.reference}] ` : ""}
                                  {c.name} · {formatMinor(
                                    c.default_price.amount_minor,
                                    c.default_price.currency,
                                  )}
                                  {c.unit ? ` / ${c.unit}` : ""}
                                </option>
                              ))}
                            </optgroup>
                          );
                        })}
                      </select>
                    ) : null}
                    <Input
                      label={t("invoices.line_description") ?? ""}
                      value={line.description}
                      disabled={readOnly}
                      onChange={(e) =>
                        updateLine(idx, { description: e.target.value })
                      }
                    />
                  </div>
                  <div className="col-span-3 md:col-span-2">
                    <Input
                      label={t("invoices.line_qty") ?? ""}
                      value={line.quantity}
                      type="number"
                      step="0.01"
                      min="0"
                      disabled={readOnly}
                      onChange={(e) =>
                        updateLine(idx, { quantity: e.target.value })
                      }
                    />
                  </div>
                  <div className="col-span-6 md:col-span-3">
                    {appCurrency ? (
                      <MoneyInput
                        label={t("invoices.line_unit_price") ?? ""}
                        valueMinor={line.unit_price_cents}
                        currency={appCurrency}
                        disabled={readOnly}
                        onChangeMinor={(minor) =>
                          updateLine(idx, { unit_price_cents: minor })
                        }
                      />
                    ) : null}
                  </div>
                  <div className="col-span-2 md:col-span-1 flex justify-end text-right text-sm text-fg">
                    {formatMinor(lineTotal, currencyCode)}
                  </div>
                  {!readOnly ? (
                    <div className="col-span-1 flex justify-end">
                      <button
                        type="button"
                        onClick={() => removeLine(idx)}
                        className="text-fg-subtle hover:text-danger"
                        aria-label={t("common.delete") ?? ""}
                      >
                        ×
                      </button>
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </section>
      </div>

      {markingPaid && invoice ? (
        <MarkPaidModal
          invoice={invoice}
          onClose={() => setMarkingPaid(false)}
          onPaid={onClose}
        />
      ) : null}
    </div>
  );
}
