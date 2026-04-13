import type { ChangeEvent } from "react";
import { Input } from "./Input";

interface Props {
  label?: string;
  valueCents: number;
  onChangeCents: (cents: number) => void;
  currencySymbol?: string;
  disabled?: boolean;
}

export function MoneyInput({
  label,
  valueCents,
  onChangeCents,
  currencySymbol = "€",
  disabled,
}: Props) {
  const displayValue = (valueCents / 100).toFixed(2);
  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value.replace(",", ".");
    const parsed = parseFloat(raw);
    if (Number.isNaN(parsed)) {
      onChangeCents(0);
      return;
    }
    onChangeCents(Math.round(parsed * 100));
  };
  return (
    <div className="flex items-end gap-2">
      <Input
        label={label}
        type="number"
        step="0.01"
        min="0"
        value={displayValue}
        onChange={handleChange}
        disabled={disabled}
      />
      <span className="pb-2 text-sm text-zinc-600">{currencySymbol}</span>
    </div>
  );
}
