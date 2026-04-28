import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";
import {
  ArrowLeft,
  Check,
  GripVertical,
  Plus,
  Send,
  X,
} from "lucide-react";

import { Page } from "../components/layout/Page";
import { useWorkspaceName } from "../hooks/useWorkspaceName";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { Field, Input, Select, Textarea } from "../components/ui/Input";
import { StatusDot } from "../components/ui/StatusDot";
import { Tabs, type TabOption } from "../components/ui/Tabs";
import { EmptyState } from "../components/ui/EmptyState";
import { MoneyInput } from "../components/common/MoneyInput";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { ExternalLink, Folder, Printer } from "lucide-react";

import { ConfirmModal } from "../components/ui/ConfirmModal";
import { MarkPaidModal } from "../components/invoice/MarkPaidModal";
import { PdfPreview } from "../components/template/PdfPreview";
import { useMoneyFormat } from "../lib/money";
import { useInvoiceStore } from "../stores/invoiceStore";
import { useClientStore } from "../stores/clientStore";
import { useCatalogStore } from "../stores/catalogStore";
import { useTaxStore } from "../stores/taxStore";
import { useTemplateStore } from "../stores/templateStore";
import { useSettingsStore } from "../stores/settingsStore";
import {
  ipc,
  type InvoiceDto,
  type NewInvoiceDto,
  type NewLineItemDto,
  type PaymentDto,
  type PaymentMethodDto,
  type UpdateDraftInvoiceDto,
} from "../ipc";

interface LineRow {
  description: string;
  quantity: string;
  unit_price_cents: number;
}

interface FormState {
  client_id: string;
  template_id: string | null;
  date: string;
  due_days: string;
  notes: string;
  lines: LineRow[];
  tax_ids: string[];
}

const today = () => new Date().toISOString().slice(0, 10);

function paymentMethodLabel(
  method: PaymentMethodDto,
  t: (k: string) => string,
): string {
  switch (method.kind) {
    case "BankTransfer":
      return t("payments.method_banktransfer");
    case "Cash":
      return t("payments.method_cash");
    case "Check":
      return t("payments.method_check");
    case "Card":
      return t("payments.method_card");
    case "Other":
      return method.detail || t("payments.method_other");
  }
}

function computeDueDate(issueDate: string, days: string): string | null {
  if (days === "") return null;
  const n = parseInt(days, 10);
  if (Number.isNaN(n)) return null;
  const d = new Date(issueDate);
  if (Number.isNaN(d.getTime())) return null;
  d.setDate(d.getDate() + n);
  return d.toISOString().slice(0, 10);
}

function daysBetween(issueDate: string, dueDate: string): string {
  const a = new Date(issueDate);
  const b = new Date(dueDate);
  if (Number.isNaN(a.getTime()) || Number.isNaN(b.getTime())) return "";
  const diff = Math.round((b.getTime() - a.getTime()) / 86_400_000);
  return String(diff);
}

function initialForm(invoice: InvoiceDto | null): FormState {
  if (!invoice) {
    return {
      client_id: "",
      template_id: null,
      date: today(),
      due_days: "",
      notes: "",
      lines: [{ description: "", quantity: "1", unit_price_cents: 0 }],
      tax_ids: [],
    };
  }
  return {
    client_id: invoice.client_id,
    template_id: invoice.template_id,
    date: invoice.date,
    due_days: invoice.due_date ? daysBetween(invoice.date, invoice.due_date) : "",
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

export function InvoiceEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const workspaceName = useWorkspaceName();

  const {
    get: getInvoice,
    createDraft,
    updateDraft,
    finalize,
    cancel,
    send,
  } = useInvoiceStore();
  const { clients, refresh: refreshClients } = useClientStore();
  const { items: catalogItems, refresh: refreshCatalog } = useCatalogStore();
  const { taxes, refresh: refreshTaxes } = useTaxStore();
  const { templates, refresh: refreshTemplates } = useTemplateStore();
  const { snapshot, load: loadSettings } = useSettingsStore();

  const [invoice, setInvoice] = useState<InvoiceDto | null>(null);
  const [form, setForm] = useState<FormState>(() => initialForm(null));
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [markingPaid, setMarkingPaid] = useState(false);
  const [confirmingCancel, setConfirmingCancel] = useState(false);
  const [pdfBytes, setPdfBytes] = useState<Uint8Array | null>(null);
  const [pdfLoading, setPdfLoading] = useState(false);
  const [pdfError, setPdfError] = useState<string | null>(null);
  const [invoicePayments, setInvoicePayments] = useState<PaymentDto[]>([]);
  type ViewerTab = "summary" | "preview" | "payments" | "email";
  const [tab, setTab] = useState<ViewerTab>("summary");

  // Load the invoice when an id is present.
  useEffect(() => {
    if (!id) {
      setInvoice(null);
      setForm(initialForm(null));
      return;
    }
    let cancelled = false;
    getInvoice(id)
      .then((inv) => {
        if (cancelled) return;
        setInvoice(inv);
        setForm(initialForm(inv));
      })
      .catch((e) => {
        if (!cancelled) toast.error(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id, getInvoice]);

  // Pull the list of payments allocated to this invoice once the
  // invoice loads. Drafts can't have payments, so skip the round-trip.
  // Refetch when the invoice id changes (navigating between invoices).
  useEffect(() => {
    if (!invoice || invoice.status === "Draft") {
      setInvoicePayments([]);
      return;
    }
    let cancelled = false;
    ipc
      .paymentList({ invoice_id: invoice.id })
      .then((rows) => {
        if (!cancelled) setInvoicePayments(rows);
      })
      .catch(() => {
        // Non-critical — the section just shows empty if the fetch fails.
        if (!cancelled) setInvoicePayments([]);
      });
    return () => {
      cancelled = true;
    };
  }, [invoice?.id, invoice?.status]);

  // Load the rendered PDF whenever the invoice gains a `pdf_path`. This
  // covers both opening an already-finalized invoice and a fresh finalize
  // that mutates the invoice in place (we re-fetch the bytes when the
  // path changes). Drafts have no PDF — skip the call.
  useEffect(() => {
    if (!invoice || !invoice.pdf_path) {
      setPdfBytes(null);
      setPdfError(null);
      return;
    }
    let cancelled = false;
    setPdfLoading(true);
    setPdfError(null);
    ipc
      .invoicePdfBytes(invoice.id)
      .then((bytes) => {
        if (cancelled) return;
        setPdfBytes(new Uint8Array(bytes));
      })
      .catch((e) => {
        if (!cancelled) toast.error(String(e));
      })
      .finally(() => {
        if (!cancelled) setPdfLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [invoice?.id, invoice?.pdf_path]);

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

  const seededTaxesRef = useRef(false);
  useEffect(() => {
    if (invoice !== null) return;
    if (seededTaxesRef.current) return;
    if (taxes.length === 0) return;
    seededTaxesRef.current = true;
    setForm((f) => ({ ...f, tax_ids: taxes.map((t) => t.id) }));
  }, [invoice, taxes]);

  const seededDueDaysRef = useRef(false);
  useEffect(() => {
    if (invoice !== null) return;
    if (seededDueDaysRef.current) return;
    if (!snapshot) return;
    seededDueDaysRef.current = true;
    const days = snapshot.preferences.default_invoice_due_days;
    if (days <= 0) return;
    setForm((f) => (f.due_days ? f : { ...f, due_days: String(days) }));
  }, [invoice, snapshot]);

  const currencyCode = snapshot?.currency.code ?? "EUR";
  const appCurrency = snapshot?.currency;
  const { formatMinor } = useMoneyFormat();
  const readOnly = invoice !== null && invoice.status !== "Draft";
  const selectedClientName =
    invoice?.client_name ?? clients.find((c) => c.id === form.client_id)?.name ?? null;

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

  const toggleTax = (tid: string) =>
    setForm((f) => ({
      ...f,
      tax_ids: f.tax_ids.includes(tid)
        ? f.tax_ids.filter((x) => x !== tid)
        : [...f.tax_ids, tid],
    }));

  const buildLineItems = (): NewLineItemDto[] =>
    form.lines
      .filter((li) => li.description.trim() !== "")
      .map((li) => ({
        description: li.description,
        quantity: li.quantity || "1",
        unit_price: { amount_minor: li.unit_price_cents, currency: currencyCode },
      }));

  const persistDraft = async (): Promise<string> => {
    if (!form.client_id) {
      throw new Error(t("invoices.err_no_client"));
    }
    const dueDate = computeDueDate(form.date, form.due_days);
    if (invoice && invoice.status === "Draft") {
      const payload: UpdateDraftInvoiceDto = {
        id: invoice.id,
        template_id: form.template_id,
        date: form.date,
        due_date: dueDate,
        line_items: buildLineItems(),
        tax_ids: form.tax_ids,
        notes: form.notes || null,
      };
      await updateDraft(payload);
      return invoice.id;
    }
    const payload: NewInvoiceDto = {
      client_id: form.client_id,
      template_id: form.template_id,
      date: form.date,
      due_date: dueDate,
      line_items: buildLineItems(),
      tax_ids: form.tax_ids,
      notes: form.notes || null,
      currency: currencyCode,
    };
    const created = await createDraft(payload);
    return created.id;
  };

  const goBack = () => navigate("/invoices");

  const submitDraft = async () => {
    setError(null);
    setSubmitting(true);
    try {
      await persistDraft();
      goBack();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const finalizeNow = async () => {
    setError(null);
    setSubmitting(true);
    try {
      const newId = await persistDraft();
      await finalize(newId);
      goBack();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const cancelInvoice = async () => {
    if (!invoice) return;
    setSubmitting(true);
    try {
      await cancel(invoice.id);
      goBack();
    } catch (e) {
      toast.error(String(e));
      throw e;
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
      goBack();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const titleNode = invoice ? (
    <>
      Facture{" "}
      <span className="text-ink-3 font-mono text-[20px]">
        #{invoice.number ?? "—"}
      </span>
    </>
  ) : (
    t("invoices.new")
  );

  const subtitleNode = invoice ? (
    <span className="inline-flex items-center gap-2">
      {t("invoices.issued_on", { date: invoice.date })}
      {invoice.due_date
        ? ` · ${t("invoices.due_on", { date: invoice.due_date })}`
        : null}
      <Badge dot kind={invoice.status === "Draft" ? "draft" : invoice.status === "Cancelled" ? "cancel" : invoice.status === "Sent" ? "sent" : "final"}>
        {t(`invoices.status_${invoice.status.toLowerCase()}`)}
      </Badge>
    </span>
  ) : (
    t("invoices.draft_subtitle")
  );

  return (
    <Page
      crumbs={[
        workspaceName,
        t("invoices.title"),
        invoice
          ? `#${invoice.number ?? "—"}${selectedClientName ? ` — ${selectedClientName}` : ""}`
          : t("invoices.new"),
      ]}
      title={titleNode}
      subtitle={subtitleNode}
      actions={
        <>
          <Button
            leadingIcon={<ArrowLeft size={13} strokeWidth={1.5} />}
            onClick={goBack}
          >
            {t("common.back")}
          </Button>
          {!readOnly ? (
            <>
              <Button onClick={submitDraft} disabled={submitting}>
                {t("invoices.save_draft")}
              </Button>
              <Button variant="primary" onClick={finalizeNow} disabled={submitting}>
                {t("invoices.finalize")}
              </Button>
            </>
          ) : null}
          {invoice && invoice.status === "Finalized" ? (
            <Button
              variant="primary"
              leadingIcon={<Send size={13} strokeWidth={1.5} />}
              onClick={sendInvoice}
              disabled={submitting}
            >
              {t("invoices.send")}
            </Button>
          ) : null}
          {invoice &&
          (invoice.status === "Finalized" || invoice.status === "Sent") &&
          invoice.payment_status !== "Paid" ? (
            <Button
              leadingIcon={<Check size={13} strokeWidth={1.5} />}
              onClick={() => setMarkingPaid(true)}
              disabled={submitting}
            >
              {t("invoices.mark_paid")}
            </Button>
          ) : null}
          {invoice &&
          (invoice.status === "Finalized" || invoice.status === "Sent") ? (
            <Button
              variant="danger"
              onClick={() => setConfirmingCancel(true)}
              disabled={submitting}
            >
              {t("invoices.cancel")}
            </Button>
          ) : null}
        </>
      }
    >
      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Tabs<ViewerTab>
        value={tab}
        onChange={setTab}
        className="mb-5"
        options={
          [
            { id: "summary", label: t("invoices.tab_summary") },
            { id: "preview", label: t("invoices.tab_preview") },
            {
              id: "payments",
              label: t("invoices.tab_payments"),
              count: invoice && invoice.status !== "Draft"
                ? invoicePayments.length
                : null,
            },
            {
              id: "email",
              label: t("invoices.tab_email"),
              count: invoice ? invoice.email_sends.length : null,
            },
          ] as TabOption<ViewerTab>[]
        }
      />

      {tab === "summary" ? (
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1fr_380px]">
        <Card>
          <CardHead
            title={t("invoices.section_header")}
            actions={
              invoice ? (
                <span className="inline-flex items-center gap-1.5 text-[12px] text-ink-3">
                  <StatusDot status="ok" />
                  {t("invoices.saved")}
                </span>
              ) : null
            }
          />
          <CardBody>
            <div className="grid grid-cols-2 gap-3.5">
              <Field label={t("invoices.client")}>
                <Select
                  value={form.client_id}
                  disabled={readOnly}
                  onChange={(e) => setForm({ ...form, client_id: e.target.value })}
                >
                  <option value="">—</option>
                  {clients.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label={t("invoices.template")}>
                <Select
                  value={form.template_id ?? ""}
                  disabled={readOnly}
                  onChange={(e) =>
                    setForm({ ...form, template_id: e.target.value || null })
                  }
                >
                  <option value="">{t("invoices.template_default")}</option>
                  {templates.map((tpl) => (
                    <option key={tpl.id} value={tpl.id}>
                      {tpl.name}
                      {tpl.is_default ? ` · ${t("templates.default")}` : ""}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label={t("common.date")}>
                <Input
                  type="date"
                  mono
                  value={form.date}
                  disabled={readOnly}
                  onChange={(e) => setForm({ ...form, date: e.target.value })}
                />
              </Field>
              <Field
                label={t("invoices.due_in_days")}
                help={
                  form.due_days === ""
                    ? t("invoices.due_no_date") ?? undefined
                    : (() => {
                        const d = computeDueDate(form.date, form.due_days);
                        return d ? `→ ${d}` : "—";
                      })()
                }
              >
                <Input
                  mono
                  type="number"
                  min="0"
                  value={form.due_days}
                  disabled={readOnly}
                  onChange={(e) => setForm({ ...form, due_days: e.target.value })}
                  placeholder="30"
                />
              </Field>
            </div>
            <div className="mt-3.5">
              <Field
                label={t("invoices.public_notes")}
                help={t("invoices.public_notes_hint") ?? undefined}
              >
                <Textarea
                  rows={3}
                  value={form.notes}
                  disabled={readOnly}
                  onChange={(e) => setForm({ ...form, notes: e.target.value })}
                />
              </Field>
            </div>
          </CardBody>

          <CardHead
            title={t("invoices.section_lines")}
            className="border-t border-line"
            actions={
              !readOnly ? (
                <Button
                  size="sm"
                  leadingIcon={<Plus size={11} strokeWidth={1.5} />}
                  onClick={addLine}
                >
                  {t("invoices.add_line")}
                </Button>
              ) : null
            }
          />
          <div>
            {form.lines.map((line, idx) => {
              const q = parseFloat(line.quantity);
              const lineTotal = Number.isNaN(q) ? 0 : Math.round(q * line.unit_price_cents);
              return (
                <div
                  key={idx}
                  className="grid items-center gap-2 px-5 py-3 border-b border-line-soft last:border-b-0"
                  style={{ gridTemplateColumns: "24px 1fr 80px 140px 120px 24px" }}
                >
                  <span className="text-ink-4 cursor-grab">
                    <GripVertical size={14} strokeWidth={1.5} />
                  </span>
                  <div className="min-w-0">
                    <Input
                      value={line.description}
                      disabled={readOnly}
                      onChange={(e) => updateLine(idx, { description: e.target.value })}
                      placeholder={t("invoices.line_description") ?? ""}
                    />
                    {!readOnly && catalogItems.length > 0 ? (
                      <Select
                        className="mt-1.5 text-[11px] py-1"
                        value=""
                        onChange={(e) => {
                          const item = catalogItems.find((c) => c.id === e.target.value);
                          if (!item) return;
                          updateLine(idx, {
                            description: item.name,
                            unit_price_cents: item.default_price.amount_minor,
                          });
                        }}
                      >
                        <option value="">{t("invoices.pick_catalog_item")}</option>
                        {(["Service", "Product"] as const).map((kind) => {
                          const group = catalogItems.filter((c) => c.kind === kind);
                          if (group.length === 0) return null;
                          return (
                            <optgroup
                              key={kind}
                              label={t(`catalog.kind_${kind.toLowerCase()}_plural`)}
                            >
                              {group.map((c) => (
                                <option key={c.id} value={c.id}>
                                  {c.reference ? `[${c.reference}] ` : ""}
                                  {c.name} · {formatMinor(c.default_price.amount_minor, c.default_price.currency)}
                                  {c.unit ? ` / ${c.unit}` : ""}
                                </option>
                              ))}
                            </optgroup>
                          );
                        })}
                      </Select>
                    ) : null}
                  </div>
                  <Input
                    mono
                    type="number"
                    step="0.01"
                    min="0"
                    value={line.quantity}
                    disabled={readOnly}
                    onChange={(e) => updateLine(idx, { quantity: e.target.value })}
                    className="text-right"
                  />
                  <div>
                    {appCurrency ? (
                      <MoneyInput
                        valueMinor={line.unit_price_cents}
                        currency={appCurrency}
                        disabled={readOnly}
                        onChangeMinor={(minor) =>
                          updateLine(idx, { unit_price_cents: minor })
                        }
                      />
                    ) : null}
                  </div>
                  <span className="text-right tabular font-mono text-[13px]">
                    {formatMinor(lineTotal, currencyCode)}
                  </span>
                  {!readOnly ? (
                    <button
                      type="button"
                      onClick={() => removeLine(idx)}
                      className="text-ink-3 hover:text-danger"
                      aria-label={t("common.delete") ?? ""}
                    >
                      <X size={13} strokeWidth={1.5} />
                    </button>
                  ) : (
                    <span />
                  )}
                </div>
              );
            })}
          </div>
        </Card>

        <div className="flex flex-col gap-3.5">
          <Card>
            <CardHead title={t("invoices.section_taxes")} />
            <CardBody>
              {taxes.length === 0 ? (
                <p className="text-[12px] text-ink-4">{t("invoices.no_taxes")}</p>
              ) : (
                <div className="flex flex-col gap-1.5">
                  {taxes.map((tax) => (
                    <Checkbox
                      key={tax.id}
                      checked={form.tax_ids.includes(tax.id)}
                      disabled={readOnly}
                      onChange={() => toggleTax(tax.id)}
                    >
                      <span className="flex-1">{tax.name}</span>
                      <span className="ml-auto font-mono tabular text-ink-3 text-[12px]">
                        {tax.percentage}&nbsp;%
                      </span>
                    </Checkbox>
                  ))}
                </div>
              )}
            </CardBody>
          </Card>

          <Card>
            <CardHead title={t("invoices.totals")} />
            <CardBody>
              <div className="flex justify-between py-1.5 text-[13px] text-ink-3">
                <span>{t("invoices.subtotal")}</span>
                <span className="font-mono tabular text-ink">
                  {formatMinor(subtotalCents, currencyCode)}
                </span>
              </div>
              {taxBreakdown.map((t) => (
                <div key={t.id} className="flex justify-between py-1.5 text-[13px] text-ink-3">
                  <span>
                    {t.name} ({t.percentage}%)
                  </span>
                  <span className="font-mono tabular text-ink">
                    {formatMinor(t.amount, currencyCode)}
                  </span>
                </div>
              ))}
              <div className="flex items-baseline justify-between border-t border-line mt-2 pt-3">
                <span className="font-medium">{t("invoices.total")}</span>
                <span className="font-mono tabular text-[18px] font-semibold">
                  {formatMinor(totalCents, currencyCode)}
                </span>
              </div>
            </CardBody>
          </Card>

          {invoice ? (
            <div className="text-[11px] text-ink-3">
              {t("invoices.created_at", { date: invoice.created_at })}
            </div>
          ) : null}
        </div>
      </div>
      ) : null}

      {tab === "preview" ? (
        invoice && invoice.pdf_path ? (
          <Card className="overflow-hidden">
            <CardHead
              title={t("invoices.preview_title")}
              subtitle={invoice.pdf_path}
              actions={
                <>
                  <Button
                    size="sm"
                    leadingIcon={<Folder size={11} strokeWidth={1.5} />}
                    onClick={() => {
                      if (!invoice.pdf_path) return;
                      void revealItemInDir(invoice.pdf_path).catch((e) =>
                        toast.error(String(e)),
                      );
                    }}
                  >
                    {t("invoices.preview_open_folder")}
                  </Button>
                  <Button
                    size="sm"
                    leadingIcon={<ExternalLink size={11} strokeWidth={1.5} />}
                    onClick={() => {
                      void ipc
                        .invoiceOpenExternal(invoice.id)
                        .catch((e) => toast.error(String(e)));
                    }}
                  >
                    {t("invoices.preview_open_externally")}
                  </Button>
                  <Button
                    size="sm"
                    variant="primary"
                    leadingIcon={<Printer size={11} strokeWidth={1.5} />}
                    onClick={() => {
                      void ipc
                        .invoicePrint(invoice.id)
                        .catch((e) => toast.error(String(e)));
                    }}
                  >
                    {t("invoices.preview_print")}
                  </Button>
                </>
              }
            />
            <div className="h-[820px] bg-paper-3">
              <PdfPreview
                bytes={pdfBytes}
                loading={pdfLoading}
                error={pdfError}
              />
            </div>
          </Card>
        ) : (
          <Card>
            <EmptyState description={t("invoices.preview_unavailable")} />
          </Card>
        )
      ) : null}

      {tab === "payments" && invoice ? (
        <Card>
          <CardHead
            title={t("invoices.payments_section")}
            actions={
              (() => {
                const dueCents =
                  invoice.total.amount_minor - invoice.amount_paid.amount_minor;
                const fullyPaid = dueCents <= 0;
                return (
                  <span className="inline-flex items-baseline gap-2 text-[12px]">
                    <span className="text-ink-3">
                      {fullyPaid
                        ? t("invoices.fully_paid")
                        : t("invoices.remaining_due")}
                    </span>
                    <span
                      className={[
                        "font-mono tabular text-[14px] font-medium",
                        fullyPaid ? "text-ok-ink" : "text-ink",
                      ].join(" ")}
                    >
                      {formatMinor(
                        Math.max(0, dueCents),
                        invoice.total.currency,
                      )}
                    </span>
                  </span>
                );
              })()
            }
          />
          {invoicePayments.length === 0 ? (
            <EmptyState description={t("invoices.payments_none")} />
          ) : (
            <div className="flex flex-col">
              {invoicePayments.map((p) => {
                const allocation = p.allocations.find(
                  (a) => a.invoice_id === invoice.id,
                );
                const amount = allocation
                  ? formatMinor(
                      allocation.amount.amount_minor,
                      allocation.amount.currency,
                    )
                  : null;
                return (
                  <button
                    type="button"
                    key={p.id}
                    onClick={() => navigate(`/payments/${p.id}/edit`)}
                    className="flex items-start justify-between gap-3 px-5 py-3 border-b border-line-soft last:border-b-0 cursor-pointer hover:bg-paper-2 transition-colors text-left"
                  >
                    <div className="min-w-0">
                      <div className="text-[13px] text-ink">
                        {paymentMethodLabel(p.method, t)}
                        {p.reference ? (
                          <span className="ml-1.5 text-ink-3 font-mono">
                            · {p.reference}
                          </span>
                        ) : null}
                      </div>
                      <div className="text-[11px] text-ink-3 font-mono mt-0.5">
                        {p.date}
                      </div>
                    </div>
                    {amount ? (
                      <span className="font-mono tabular text-[14px] font-medium text-ink shrink-0">
                        {amount}
                      </span>
                    ) : null}
                  </button>
                );
              })}
            </div>
          )}
        </Card>
      ) : null}

      {tab === "email" && invoice ? (
        <Card>
          <CardHead title={t("invoices.send_history")} />
          {invoice.email_sends.length === 0 ? (
            <EmptyState description={t("invoices.email_none")} />
          ) : (
            <div className="flex flex-col">
              {invoice.email_sends.map((s) => (
                <div
                  key={s.id}
                  className="flex items-start gap-3 px-5 py-3 border-b border-line-soft last:border-b-0"
                >
                  <Send size={13} strokeWidth={1.5} className="text-ink-3 mt-0.5 shrink-0" />
                  <div className="flex-1 text-[13px] min-w-0">
                    <div>
                      {s.template_type === "InitialContact"
                        ? t("invoices.email_kind_initial")
                        : t("invoices.email_kind_follow_up")}
                    </div>
                    <div className="text-[11px] text-ink-3 font-mono mt-0.5">
                      {s.sent_at}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      ) : null}

      {markingPaid && invoice ? (
        <MarkPaidModal
          invoice={invoice}
          onClose={() => setMarkingPaid(false)}
          onPaid={goBack}
        />
      ) : null}

      <ConfirmModal
        open={confirmingCancel}
        title={t("invoices.cancel")}
        description={t("invoices.confirm_cancel")}
        confirmLabel={t("invoices.cancel")}
        tone="danger"
        onConfirm={cancelInvoice}
        onClose={() => setConfirmingCancel(false)}
      />
    </Page>
  );
}
