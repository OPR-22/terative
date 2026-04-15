import { useTranslation } from "react-i18next";

import type { DerivedPaymentStatusDto, InvoiceStatusDto } from "../../ipc";

const classes: Record<DerivedPaymentStatusDto, string> = {
  Draft: "bg-status-draft-bg text-status-draft-fg",
  Unpaid: "bg-status-finalized-bg text-status-finalized-fg",
  Partial: "bg-amber-100 text-amber-900 dark:bg-amber-900/40 dark:text-amber-200",
  Paid: "bg-status-sent-bg text-status-sent-fg",
  Overdue: "bg-status-cancelled-bg text-status-cancelled-fg",
  Cancelled: "bg-status-cancelled-bg text-status-cancelled-fg",
};

interface Props {
  /** Derived payment status from the backend. If null, falls back to rawStatus. */
  paymentStatus: DerivedPaymentStatusDto | null;
  /** Raw lifecycle status, used as a fallback when paymentStatus is null. */
  rawStatus: InvoiceStatusDto;
}

/**
 * Shows the combined payment state in a single pill. The backend's
 * `DerivedPaymentStatus` already collapses raw status + payment state into one
 * of six outcomes, so this component just picks a colour and a translation key.
 * When the backend didn't populate `payment_status` (write paths), we fall
 * back to the raw status so the badge still says something.
 *
 * Exception: a `Draft` invoice has no payment state yet — there's nothing to
 * pay until it's issued — so we render a muted em-dash instead of a pill.
 */
export function PaymentStatusBadge({ paymentStatus, rawStatus }: Props) {
  const { t } = useTranslation();
  const effective: DerivedPaymentStatusDto = paymentStatus ?? fallback(rawStatus);
  if (effective === "Draft") {
    return <span className="text-fg-subtle">—</span>;
  }
  return (
    <span
      className={`inline-flex rounded-pill px-2 py-0.5 text-xs font-medium ${classes[effective]}`}
    >
      {t(`invoices.payment_status_${effective.toLowerCase()}`)}
    </span>
  );
}

function fallback(raw: InvoiceStatusDto): DerivedPaymentStatusDto {
  switch (raw) {
    case "Draft":
      return "Draft";
    case "Cancelled":
      return "Cancelled";
    case "Finalized":
    case "Sent":
      return "Unpaid";
  }
}
