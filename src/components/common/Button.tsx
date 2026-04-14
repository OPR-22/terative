import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "secondary" | "danger";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
}

const base =
  "inline-flex items-center justify-center rounded-field px-3 py-2 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 focus:outline-none focus:ring-2 focus:ring-brand focus:ring-offset-1";

const variantClasses: Record<Variant, string> = {
  primary: "bg-brand text-brand-fg hover:bg-brand-hover",
  secondary: "bg-surface-muted text-fg hover:bg-border",
  danger: "bg-danger text-danger-fg hover:bg-danger-hover",
};

export function Button({
  variant = "primary",
  className = "",
  ...rest
}: Props) {
  return (
    <button
      type="button"
      className={`${base} ${variantClasses[variant]} ${className}`}
      {...rest}
    />
  );
}
