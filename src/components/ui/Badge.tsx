import type { ReactNode } from "react";

export type BadgeKind =
  | "neutral"
  | "outline"
  | "draft"
  | "final"
  | "sent"
  | "cancel"
  | "paid"
  | "unpaid"
  | "partial"
  | "overdue"
  | "info"
  | "ok"
  | "warn"
  | "danger";

interface BadgeProps {
  kind?: BadgeKind;
  dot?: boolean;
  className?: string;
  children: ReactNode;
}

const variants: Record<BadgeKind, string> = {
  neutral: "bg-paper-3 text-ink-2",
  outline: "bg-transparent border border-line text-ink-2",
  draft: "bg-paper-3 text-ink-3",
  final: "bg-info-soft text-info-ink",
  sent: "bg-info-soft text-info-ink",
  cancel: "bg-paper-3 text-ink-3 line-through",
  paid: "bg-ok-soft text-ok-ink",
  unpaid: "bg-warn-soft text-warn-ink",
  partial: "bg-info-soft text-info-ink",
  overdue: "bg-danger-soft text-danger",
  info: "bg-info-soft text-info-ink",
  ok: "bg-ok-soft text-ok-ink",
  warn: "bg-warn-soft text-warn-ink",
  danger: "bg-danger-soft text-danger",
};

export function Badge({ kind = "neutral", dot = false, className = "", children }: BadgeProps) {
  return (
    <span
      className={[
        "inline-flex items-center gap-1 px-1.5 py-[2px] rounded-sm text-[11px] font-medium leading-[1.4] tabular",
        variants[kind],
        className,
      ].join(" ")}
    >
      {dot ? <span className="w-[5px] h-[5px] rounded-full bg-current" /> : null}
      {children}
    </span>
  );
}
