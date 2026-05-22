import { ChevronDown, ChevronRight, FileText, User } from "lucide-react";
import { DateTime } from "luxon";
import { useMemo, useState, type MouseEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import type { AuditDto } from "../../ipc";
import { Button } from "../ui/Button";
import { AuditChanges, changesFromMeta } from "./AuditChanges";

/**
 * Timeline-style activity feed — vertical line on the left with a colored
 * dot per event, day-section headers, time on the left, bold verb followed
 * by inline metadata details. Newest first; the "load more" button at the
 * bottom appends older entries via the optional `loadMore` prop.
 *
 * The component is shared across the dedicated Activity page, the dashboard
 * recent-activity card, the per-client tab and the per-invoice strip. The
 * `header` and `loadMore` props are opt-in so embedded surfaces (where there
 * is no pagination context) stay compact.
 *
 * Rows whose `metadata_json` carries a non-empty `changes` array are
 * expandable: clicking the row toggles a typed per-field diff via
 * `<AuditChanges>`. Optional client / invoice nav buttons let the user jump
 * to the related entity; clicks on those swallow propagation so the
 * accordion doesn't also toggle.
 */
export function ActivityList({
  items,
  header,
  loadMore,
  loadingMore = false,
}: {
  items: AuditDto[];
  /** When set, shows a header strip above the timeline ("Activité · 12 événements"). */
  header?: { title: string; count?: number };
  /** When set, shows a "Load more" button below the timeline. */
  loadMore?: () => void;
  /** Disables the load-more button while a fetch is in flight. */
  loadingMore?: boolean;
}) {
  const { t, i18n } = useTranslation();

  // Group by day in the user's locale so the boundaries match what they
  // see on their wall clock, not UTC.
  const sections = useMemo(() => groupByDay(items, i18n.language), [items, i18n.language]);

  if (items.length === 0) {
    return (
      <div className="px-5 py-6 text-center text-[12px] text-ink-3">
        {t("activity.empty")}
      </div>
    );
  }

  return (
    <div className="px-5 py-4">
      {header ? (
        <div className="mb-4 flex items-baseline justify-between">
          <h2 className="text-[14px] font-semibold text-ink">{header.title}</h2>
          {header.count != null ? (
            <span className="text-[10.5px] uppercase tracking-wider text-ink-3">
              {t("activity.events_count", { count: header.count })}
            </span>
          ) : null}
        </div>
      ) : null}

      {/* The timeline rail. `relative` anchors the vertical line drawn by
          the per-row dots, which carry their own `before:` pseudo-line. */}
      <ol className="relative">
        {sections.map((section) => (
          <li key={section.dayKey}>
            <DayHeader label={section.label} />
            <ol>
              {section.items.map((item) => (
                <ActivityRow key={item.id} item={item} locale={i18n.language} />
              ))}
            </ol>
          </li>
        ))}
      </ol>

      {loadMore ? (
        <div className="mt-4 flex justify-center">
          <Button onClick={loadMore} disabled={loadingMore}>
            {loadingMore ? t("common.loading") : t("activity.load_more")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}

function DayHeader({ label }: { label: string }) {
  // Hollow circle on the rail to mark a day boundary. The rail line is
  // drawn by the trailing pseudo-element so it visually connects to the
  // first event dot below.
  return (
    <div className="relative flex items-center gap-3 py-2 first:pt-0">
      <span className="relative z-10 grid place-items-center w-4 shrink-0">
        <span className="w-2.5 h-2.5 rounded-full border border-ink-3 bg-paper" />
      </span>
      <span className="text-[10.5px] uppercase tracking-wider text-ink-3">
        {label}
      </span>
    </div>
  );
}

function ActivityRow({ item, locale }: { item: AuditDto; locale: string }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);

  const meta = parseMeta(item.metadata_json);
  const key = `activity.event.${item.event_type.replace(/\./g, "_")}`;
  const fullTitle = t(key, { ...meta, defaultValue: item.event_type });
  const { verb, details } = splitTitle(fullTitle);

  const changes = changesFromMeta(item.metadata_json);
  const expandable = changes.length > 0;
  const time = DateTime.fromISO(item.occurred_at, { locale }).toFormat("HH:mm");

  const stop = (e: MouseEvent) => e.stopPropagation();
  const toggle = () => setOpen((v) => !v);

  return (
    <li className="relative">
      <div
        role={expandable ? "button" : undefined}
        tabIndex={expandable ? 0 : undefined}
        onClick={expandable ? toggle : undefined}
        onKeyDown={
          expandable
            ? (e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  toggle();
                }
              }
            : undefined
        }
        className={[
          "group flex items-start gap-3 py-2 text-left",
          expandable ? "cursor-pointer" : "cursor-default",
        ].join(" ")}
      >
        <span className="font-mono tabular-nums text-[11px] text-ink-3 w-10 shrink-0 mt-0.5">
          {time}
        </span>

        {/* Colored dot + trailing line drawn by ::after so consecutive rows
            visually share one continuous rail. */}
        <span className="relative z-10 grid place-items-center w-4 shrink-0 mt-1">
          <span
            className={[
              "w-2 h-2 rounded-full",
              colorFor(item.event_type),
            ].join(" ")}
          />
          <span
            aria-hidden
            className="absolute top-1/2 left-1/2 -translate-x-1/2 w-px bg-line-soft h-[calc(100%+0.5rem)] -z-10"
          />
        </span>

        <div className="min-w-0 flex-1">
          <div className="text-[12.5px] text-ink-2 leading-snug">
            <span className="font-semibold text-ink">{verb}</span>
            {details.length > 0 ? (
              <>
                <Bullet />
                {details.map((d, i) => (
                  <span key={i}>
                    {i > 0 ? <Bullet /> : null}
                    {d}
                  </span>
                ))}
              </>
            ) : null}
          </div>
        </div>

        {item.client_id ? (
          <NavIconButton
            title={t("activity.open_client")}
            onClick={(e) => {
              stop(e);
              navigate(`/clients/${item.client_id}`);
            }}
          >
            <User size={13} strokeWidth={1.5} />
          </NavIconButton>
        ) : null}
        {item.invoice_id ? (
          <NavIconButton
            title={t("activity.open_invoice")}
            onClick={(e) => {
              stop(e);
              navigate(`/invoices/${item.invoice_id}/edit`);
            }}
          >
            <FileText size={13} strokeWidth={1.5} />
          </NavIconButton>
        ) : null}

        {expandable ? (
          <span className="text-ink-3 shrink-0 mt-1">
            {open ? (
              <ChevronDown size={13} strokeWidth={1.5} />
            ) : (
              <ChevronRight size={13} strokeWidth={1.5} />
            )}
          </span>
        ) : null}
      </div>

      {expandable && open ? (
        // Indent the expanded sub-diff so it aligns with the title column,
        // past the time gutter + rail.
        <div className="pl-[calc(2.5rem+1rem+0.75rem)] pb-2">
          <AuditChanges changes={changes} />
        </div>
      ) : null}
    </li>
  );
}

function Bullet() {
  return <span className="mx-1.5 text-ink-3">·</span>;
}

function NavIconButton({
  title,
  onClick,
  children,
}: {
  title: string;
  onClick: (e: MouseEvent) => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      onClick={onClick}
      className="grid place-items-center w-6 h-6 rounded-sm text-ink-3 hover:bg-paper-2 hover:text-ink shrink-0 mt-0.5"
    >
      {children}
    </button>
  );
}

/** Splits an i18n event title on its first " — " / " · " separator: the
 *  leading verb gets bolded, the remainder becomes a list of bullet-joined
 *  detail spans. Both em-dash and middle-dot are accepted so existing
 *  strings keep rendering while we migrate. */
function splitTitle(s: string): { verb: string; details: string[] } {
  const sep = / [—·] /;
  const parts = s.split(sep);
  const [verb, ...rest] = parts;
  return { verb: (verb ?? s).trim(), details: rest.map((p) => p.trim()) };
}

/** Maps an event_type to a Tailwind background color for its timeline dot.
 *  Colors carry semantic weight — green = money in, blue = invoice
 *  progression, red = destructive, amber = backup/scheduled, gray = neutral
 *  changes. New event types fall back to gray. */
function colorFor(eventType: string): string {
  if (eventType.startsWith("payment.")) {
    return eventType === "payment.deleted" ? "bg-danger" : "bg-success";
  }
  if (eventType === "invoice.sent" || eventType === "invoice.finalized") {
    return "bg-info";
  }
  if (eventType === "invoice.cancelled") return "bg-danger";
  if (eventType.startsWith("invoice.")) return "bg-ink-3";
  if (eventType.startsWith("client.")) return "bg-ink-3";
  if (eventType.startsWith("backup.")) return "bg-warn";
  if (eventType.startsWith("catalog_item.") || eventType.startsWith("tax.")) {
    return "bg-ink-3";
  }
  return "bg-ink-3";
}

/** Best-effort parse of the audit row's `metadata_json` blob into an i18n
 *  interpolation bag. Never throws. */
function parseMeta(json: string): Record<string, unknown> {
  try {
    const v = JSON.parse(json) as unknown;
    return v && typeof v === "object" ? (v as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

interface DaySection {
  dayKey: string;
  label: string;
  items: AuditDto[];
}

/** Groups newest-first audit rows into per-day sections, preserving the
 *  incoming order within each day. Day key is the local calendar date so
 *  midnight rolls in the user's wall clock, not UTC. */
function groupByDay(items: AuditDto[], locale: string): DaySection[] {
  const sections: DaySection[] = [];
  for (const item of items) {
    const dt = DateTime.fromISO(item.occurred_at).setLocale(locale);
    const key = dt.isValid ? dt.toISODate() ?? item.occurred_at : item.occurred_at;
    const last = sections[sections.length - 1];
    if (last && last.dayKey === key) {
      last.items.push(item);
    } else {
      sections.push({
        dayKey: key,
        label: dayLabel(dt),
        items: [item],
      });
    }
  }
  return sections;
}

/** "Aujourd'hui · Mardi 17 mai" / "Hier · Lundi 16 mai" / "Mercredi 15 mai".
 *  Luxon's `toRelativeCalendar` returns a locale-aware "today" /
 *  "yesterday" string ("aujourd'hui" / "hier" in fr). We only use it for
 *  rows within the last 24h — older rows fall through to the full
 *  weekday+date. */
function dayLabel(dt: DateTime): string {
  if (!dt.isValid) return "";
  const full = dt.toFormat("cccc d LLLL");
  const today = DateTime.now().startOf("day");
  const diff = today.diff(dt.startOf("day"), "days").days;
  if (diff !== 0 && diff !== 1) return full;
  const relative = dt.toRelativeCalendar({ unit: "days" });
  return relative ? `${capitalize(relative)} · ${full}` : full;
}

function capitalize(s: string): string {
  return s.length === 0 ? s : s.charAt(0).toUpperCase() + s.slice(1);
}
