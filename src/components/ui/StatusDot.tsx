type Status = "ok" | "warn" | "err" | "idle";

const colors: Record<Status, string> = {
  ok: "bg-ok",
  warn: "bg-warn",
  err: "bg-danger",
  idle: "bg-ink-4",
};

export function StatusDot({ status, className = "" }: { status: Status; className?: string }) {
  return (
    <span
      className={["inline-block w-1.5 h-1.5 rounded-full", colors[status], className].join(" ")}
    />
  );
}
