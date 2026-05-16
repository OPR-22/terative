import {
  ChevronDown,
  ChevronRight,
  Database,
  FileText,
  Package,
  Percent,
  Send,
  User,
  Wallet,
} from "lucide-react";
import { useState, type ComponentType } from "react";
import { useTranslation } from "react-i18next";

import type { AuditDto } from "../../ipc";
import { formatRelativeTime } from "../../lib/datetime";
import { AuditChanges, changesFromMeta } from "./AuditChanges";

/** Picks the lucide icon for an `event_type` by its `<entity>.` prefix. */
function iconFor(eventType: string): ComponentType<{ size?: number; strokeWidth?: number }> {
  if (eventType.startsWith("payment.")) return Wallet;
  if (eventType === "invoice.sent") return Send;
  if (eventType.startsWith("invoice.")) return FileText;
  if (eventType.startsWith("client.")) return User;
  if (eventType.startsWith("catalog_item.")) return Package;
  if (eventType.startsWith("tax.")) return Percent;
  if (eventType.startsWith("backup.")) return Database;
  return FileText;
}

/** Parses the opaque `metadata_json` blob; never throws. */
function parseMeta(json: string): Record<string, unknown> {
  try {
    const v = JSON.parse(json) as unknown;
    return v && typeof v === "object" ? (v as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

/**
 * Renders a list of activity-log rows — shared by the dashboard "Recent
 * activity" card, the per-client tab and the per-invoice strip. Each row's
 * label comes from `activity.event.<event_type>` with the event's metadata
 * spread in as interpolation values; an unknown `event_type` falls back to
 * the raw string so a new backend event is still legible before its i18n
 * key is added.
 *
 * Rows whose `metadata_json` carries a non-empty `changes` array become
 * expandable: collapsed shows just the title, expanded reveals the typed
 * field-by-field diff via [`AuditChanges`].
 */
export function ActivityList({ items }: { items: AuditDto[] }) {
  const { t } = useTranslation();

  if (items.length === 0) {
    return (
      <div className="px-5 py-6 text-center text-[12px] text-ink-3">
        {t("activity.empty")}
      </div>
    );
  }

  return (
    <div className="py-1.5">
      {items.map((a, i, arr) => (
        <ActivityRow key={a.id} item={a} isLast={i === arr.length - 1} />
      ))}
    </div>
  );
}

function ActivityRow({ item, isLast }: { item: AuditDto; isLast: boolean }) {
  const { t, i18n } = useTranslation();
  const [open, setOpen] = useState(false);

  const Ic = iconFor(item.event_type);
  const meta = parseMeta(item.metadata_json);
  // i18next nests on ".", so the dotted event_type maps to an underscored
  // key: "invoice.finalized" → activity.event.invoice_finalized.
  const key = `activity.event.${item.event_type.replace(/\./g, "_")}`;
  const title = t(key, { ...meta, defaultValue: item.event_type });

  const changes = changesFromMeta(item.metadata_json);
  const expandable = changes.length > 0;

  return (
    <div className={isLast ? "" : "border-b border-line-soft"}>
      <button
        type="button"
        onClick={() => expandable && setOpen((v) => !v)}
        disabled={!expandable}
        className={[
          "w-full flex items-start gap-3 px-5 py-2.5 text-left",
          expandable ? "cursor-pointer hover:bg-paper-2" : "cursor-default",
        ].join(" ")}
      >
        <span className="grid place-items-center w-6 h-6 rounded-sm bg-paper-2 text-ink-2 shrink-0">
          <Ic size={13} strokeWidth={1.5} />
        </span>
        <div className="min-w-0 flex-1">
          <div className="text-[12.5px] font-medium text-ink truncate">
            {title}
          </div>
        </div>
        <div className="text-[11px] text-ink-3 whitespace-nowrap">
          {formatRelativeTime(item.occurred_at, i18n.language)}
        </div>
        {expandable ? (
          <span className="text-ink-3 shrink-0 pt-0.5">
            {open ? (
              <ChevronDown size={13} strokeWidth={1.5} />
            ) : (
              <ChevronRight size={13} strokeWidth={1.5} />
            )}
          </span>
        ) : null}
      </button>
      {expandable && open ? <AuditChanges changes={changes} /> : null}
    </div>
  );
}
