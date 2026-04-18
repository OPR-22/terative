import { useEffect } from "react";

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
 * - `currencyCode` is the ISO 4217 code of the currency.
 *
 * The class is immutable; every operation returns a fresh instance.
 *
 * Display metadata (symbol, fraction digits, unit names) lives in the
 * `CurrencyCatalog` fetched from the backend at boot — not on `Money` itself.
 * This keeps `Money` small and keeps the metadata table as a single source
 * of truth on the backend.
 */
export class Money {
  readonly minorUnits: bigint;
  readonly currencyCode: string;

  constructor(minorUnits: bigint, currencyCode: string) {
    this.minorUnits = minorUnits;
    this.currencyCode = currencyCode;
  }

  // --- Constructors ---

  /**
   * Build from an integer minor-unit count. Accepts `number` for convenience
   * but the value must be a safe integer.
   */
  static fromMinor(minor: bigint | number, currencyCode: string): Money {
    const value =
      typeof minor === "bigint" ? minor : BigInt(Math.round(minor));
    return new Money(value, currencyCode);
  }

  /**
   * Build from a "major" (whole currency unit) value — e.g. 12.34 for €12.34,
   * or 100 for ¥100. The `fractionDigits` comes from the currency catalog;
   * passing the wrong value will silently mis-scale.
   *
   * Uses banker's rounding so a borderline value like 0.005 rounds the same
   * way on frontend and backend.
   */
  static fromMajor(
    major: string | number | BigNumber,
    currencyCode: string,
    fractionDigits: number,
  ): Money {
    const scale = new BigNumber(10).pow(fractionDigits);
    const minor = new BigNumber(major).times(scale).integerValue();
    return new Money(BigInt(minor.toFixed(0)), currencyCode);
  }

  static zero(currencyCode: string): Money {
    return new Money(0n, currencyCode);
  }

  static fromDto(dto: MoneyDto): Money {
    return new Money(BigInt(dto.amount_minor), dto.currency);
  }

  // --- Conversions ---

  toDto(): MoneyDto {
    return {
      amount_minor: Number(this.minorUnits),
      currency: this.currencyCode,
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
  toMajorBigNumber(fractionDigits: number): BigNumber {
    const scale = new BigNumber(10).pow(fractionDigits);
    return new BigNumber(this.minorUnits.toString()).div(scale);
  }

  // --- Arithmetic ---

  add(other: Money): Money {
    this.ensureSameCurrency(other);
    return new Money(this.minorUnits + other.minorUnits, this.currencyCode);
  }

  subtract(other: Money): Money {
    this.ensureSameCurrency(other);
    return new Money(this.minorUnits - other.minorUnits, this.currencyCode);
  }

  negate(): Money {
    return new Money(-this.minorUnits, this.currencyCode);
  }

  /**
   * Multiply by a scalar (quantity × price, tax rate, etc.) with banker's
   * rounding back to whole minor units. The scalar can be anything BigNumber
   * understands.
   */
  multiply(multiplier: string | number | BigNumber): Money {
    const result = new BigNumber(this.minorUnits.toString()).times(multiplier);
    const rounded = result.integerValue(BigNumber.ROUND_HALF_EVEN);
    return new Money(BigInt(rounded.toFixed(0)), this.currencyCode);
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
      this.currencyCode === other.currencyCode &&
      this.minorUnits === other.minorUnits
    );
  }

  // --- Display ---

  /**
   * Format using the currency's own symbol + position + fraction digits.
   * Examples:
   * - EUR 12345 → `"123.45 €"`
   * - USD 12345 → `"$123.45"`
   * - JPY 100   → `"¥100"`
   * - JPY -500  → `"¥-500"`
   */
  format(meta: CurrencyConfigDto): string {
    const scale = new BigNumber(10).pow(meta.fraction_digits);
    const major = new BigNumber(this.minorUnits.toString()).div(scale);
    const number = major.toFixed(meta.fraction_digits);
    return meta.symbol_before
      ? `${meta.symbol}${number}`
      : `${number} ${meta.symbol}`;
  }

  private ensureSameCurrency(other: Money): void {
    if (this.currencyCode !== other.currencyCode) {
      throw new Error(
        `currency mismatch: ${this.currencyCode} vs ${other.currencyCode}`,
      );
    }
  }
}

/**
 * React hook returning a formatter bound to the currency catalog. Call sites
 * just pass a `MoneyDto` (or a raw minor-unit integer + code) and get back
 * the formatted string, using the right symbol, symbol-position, and
 * fraction-digit count for that currency.
 *
 * The hook triggers catalog load if it hasn't happened yet, so any page that
 * displays money automatically warms the cache.
 */
export function useMoneyFormat(): {
  /** Format a DTO. Falls back to `"{value}"` if the catalog isn't loaded yet. */
  format: (dto: MoneyDto) => string;
  /** Format a raw minor-unit count in a given currency. */
  formatMinor: (minor: number | bigint, currencyCode: string) => string;
  /** Look up metadata for a currency code. Returns `undefined` if unloaded or unknown. */
  meta: (code: string) => CurrencyConfigDto | undefined;
} {
  const { all, load, byCode } = useCurrencyCatalogStore();

  useEffect(() => {
    if (all.length === 0) void load();
  }, [all.length, load]);

  const format = (dto: MoneyDto): string => {
    const meta = byCode(dto.currency);
    if (!meta) {
      // Fallback before the catalog is loaded: render as a bare decimal.
      return `${(dto.amount_minor / 100).toFixed(2)} ${dto.currency}`;
    }
    return new Money(BigInt(dto.amount_minor), dto.currency).format(meta);
  };

  const formatMinor = (minor: number | bigint, currencyCode: string): string => {
    const meta = byCode(currencyCode);
    if (!meta) {
      const n = typeof minor === "bigint" ? Number(minor) : minor;
      return `${(n / 100).toFixed(2)} ${currencyCode}`;
    }
    return new Money(
      typeof minor === "bigint" ? minor : BigInt(Math.round(minor)),
      currencyCode,
    ).format(meta);
  };

  return { format, formatMinor, meta: byCode };
}
