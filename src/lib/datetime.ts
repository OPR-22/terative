import { DateTime } from "luxon";

/**
 * Locale-aware "2 hours ago" style relative time.
 *
 * Used by the activity log surfaces (dashboard card, client tab, invoice
 * strip) to render `occurred_at` timestamps. Falls back to the raw input
 * if it isn't a parseable ISO string.
 */
export function formatRelativeTime(iso: string, locale?: string): string {
  const dt = DateTime.fromISO(iso, locale ? { locale } : undefined);
  if (!dt.isValid) return iso;
  return dt.toRelative() ?? iso;
}
