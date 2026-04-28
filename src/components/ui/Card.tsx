import type { ReactNode } from "react";

interface CardProps {
  className?: string;
  flat?: boolean;
  children: ReactNode;
}

export function Card({ className = "", flat = false, children }: CardProps) {
  return (
    <div
      className={[
        "rounded-card border border-line",
        flat ? "bg-paper-2" : "bg-paper",
        className,
      ].join(" ")}
    >
      {children}
    </div>
  );
}

interface CardHeadProps {
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  className?: string;
  children?: ReactNode;
}

export function CardHead({
  title,
  subtitle,
  actions,
  className = "",
  children,
}: CardHeadProps) {
  return (
    <div
      className={[
        "flex items-center justify-between gap-3 px-5 py-3.5 border-b border-line-soft",
        className,
      ].join(" ")}
    >
      {children ?? (
        <div className="min-w-0">
          {title ? (
            <div className="text-[13px] font-medium text-ink truncate">{title}</div>
          ) : null}
          {subtitle ? (
            <div className="text-[12px] text-ink-3 truncate">{subtitle}</div>
          ) : null}
        </div>
      )}
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </div>
  );
}

interface CardBodyProps {
  className?: string;
  children: ReactNode;
}

export function CardBody({ className = "", children }: CardBodyProps) {
  return <div className={["px-5 py-4", className].join(" ")}>{children}</div>;
}
