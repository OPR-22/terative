import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";

type Variant = "default" | "primary" | "accent" | "ghost" | "danger";
type Size = "sm" | "md";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  iconOnly?: boolean;
  leadingIcon?: ReactNode;
  trailingIcon?: ReactNode;
}

const base =
  "inline-flex items-center justify-center gap-1.5 font-medium leading-none border rounded-sm transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-soft";

const variants: Record<Variant, string> = {
  default:
    "bg-paper-2 border-line text-ink hover:bg-paper-3 active:bg-paper-3",
  primary: "bg-fill border-fill text-fill-fg hover:bg-fill-hover",
  accent: "bg-accent border-accent text-white hover:opacity-90",
  ghost:
    "bg-transparent border-transparent text-ink-2 hover:bg-paper-2 hover:text-ink",
  danger:
    "bg-paper-2 border-line text-danger hover:bg-danger-soft hover:border-danger-soft",
};

const sizes: Record<Size, string> = {
  sm: "px-2.5 py-1.5 text-[12px]",
  md: "px-3.5 py-2 text-[13px]",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant = "default",
    size = "md",
    iconOnly = false,
    leadingIcon,
    trailingIcon,
    className = "",
    children,
    type = "button",
    ...rest
  },
  ref,
) {
  const padding = iconOnly ? (size === "sm" ? "p-1.5" : "p-2") : sizes[size];
  return (
    <button
      ref={ref}
      type={type}
      className={[base, variants[variant], padding, className].join(" ")}
      {...rest}
    >
      {leadingIcon}
      {children}
      {trailingIcon}
    </button>
  );
});
