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
  "inline-flex items-center justify-center gap-1.5 font-medium leading-none border rounded-sm transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-soft";

const variants: Record<Variant, string> = {
  default:
    "bg-paper-2 border-line text-ink hover:bg-paper-3 active:bg-paper-3",
  primary: "bg-ink border-ink text-paper hover:bg-ink-2",
  accent: "bg-accent border-accent text-white hover:opacity-90",
  ghost:
    "bg-transparent border-transparent text-ink-2 hover:bg-paper-2 hover:text-ink",
  danger:
    "bg-paper-2 border-line text-danger hover:bg-danger-soft hover:border-danger-soft",
};

const sizes: Record<Size, string> = {
  sm: "px-2 py-1 text-[11.5px]",
  md: "px-3 py-1.5 text-[12.5px]",
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
  const padding = iconOnly ? (size === "sm" ? "p-1" : "p-1.5") : sizes[size];
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
