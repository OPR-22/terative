import { useTranslation } from "react-i18next";

import { Badge, type BadgeKind } from "../ui/Badge";
import type { DerivedPaymentStatusDto, InvoiceStatusDto } from "../../ipc";

const kindMap: Record<DerivedPaymentStatusDto, BadgeKind> = {
  Draft: "draft",
  Unpaid: "unpaid",
  Partial: "partial",
  Paid: "paid",
  Overdue: "overdue",
  Cancelled: "cancel",
};

interface Props {
  paymentStatus: DerivedPaymentStatusDto | null;
  rawStatus: InvoiceStatusDto;
}

export function PaymentStatusBadge({ paymentStatus, rawStatus }: Props) {
  const { t } = useTranslation();
  const effective: DerivedPaymentStatusDto = paymentStatus ?? fallback(rawStatus);
  if (effective === "Draft") {
    return <span className="text-ink-4">—</span>;
  }
  return (
    <Badge dot kind={kindMap[effective]}>
      {t(`invoices.payment_status_${effective.toLowerCase()}`)}
    </Badge>
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
