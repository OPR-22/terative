import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../../stores/toastStore";

import { Button } from "../ui/Button";
import { Field, Input, Select } from "../ui/Input";
import { Modal } from "../ui/Modal";
import { MoneyInput } from "../common/MoneyInput";
import { useMoneyFormat } from "../../lib/money";
import {
  ipc,
  type InvoiceDto,
  type NewPaymentDto,
  type PaymentMethodDto,
} from "../../ipc";
import { usePaymentStore } from "../../stores/paymentStore";

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
  const { formatMinor } = useMoneyFormat();
  // Strict silos: a payment against an invoice must be in the invoice's
  // own currency, not the org's default. `invoice.total.currency` carries
  // the full CurrencyConfigDto inline, so we never need a catalog lookup.
  const currency = invoice.total.currency;
  const currencyCode = currency.code;

  const [loading, setLoading] = useState(true);
  const [amountDueCents, setAmountDueCents] = useState(0);
  const [fullyPaid, setFullyPaid] = useState(false);

  const [amountCents, setAmountCents] = useState(0);
  const [date, setDate] = useState(today());
  const [methodKind, setMethodKind] = useState<PaymentMethodKind>("BankTransfer");
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
        if (!row || row.amount_due.amount <= 0) {
          setFullyPaid(true);
          setAmountDueCents(0);
          setAmountCents(0);
        } else {
          setAmountDueCents(row.amount_due.amount);
          setAmountCents(row.amount_due.amount);
        }
        setLoading(false);
      })
      .catch((e) => {
        if (!cancelled) {
          toast.error(e);
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [invoice.id]);

  const buildMethod = (): PaymentMethodDto => {
    switch (methodKind) {
      case "Other":
        return { kind: "Other", detail: methodDetail };
      default:
        return { kind: methodKind };
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
    const money = { amount: amountCents, currency };
    setSubmitting(true);
    try {
      const payload: NewPaymentDto = {
        client_id: invoice.client_id,
        date,
        amount: money,
        method: buildMethod(),
        reference: reference || null,
        allocations: [
          {
            invoice_id: invoice.id,
            amount: money,
          },
        ],
        notes: notes || null,
      };
      await record(payload);
      onPaid();
      onClose();
    } catch (e) {
      toast.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  const title = `${t("invoices.mark_paid_title")}${
    invoice.number != null ? ` #${invoice.number}` : ""
  }`;

  return (
    <Modal
      open
      onClose={onClose}
      title={title}
      subtitle={
        !loading && !fullyPaid
          ? `${t("invoices.mark_paid_amount_due")}: ${formatMinor(amountDueCents, currencyCode)}`
          : undefined
      }
      width={560}
      footer={
        loading || fullyPaid ? (
          <Button type="button" onClick={onClose}>
            {t("common.close")}
          </Button>
        ) : (
          <>
            <Button type="button" onClick={onClose}>
              {t("common.cancel")}
            </Button>
            <Button
              type="submit"
              form="mark-paid-form"
              variant="primary"
              disabled={submitting}
            >
              {t("invoices.mark_paid_confirm")}
            </Button>
          </>
        )
      }
    >
      {loading ? (
        <p className="text-[13px] text-ink-3">{t("common.loading")}</p>
      ) : fullyPaid ? (
        <p className="text-[13px] text-ink-3">
          {t("invoices.mark_paid_already_paid")}
        </p>
      ) : (
        <form id="mark-paid-form" onSubmit={submit} className="grid gap-3.5 grid-cols-2">
          <Field label={t("payments.amount")}>
            <MoneyInput
              valueMinor={amountCents}
              currency={currency}
              onChangeMinor={setAmountCents}
            />
          </Field>
          <Field label={t("common.date")}>
            <Input
              mono
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
            />
          </Field>
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
          <Field label={t("common.notes")}>
            <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
          </Field>
          {err ? (
            <p className="col-span-2 text-[13px] text-danger">{err}</p>
          ) : null}
        </form>
      )}
    </Modal>
  );
}
