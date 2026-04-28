import type { ReactNode } from "react";

export interface TabOption<T extends string = string> {
  id: T;
  label: ReactNode;
  count?: number | null;
}

interface TabsProps<T extends string> {
  options: TabOption<T>[];
  value: T;
  onChange: (id: T) => void;
  className?: string;
}

export function Tabs<T extends string>({
  options,
  value,
  onChange,
  className = "",
}: TabsProps<T>) {
  return (
    <div
      className={[
        "flex border-b border-line",
        className,
      ].join(" ")}
    >
      {options.map((o) => {
        const active = o.id === value;
        return (
          <button
            key={o.id}
            type="button"
            onClick={() => onChange(o.id)}
            className={[
              "px-4 py-2.5 text-[13px] -mb-px border-b-2 transition-colors inline-flex items-center gap-2",
              active
                ? "border-accent text-ink font-medium"
                : "border-transparent text-ink-3 hover:text-ink",
            ].join(" ")}
          >
            {o.label}
            {o.count != null ? (
              <span
                className={[
                  "rounded-sm px-1.5 py-[1px] font-mono text-[10px] tabular",
                  active ? "bg-accent-soft text-accent-ink" : "bg-paper-3 text-ink-3",
                ].join(" ")}
              >
                {o.count}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
