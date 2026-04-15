import { useTranslation } from "react-i18next";
import type { InvoiceStatusDto } from "../../ipc";

const classes: Record<InvoiceStatusDto, string> = {
  Draft: "bg-status-draft-bg text-status-draft-fg",
  Finalized: "bg-status-finalized-bg text-status-finalized-fg",
  Sent: "bg-status-sent-bg text-status-sent-fg",
  Cancelled: "bg-status-cancelled-bg text-status-cancelled-fg",
};

export function StatusBadge({ status }: { status: InvoiceStatusDto }) {
  const { t } = useTranslation();
  return (
    <span
      className={`inline-flex rounded-pill px-2 py-0.5 text-xs font-medium ${classes[status]}`}
    >
      {t(`invoices.status_${status.toLowerCase()}`)}
    </span>
  );
}
