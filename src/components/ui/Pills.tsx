import type { ReactNode } from "react";

export interface PillOption<T extends string = string> {
  id: T;
  label: ReactNode;
  count?: number | null;
}

interface PillsProps<T extends string> {
  options: PillOption<T>[];
  value: T;
  onChange?: (id: T) => void;
  className?: string;
}

export function Pills<T extends string>({
  options,
  value,
  onChange,
  className = "",
}: PillsProps<T>) {
  return (
    <div
      className={[
        "inline-flex p-[3px] gap-[2px] bg-paper-2 border border-line rounded-sm",
        className,
      ].join(" ")}
    >
      {options.map((o) => {
        const active = o.id === value;
        return (
          <button
            key={o.id}
            type="button"
            onClick={() => onChange?.(o.id)}
            className={[
              "inline-flex items-center gap-1.5 rounded-sm px-3 py-[5px] text-[12px] transition-colors",
              active
                ? "bg-paper text-ink border border-line-soft shadow-sm font-medium"
                : "text-ink-3 hover:text-ink",
            ].join(" ")}
          >
            {o.label}
            {o.count != null ? (
              <span
                className={[
                  "font-mono text-[10px] tabular",
                  active ? "text-ink-3" : "text-ink-4",
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
