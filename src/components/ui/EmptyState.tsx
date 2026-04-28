import type { ReactNode } from "react";

interface EmptyStateProps {
  title?: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ title, description, action, className = "" }: EmptyStateProps) {
  return (
    <div
      className={[
        "flex flex-col items-center justify-center text-center px-4 py-12 text-ink-3 text-[13px]",
        className,
      ].join(" ")}
    >
      {title ? <div className="text-ink font-medium mb-1.5">{title}</div> : null}
      {description ? <p className="max-w-md">{description}</p> : null}
      {action ? <div className="mt-4">{action}</div> : null}
    </div>
  );
}
