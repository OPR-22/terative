import type { ButtonHTMLAttributes } from "react";

import { Button as UiButton } from "../ui/Button";

type LegacyVariant = "primary" | "secondary" | "danger";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: LegacyVariant;
}

const variantMap = {
  primary: "primary",
  secondary: "default",
  danger: "danger",
} as const;

/**
 * Legacy Button kept as a shim around the new design-system Button.
 * Prefer importing `Button` from `components/ui` in new code.
 */
export function Button({ variant = "primary", ...rest }: Props) {
  return <UiButton variant={variantMap[variant]} {...rest} />;
}
