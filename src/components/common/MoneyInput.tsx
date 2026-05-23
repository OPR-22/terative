import type { ChangeEvent } from "react";
import BigNumber from "bignumber.js";

import type { CurrencyConfigDto } from "../../ipc";
import { Field, Input } from "../ui/Input";

interface Props {
  label?: string;
  /** Value as a raw minor-unit integer in the component's currency. */
  valueMinor: number;
  onChangeMinor: (minor: number) => void;
  /**
   * Full currency metadata. Dictates the number of decimals shown, the `step`
   * on the underlying `<input type="number">`, and the symbol rendered after
   * the field. Required — callers should read it from the currency catalog.
   */
  currency: CurrencyConfigDto;
  disabled?: boolean;
}

export function MoneyInput({
  label,
  valueMinor,
  onChangeMinor,
  currency,
  disabled,
}: Props) {
  const scale = new BigNumber(10).pow(currency.fraction_digits);
  const displayValue = new BigNumber(valueMinor)
    .div(scale)
    .toFixed(currency.fraction_digits);

  const step =
    currency.fraction_digits === 0
      ? "1"
      : `0.${"0".repeat(currency.fraction_digits - 1)}1`;

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value.replace(",", ".");
    if (raw === "" || raw === "-") {
      onChangeMinor(0);
      return;
    }
    const parsed = new BigNumber(raw);
    if (!parsed.isFinite()) {
      onChangeMinor(0);
      return;
    }
    const minor = parsed
      .times(scale)
      .integerValue(BigNumber.ROUND_HALF_EVEN)
      .toNumber();
    onChangeMinor(minor);
  };

  const input = (
    <div className="flex items-center gap-2">
      <span className="text-[12px] text-ink-3 font-mono tabular shrink-0">
        {currency.code}
      </span>
      <Input
        mono
        type="number"
        step={step}
        min="0"
        value={displayValue}
        onChange={handleChange}
        disabled={disabled}
        className="text-right"
      />
    </div>
  );

  if (label) return <Field label={label}>{input}</Field>;
  return input;
}
