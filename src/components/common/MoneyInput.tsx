import type { ChangeEvent } from "react";

import BigNumber from "bignumber.js";

import type { CurrencyConfigDto } from "../../ipc";
import { Input } from "./Input";

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

/**
 * Currency-aware money input. For zero-decimal currencies (JPY, KRW) the
 * field accepts whole numbers only; for others, it accepts decimals with the
 * right precision.
 */
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

  return (
    <div className="flex items-end gap-2">
      <Input
        label={label}
        type="number"
        step={step}
        min="0"
        value={displayValue}
        onChange={handleChange}
        disabled={disabled}
      />
      <span className="pb-2 text-sm text-fg-muted">{currency.symbol}</span>
    </div>
  );
}
