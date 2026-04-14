import type { InputHTMLAttributes } from "react";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

const inputClass =
  "block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm placeholder:text-fg-subtle focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand";

export function Input({ label, className = "", id, ...rest }: Props) {
  const inputId = id ?? rest.name;
  return (
    <div className="flex flex-col gap-1">
      {label ? (
        <label htmlFor={inputId} className="text-sm font-medium text-fg-muted">
          {label}
        </label>
      ) : null}
      <input id={inputId} className={`${inputClass} ${className}`} {...rest} />
    </div>
  );
}
