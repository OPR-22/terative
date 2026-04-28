import type { InputHTMLAttributes } from "react";

import { Field, Input as UiInput } from "../ui/Input";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

/**
 * Legacy Input kept as a shim around the new design-system Field+Input.
 * Prefer importing `Input` and `Field` from `components/ui` in new code.
 */
export function Input({ label, id, ...rest }: Props) {
  const inputId = id ?? rest.name;
  if (label) {
    return (
      <Field label={label} htmlFor={inputId}>
        <UiInput id={inputId} {...rest} />
      </Field>
    );
  }
  return <UiInput id={inputId} {...rest} />;
}
