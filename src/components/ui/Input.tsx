import {
  forwardRef,
  type InputHTMLAttributes,
  type SelectHTMLAttributes,
  type TextareaHTMLAttributes,
} from "react";

const fieldBase =
  "w-full bg-paper border border-line text-ink rounded-sm px-2.5 py-2 text-[13px] outline-none transition-colors placeholder:text-ink-4 focus:border-accent focus:ring-2 focus:ring-accent-soft disabled:bg-paper-2 disabled:text-ink-3";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  mono?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { mono = false, className = "", ...rest },
  ref,
) {
  return (
    <input
      ref={ref}
      className={[fieldBase, mono ? "font-mono tabular" : "", className].join(" ")}
      {...rest}
    />
  );
});

export interface TextareaProps
  extends TextareaHTMLAttributes<HTMLTextAreaElement> {}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  function Textarea({ className = "", rows = 3, ...rest }, ref) {
    return (
      <textarea
        ref={ref}
        rows={rows}
        className={[fieldBase, "resize-y", className].join(" ")}
        {...rest}
      />
    );
  },
);

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select(
  { className = "", children, ...rest },
  ref,
) {
  return (
    <select
      ref={ref}
      className={[fieldBase, "appearance-none pr-8 cursor-pointer", className].join(
        " ",
      )}
      {...rest}
    >
      {children}
    </select>
  );
});

export interface FieldProps {
  label?: string;
  help?: string;
  htmlFor?: string;
  className?: string;
  children: React.ReactNode;
}

export function Field({ label, help, htmlFor, className = "", children }: FieldProps) {
  return (
    <div className={["flex flex-col gap-1.5", className].join(" ")}>
      {label ? (
        <label htmlFor={htmlFor} className="text-[12px] font-medium text-ink-3">
          {label}
        </label>
      ) : null}
      {children}
      {help ? <p className="text-[11px] text-ink-4">{help}</p> : null}
    </div>
  );
}
