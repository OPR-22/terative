import { ArrowRight, MinusCircle, PencilLine, PlusCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

/**
 * Frontend mirror of the backend `FieldChange` enum. The bindings ship the
 * payload as opaque `metadata_json: string`, so each call site `JSON.parse`s
 * and narrows to this shape before handing it to `<AuditChanges>`.
 */
export type FieldChange =
  | { kind: "scalar"; field: string; from: unknown; to: unknown }
  | { kind: "number"; field: string; from: unknown; to: unknown }
  | {
      kind: "money";
      field: string;
      from: MoneyValue | null;
      to: MoneyValue | null;
    }
  | { kind: "collection"; field: string; from_count: number; to_count: number }
  | {
      kind: "indexed_collection";
      field: string;
      added: IndexedDelta[];
      removed: IndexedDelta[];
      changed: IndexedDelta[];
    };

export type MoneyValue = { currency: string; amount: string };
/** `label` is a human-friendly rendering of `key` (e.g. "#1001" for an
 *  invoice UUID). Optional — keys that are already user-readable (like
 *  currency codes) come back with `label` undefined. */
export type IndexedDelta = {
  key: string;
  label?: string;
  from?: unknown;
  to?: unknown;
};

/** Best-effort parse of an audit row's `metadata_json` into a `changes` list. */
export function changesFromMeta(json: string): FieldChange[] {
  try {
    const parsed = JSON.parse(json) as { changes?: unknown };
    return Array.isArray(parsed?.changes) ? (parsed.changes as FieldChange[]) : [];
  } catch {
    return [];
  }
}

/**
 * Renders a `Vec<FieldChange>` payload as a compact list of "field: from → to"
 * lines. Plugs into the activity row's accordion body.
 */
export function AuditChanges({ changes }: { changes: FieldChange[] }) {
  const { t } = useTranslation();
  if (changes.length === 0) {
    return (
      <div className="px-5 pb-3 text-[11px] text-ink-3 italic">
        {t("audit.no_changes")}
      </div>
    );
  }
  return (
    <ul className="px-5 pb-3 space-y-1.5">
      {changes.map((c, i) => (
        <li key={`${c.field}-${i}`} className="text-[11.5px] text-ink-2">
          <ChangeLine change={c} />
        </li>
      ))}
    </ul>
  );
}

function ChangeLine({ change }: { change: FieldChange }) {
  const { t } = useTranslation();
  const label = t(`audit.field.${change.field}`, { defaultValue: change.field });

  switch (change.kind) {
    case "scalar":
    case "number":
      return (
        <FromTo label={label} from={fmtScalar(change.from)} to={fmtScalar(change.to)} />
      );
    case "money":
      return (
        <FromTo label={label} from={fmtMoney(change.from)} to={fmtMoney(change.to)} />
      );
    case "collection":
      return (
        <FromTo
          label={label}
          from={String(change.from_count)}
          to={String(change.to_count)}
          tail={t("audit.entries_suffix")}
        />
      );
    case "indexed_collection":
      return (
        <div>
          <span className="font-medium text-ink">{label}</span>
          <ul className="mt-1 ml-4 space-y-0.5">
            {change.added.map((d) => (
              <li
                key={`add-${d.key}`}
                className="flex items-center gap-1.5 text-success"
              >
                <PlusCircle size={11} strokeWidth={1.5} />
                <span className="font-medium">{deltaLabel(d)}</span>
                <span className="text-ink-3">{fmtAny(d.to)}</span>
              </li>
            ))}
            {change.removed.map((d) => (
              <li
                key={`rem-${d.key}`}
                className="flex items-center gap-1.5 text-danger"
              >
                <MinusCircle size={11} strokeWidth={1.5} />
                <span className="font-medium line-through">{deltaLabel(d)}</span>
                <span className="text-ink-3 line-through">{fmtAny(d.from)}</span>
              </li>
            ))}
            {change.changed.map((d) => (
              <li
                key={`chg-${d.key}`}
                className="flex items-center gap-1.5 text-ink-2"
              >
                <PencilLine size={11} strokeWidth={1.5} />
                <span className="font-medium">{deltaLabel(d)}</span>
                <span className="text-ink-3">{fmtAny(d.from)}</span>
                <ArrowRight size={10} strokeWidth={1.5} className="text-ink-3" />
                <span>{fmtAny(d.to)}</span>
              </li>
            ))}
          </ul>
        </div>
      );
  }
}

/** Prefer the human label resolved at write time (e.g. `"#1001"`); fall
 *  back to the raw key when none was supplied (e.g. currency codes for
 *  catalog-item prices). */
function deltaLabel(d: IndexedDelta): string {
  return d.label ?? d.key;
}

function FromTo({
  label,
  from,
  to,
  tail,
}: {
  label: string;
  from: string;
  to: string;
  tail?: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className="font-medium text-ink">{label}:</span>
      <span className="text-ink-3">{from}</span>
      <ArrowRight size={10} strokeWidth={1.5} className="text-ink-3" />
      <span>{to}</span>
      {tail ? <span className="text-ink-3">{tail}</span> : null}
    </span>
  );
}

/** Format any JSON-ish value for inline display: strings unquoted, null as
 *  an em-dash, objects via `fmtAny` recursion (handles MoneyValue). */
function fmtScalar(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "string") return v;
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  return fmtAny(v);
}

function fmtMoney(m: MoneyValue | null): string {
  if (m === null) return "—";
  return `${m.amount} ${m.currency}`;
}

function fmtAny(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "string") return v;
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  // MoneyValue ducktype.
  if (
    typeof v === "object" &&
    v !== null &&
    "amount" in v &&
    "currency" in v
  ) {
    const m = v as MoneyValue;
    return `${m.amount} ${m.currency}`;
  }
  try {
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
}
