import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../common/Button";
import { Input } from "../common/Input";
import { MoneyInput } from "../common/MoneyInput";
import { useMoneyFormat } from "../../lib/money";
import {
  ipc,
  type InvoiceDto,
  type NewPaymentDto,
  type PaymentMethodDto,
} from "../../ipc";
import { usePaymentStore } from "../../stores/paymentStore";
import { useSettingsStore } from "../../stores/settingsStore";

type PaymentMethodKind = PaymentMethodDto["kind"];

const METHOD_KINDS: PaymentMethodKind[] = [
  "BankTransfer",
  "Cash",
  "Check",
  "Card",
  "Other",
];

const today = () => new Date().toISOString().slice(0, 10);

interface Props {
  invoice: InvoiceDto;
  onClose: () => void;
  onPaid: () => void;
}

export function MarkPaidModal({ invoice, onClose, onPaid }: Props) {
  const { t } = useTranslation();
  const { record } = usePaymentStore();
  const { snapshot } = useSettingsStore();
  const { formatMinor } = useMoneyFormat();
  const currency = snapshot?.currency;
  const currencyCode = currency?.code ?? invoice.currency;

  const [loading, setLoading] = useState(true);
  const [amountDueCents, setAmountDueCents] = useState(0);
  const [fullyPaid, setFullyPaid] = useState(false);

  const [amountCents, setAmountCents] = useState(0);
  const [date, setDate] = useState(today());
  const [methodKind, setMethodKind] =
    useState<PaymentMethodKind>("BankTransfer");
  const [methodDetail, setMethodDetail] = useState("");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");

  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingListOutstanding()
      .then((rows) => {
        if (cancelled) return;
        const row = rows.find((r) => r.invoice_id === invoice.id);
        if (!row || row.amount_due.amount_minor <= 0) {
          setFullyPaid(true);
          setAmountDueCents(0);
          setAmountCents(0);
        } else {
          setAmountDueCents(row.amount_due.amount_minor);
          setAmountCents(row.amount_due.amount_minor);
        }
        setLoading(false);
      })
      .catch((e) => {
        if (!cancelled) {
          setErr(String(e));
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [invoice.id]);

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

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    if (amountCents <= 0) {
      setErr(t("payments.err_non_positive"));
      return;
    }
    if (methodKind === "Other" && methodDetail.trim() === "") {
      setErr(t("payments.err_method_detail"));
      return;
    }
    setSubmitting(true);
    try {
      const payload: NewPaymentDto = {
        client_id: invoice.client_id,
        date,
        amount: { amount_minor: amountCents, currency: currencyCode },
        method: buildMethod(),
        reference: reference || null,
        allocations: [
          {
            invoice_id: invoice.id,
            amount: { amount_minor: amountCents, currency: currencyCode },
          },
        ],
        notes: notes || null,
      };
      await record(payload);
      onPaid();
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
        className="my-8 w-full max-w-lg rounded-card bg-surface p-6 shadow-card"
        onSubmit={submit}
      >
        <h2 className="mb-4 text-lg font-bold text-fg">
          {t("invoices.mark_paid_title")}
          {invoice.number != null ? ` #${invoice.number}` : ""}
        </h2>
        {loading ? (
          <p className="text-sm text-fg-muted">{t("common.loading")}</p>
        ) : fullyPaid ? (
          <>
            <p className="mb-4 text-sm text-fg-muted">
              {t("invoices.mark_paid_already_paid")}
            </p>
            <div className="flex justify-end">
              <Button type="button" onClick={onClose}>
                {t("common.close")}
              </Button>
            </div>
          </>
        ) : (
          <>
            <p className="mb-3 text-xs text-fg-subtle">
              {t("invoices.mark_paid_amount_due")}:{" "}
              {formatMinor(amountDueCents, currencyCode)}
            </p>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {currency ? (
                <MoneyInput
                  label={t("payments.amount") ?? ""}
                  valueMinor={amountCents}
                  currency={currency}
                  onChangeMinor={setAmountCents}
                />
              ) : null}
              <Input
                type="date"
                label={t("common.date") ?? ""}
                value={date}
                onChange={(e) => setDate(e.target.value)}
              />
              <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
                {t("payments.method")}
                <select
                  className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
                  value={methodKind}
                  onChange={(e) =>
                    setMethodKind(e.target.value as PaymentMethodKind)
                  }
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
            {err ? <p className="mt-3 text-sm text-danger">{err}</p> : null}
            <div className="mt-5 flex justify-end gap-2">
              <Button variant="secondary" type="button" onClick={onClose}>
                {t("common.cancel")}
              </Button>
              <Button type="submit" disabled={submitting}>
                {t("invoices.mark_paid_confirm")}
              </Button>
            </div>
          </>
        )}
      </form>
    </div>
  );
}
