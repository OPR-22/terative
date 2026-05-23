import type { ReactNode } from "react";
import { Check } from "lucide-react";

interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  children?: ReactNode;
}

export function Checkbox({
  checked,
  onChange,
  disabled,
  className = "",
  children,
}: CheckboxProps) {
  return (
    <label
      className={[
        "inline-flex items-center gap-2 text-[13px] cursor-pointer select-none",
        disabled ? "opacity-50 cursor-not-allowed" : "",
        className,
      ].join(" ")}
    >
      <span
        onClick={(e) => {
          e.preventDefault();
          if (!disabled) onChange(!checked);
        }}
        className={[
          "inline-grid place-items-center w-[14px] h-[14px] rounded-[3px] border-[1.5px] transition-colors flex-none",
          checked
            ? "bg-ink border-ink text-paper"
            : "bg-paper border-ink-4 hover:border-ink-3",
        ].join(" ")}
      >
        {checked ? <Check size={10} strokeWidth={3} /> : null}
      </span>
      {children}
    </label>
  );
}
