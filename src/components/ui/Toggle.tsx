interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  className?: string;
}

export function Toggle({ checked, onChange, disabled, label, className = "" }: ToggleProps) {
  return (
    <label
      className={[
        "inline-flex items-center gap-2.5 text-[13px] cursor-pointer select-none",
        disabled ? "opacity-50 cursor-not-allowed" : "",
        className,
      ].join(" ")}
    >
      <span
        role="switch"
        aria-checked={checked}
        onClick={() => !disabled && onChange(!checked)}
        className={[
          "relative inline-block w-8 h-[18px] rounded-full transition-colors flex-none",
          checked ? "bg-accent" : "bg-line",
        ].join(" ")}
      >
        <span
          className="absolute top-[2px] left-[2px] w-[14px] h-[14px] bg-white rounded-full shadow-sm transition-transform"
          style={{ transform: checked ? "translateX(14px)" : "translateX(0)" }}
        />
      </span>
      {label ? <span>{label}</span> : null}
    </label>
  );
}
