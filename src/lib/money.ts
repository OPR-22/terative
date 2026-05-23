import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import BigNumber from "bignumber.js";

import type { CurrencyConfigDto, MoneyDto } from "../ipc";
import { useCurrencyCatalogStore } from "../stores/currencyCatalogStore";

// Global: banker's rounding everywhere. Matches the backend's
// `RoundingStrategy::MidpointNearestEven` so frontend previews and server
// results agree on any borderline value.
BigNumber.set({ ROUNDING_MODE: BigNumber.ROUND_HALF_EVEN });

/**
 * `Money` is the frontend mirror of the Rust `Money` value object.
 *
 * - `minorUnits` is a `bigint` holding the integer count of the currency's
 *   smallest unit (cents for EUR, yen for JPY, etc.). `bigint` gives us i64
 *   parity and sidesteps the JS number precision ceiling (2^53).
 * - `currency` is the full [`CurrencyConfigDto`] — code, symbol, fraction
 *   digits, etc. — which travels with every `MoneyDto` on the wire, so a
 *   `Money` instance carries everything it needs to render itself.
 *
 * The class is immutable; every operation returns a fresh instance.
 */
export class Money {
  readonly minorUnits: bigint;
  readonly currency: CurrencyConfigDto;

  constructor(minorUnits: bigint, currency: CurrencyConfigDto) {
    this.minorUnits = minorUnits;
    this.currency = currency;
  }

  // --- Constructors ---

  /**
   * Build from an integer minor-unit count. Accepts `number` for convenience
   * but the value must be a safe integer.
   */
  static fromMinor(
    minor: bigint | number,
    currency: CurrencyConfigDto,
  ): Money {
    const value =
      typeof minor === "bigint" ? minor : BigInt(Math.round(minor));
    return new Money(value, currency);
  }

  /**
   * Build from a "major" (whole currency unit) value — e.g. 12.34 for €12.34,
   * or 100 for ¥100. Uses banker's rounding so a borderline value like 0.005
   * rounds the same way on frontend and backend.
   */
  static fromMajor(
    major: string | number | BigNumber,
    currency: CurrencyConfigDto,
  ): Money {
    const scale = new BigNumber(10).pow(currency.fraction_digits);
    const minor = new BigNumber(major).times(scale).integerValue();
    return new Money(BigInt(minor.toFixed(0)), currency);
  }

  static zero(currency: CurrencyConfigDto): Money {
    return new Money(0n, currency);
  }

  static fromDto(dto: MoneyDto): Money {
    return new Money(BigInt(dto.amount), dto.currency);
  }

  // --- Conversions ---

  toDto(): MoneyDto {
    return {
      amount: Number(this.minorUnits),
      currency: this.currency,
    };
  }

  /** Minor units as a `number`. Throws if the value exceeds 2^53. */
  toMinorNumber(): number {
    const MAX = BigInt(Number.MAX_SAFE_INTEGER);
    const MIN = -MAX;
    if (this.minorUnits > MAX || this.minorUnits < MIN) {
      throw new RangeError(
        `Money value ${this.minorUnits} exceeds JS safe integer range`,
      );
    }
    return Number(this.minorUnits);
  }

  /** Major-unit BigNumber, scaled by the currency's fraction digits. */
  toMajorBigNumber(): BigNumber {
    const scale = new BigNumber(10).pow(this.currency.fraction_digits);
    return new BigNumber(this.minorUnits.toString()).div(scale);
  }

  // --- Arithmetic ---

  add(other: Money): Money {
    this.ensureSameCurrency(other);
    return new Money(this.minorUnits + other.minorUnits, this.currency);
  }

  subtract(other: Money): Money {
    this.ensureSameCurrency(other);
    return new Money(this.minorUnits - other.minorUnits, this.currency);
  }

  negate(): Money {
    return new Money(-this.minorUnits, this.currency);
  }

  /**
   * Multiply by a scalar (quantity × price, tax rate, etc.) with banker's
   * rounding back to whole minor units. The scalar can be anything BigNumber
   * understands.
   */
  multiply(multiplier: string | number | BigNumber): Money {
    const result = new BigNumber(this.minorUnits.toString()).times(multiplier);
    const rounded = result.integerValue(BigNumber.ROUND_HALF_EVEN);
    return new Money(BigInt(rounded.toFixed(0)), this.currency);
  }

  // --- Predicates ---

  isZero(): boolean {
    return this.minorUnits === 0n;
  }

  isNegative(): boolean {
    return this.minorUnits < 0n;
  }

  isPositive(): boolean {
    return this.minorUnits > 0n;
  }

  equals(other: Money): boolean {
    return (
      this.currency.code === other.currency.code &&
      this.minorUnits === other.minorUnits
    );
  }

  // --- Display ---

  /** ISO 4217 code, e.g. `"EUR"`. */
  get code(): string {
    return this.currency.code;
  }

  /** Currency symbol, e.g. `"€"`. */
  get symbol(): string {
    return this.currency.symbol;
  }

  /**
   * Locale-formatted major-unit amount with grouping, *without* any
   * currency marker. Example (fr): `123456789` minor units in EUR →
   * `"1 234 567,89"`.
   */
  formatAmount(locale: string): string {
    const major = this.toMajorBigNumber().toNumber();
    return new Intl.NumberFormat(locale, {
      minimumFractionDigits: this.currency.fraction_digits,
      maximumFractionDigits: this.currency.fraction_digits,
    }).format(major);
  }

  /**
   * Amount + currency symbol respecting the currency's `symbol_before`
   * preference. Example (fr): `"17 950,54 €"` for EUR, `"$4 280,00"` for USD.
   */
  formatWithSymbol(locale: string): string {
    const amount = this.formatAmount(locale);
    return this.currency.symbol_before
      ? `${this.currency.symbol}${amount}`
      : `${amount} ${this.currency.symbol}`;
  }

  /**
   * Amount + ISO code (unambiguous in accounting contexts). Example (fr):
   * `"17 950,54 EUR"`.
   */
  formatWithCode(locale: string): string {
    return `${this.formatAmount(locale)} ${this.currency.code}`;
  }

  /**
   * ISO code + amount. Example (fr): `"EUR 17 950,54"`. Prefer this in
   * multi-currency tables: the code anchors at a fixed column, so rows with
   * different digit counts stay easy to scan.
   */
  formatWithCodePrefix(locale: string): string {
    return `${this.currency.code} ${this.formatAmount(locale)}`;
  }

  private ensureSameCurrency(other: Money): void {
    if (this.currency.code !== other.currency.code) {
      throw new Error(
        `currency mismatch: ${this.currency.code} vs ${other.currency.code}`,
      );
    }
  }
}

/**
 * Ergonomic hook returning locale-bound formatters for components that
 * receive `MoneyDto`s straight from the IPC layer. Since every `MoneyDto`
 * now carries its own currency metadata, no catalog lookup is needed.
 */
export function useMoneyFormat(): {
  /**
   * Default money display: `"EUR 17 950,54"` — ISO code prefix + locale
   * formatted amount. Used everywhere outside table cells. Tables should
   * split into a code column and a right-aligned amount column instead.
   */
  format: (dto: MoneyDto) => string;
  /** `"EUR 17 950,54"` — alias for `format`, kept for clarity at call sites. */
  formatWithCodePrefix: (dto: MoneyDto) => string;
  /** `"17 950,54 €"` — symbol form. Use only when the symbol is the goal (settings sample, etc.). */
  formatWithSymbol: (dto: MoneyDto) => string;
  /** Just the locale-formatted amount, no currency marker — for in-table amount cells. */
  formatAmount: (dto: MoneyDto) => string;
  /**
   * Format a raw minor-unit count + ISO code as `"EUR 17 950,54"`. Falls back
   * to a bare decimal if the catalog hasn't loaded yet or the code is unknown.
   */
  formatMinor: (minor: number | bigint, code: string) => string;
} {
  const { all, load, byCode } = useCurrencyCatalogStore();
  const { i18n } = useTranslation();
  const locale = i18n.language;

  useEffect(() => {
    if (all.length === 0) void load();
  }, [all.length, load]);

  const formatPrefix = (dto: MoneyDto) =>
    Money.fromDto(dto).formatWithCodePrefix(locale);

  return {
    format: formatPrefix,
    formatWithCodePrefix: formatPrefix,
    formatWithSymbol: (dto) => Money.fromDto(dto).formatWithSymbol(locale),
    formatAmount: (dto) => Money.fromDto(dto).formatAmount(locale),
    formatMinor: (minor, code) => {
      const currency = byCode(code);
      if (!currency) {
        const n = typeof minor === "bigint" ? Number(minor) : minor;
        return `${code} ${(n / 100).toFixed(2)}`;
      }
      return Money.fromMinor(minor, currency).formatWithCodePrefix(locale);
    },
  };
}

/**
 * Hook for input forms that have an ISO code and minor-unit count but need
 * to send a full `MoneyDto` to the backend. Resolves the embedded currency
 * metadata from the catalog store. Returns `null` when the catalog hasn't
 * loaded yet or the code is unknown — callers should guard accordingly.
 */
export function useMakeMoneyDto(): (
  minor: number | bigint,
  code: string,
) => MoneyDto | null {
  const { all, load, byCode } = useCurrencyCatalogStore();

  useEffect(() => {
    if (all.length === 0) void load();
  }, [all.length, load]);

  return (minor, code) => {
    const currency = byCode(code);
    if (!currency) return null;
    return {
      amount: typeof minor === "bigint" ? Number(minor) : Math.round(minor),
      currency,
    };
  };
}

/**
 * Synchronous helper that throws if the catalog isn't loaded. Use only when
 * the surrounding code has already confirmed the catalog is ready (e.g.
 * inside a Settings page that gated on `useCurrencyCatalogStore.all.length`).
 */
export function makeMoneyDto(
  minor: number | bigint,
  currency: CurrencyConfigDto,
): MoneyDto {
  return {
    amount: typeof minor === "bigint" ? Number(minor) : Math.round(minor),
    currency,
  };
}
