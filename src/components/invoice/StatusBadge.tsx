import { useTranslation } from "react-i18next";

import { Badge, type BadgeKind } from "../ui/Badge";
import type { InvoiceStatusDto } from "../../ipc";

const kindMap: Record<InvoiceStatusDto, BadgeKind> = {
  Draft: "draft",
  Finalized: "final",
  Sent: "sent",
  Cancelled: "cancel",
};

export function StatusBadge({ status }: { status: InvoiceStatusDto }) {
  const { t } = useTranslation();
  return (
    <Badge dot kind={kindMap[status]}>
      {t(`invoices.status_${status.toLowerCase()}`)}
    </Badge>
  );
}
