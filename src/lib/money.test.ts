import { describe, it, expect } from "vitest";

import { Money } from "./money";
import type { CurrencyConfigDto } from "../ipc";

// Test fixtures. Each currency exercises a different shape:
//   EUR — 2 fraction digits, symbol after the amount (typical EU).
//   USD — 2 fraction digits, symbol before the amount (US convention).
//   JPY — 0 fraction digits, symbol before (no minor unit at all).
const EUR: CurrencyConfigDto = {
  code: "EUR",
  name: "Euro",
  symbol: "€",
  symbol_before: false,
  fraction_digits: 2,
  main_unit_name: "euro",
  sub_unit_name: "cent",
};

const USD: CurrencyConfigDto = {
  code: "USD",
  name: "US Dollar",
  symbol: "$",
  symbol_before: true,
  fraction_digits: 2,
  main_unit_name: "dollar",
  sub_unit_name: "cent",
};

const JPY: CurrencyConfigDto = {
  code: "JPY",
  name: "Japanese Yen",
  symbol: "¥",
  symbol_before: true,
  fraction_digits: 0,
  main_unit_name: "yen",
  sub_unit_name: null,
};

describe("Money — construction", () => {
  it("fromMinor accepts a bigint exactly", () => {
    expect(Money.fromMinor(1234n, EUR).minorUnits).toBe(1234n);
  });

  it("fromMinor accepts a number and rounds to the nearest integer", () => {
    expect(Money.fromMinor(1234.7, EUR).minorUnits).toBe(1235n);
    expect(Money.fromMinor(1234.4, EUR).minorUnits).toBe(1234n);
  });

  it("fromMajor scales by the currency's fraction_digits", () => {
    expect(Money.fromMajor("12.34", EUR).minorUnits).toBe(1234n);
    expect(Money.fromMajor("100", JPY).minorUnits).toBe(100n);
  });

  it("fromMajor uses banker's rounding at the half (0.5 → even)", () => {
    // 0.005 EUR × 100 scale = 0.5 minor units → rounds to 0 (0 is even).
    expect(Money.fromMajor("0.005", EUR).minorUnits).toBe(0n);
    // 0.015 EUR × 100 scale = 1.5 minor units → rounds to 2 (2 is even).
    expect(Money.fromMajor("0.015", EUR).minorUnits).toBe(2n);
  });

  it("zero is genuinely zero", () => {
    const z = Money.zero(EUR);
    expect(z.minorUnits).toBe(0n);
    expect(z.isZero()).toBe(true);
  });
});

describe("Money — DTO round-trip", () => {
  it("fromDto → toDto preserves amount and currency for safe-integer values", () => {
    const dto = { amount: 1234, currency: EUR };
    expect(Money.fromDto(dto).toDto()).toEqual(dto);
  });
});

describe("Money — arithmetic", () => {
  it("add and subtract within the same currency", () => {
    const a = Money.fromMinor(1000n, EUR);
    const b = Money.fromMinor(500n, EUR);
    expect(a.add(b).minorUnits).toBe(1500n);
    expect(a.subtract(b).minorUnits).toBe(500n);
  });

  it("operations return new instances (immutability)", () => {
    const a = Money.fromMinor(1000n, EUR);
    const b = a.add(Money.fromMinor(1n, EUR));
    expect(a.minorUnits).toBe(1000n);
    expect(b.minorUnits).toBe(1001n);
  });

  it("multiply rounds half to even", () => {
    // 100 × 0.5 = 50.0 — exact, no rounding.
    expect(Money.fromMinor(100n, EUR).multiply(0.5).minorUnits).toBe(50n);
    // 101 × 0.5 = 50.5 — half → 50 (even).
    expect(Money.fromMinor(101n, EUR).multiply(0.5).minorUnits).toBe(50n);
    // 103 × 0.5 = 51.5 — half → 52 (even).
    expect(Money.fromMinor(103n, EUR).multiply(0.5).minorUnits).toBe(52n);
  });

  it("negate flips the sign", () => {
    expect(Money.fromMinor(1000n, EUR).negate().minorUnits).toBe(-1000n);
    expect(Money.fromMinor(-50n, EUR).negate().minorUnits).toBe(50n);
  });

  it("add and subtract throw on currency mismatch", () => {
    const eur = Money.fromMinor(100n, EUR);
    const usd = Money.fromMinor(100n, USD);
    expect(() => eur.add(usd)).toThrow(/currency mismatch/);
    expect(() => eur.subtract(usd)).toThrow(/currency mismatch/);
  });
});

describe("Money — predicates", () => {
  it("isZero / isPositive / isNegative are mutually exclusive", () => {
    expect(Money.zero(EUR).isZero()).toBe(true);
    expect(Money.fromMinor(1n, EUR).isPositive()).toBe(true);
    expect(Money.fromMinor(-1n, EUR).isNegative()).toBe(true);
  });

  it("equals checks both currency code and amount", () => {
    const a = Money.fromMinor(100n, EUR);
    const b = Money.fromMinor(100n, EUR);
    const sameAmountDifferentCurrency = Money.fromMinor(100n, USD);
    expect(a.equals(b)).toBe(true);
    expect(a.equals(sameAmountDifferentCurrency)).toBe(false);
  });
});

describe("Money — formatting", () => {
  // en-US is locked to ASCII separators (",", "."), so we exact-match. fr-FR
  // and others use Unicode group separators (narrow no-break space) whose
  // exact code-point shifts across ICU versions — assert loosely there.
  it("formatWithCodePrefix in en-US uses commas + period", () => {
    const m = Money.fromMinor(123456789n, EUR); // 1,234,567.89
    expect(m.formatWithCodePrefix("en-US")).toBe("EUR 1,234,567.89");
  });

  it("formatWithSymbol in en-US prefixes when symbol_before=true (USD)", () => {
    const m = Money.fromMinor(100000n, USD);
    expect(m.formatWithSymbol("en-US")).toBe("$1,000.00");
  });

  it("formatWithSymbol in fr-FR suffixes when symbol_before=false (EUR)", () => {
    const m = Money.fromMinor(100000n, EUR);
    const out = m.formatWithSymbol("fr-FR");
    // Whatever group separator ICU emits, the amount + " €" suffix is stable.
    expect(out).toMatch(/^1.000,00 €$/);
  });

  it("zero-fraction currency formats without a decimal section (JPY)", () => {
    const m = Money.fromMinor(1234n, JPY);
    expect(m.formatAmount("en-US")).toBe("1,234");
    expect(m.formatWithSymbol("en-US")).toBe("¥1,234");
  });
});

describe("Money — safe-integer guard", () => {
  it("toMinorNumber returns a plain number when within 2^53", () => {
    expect(Money.fromMinor(1234n, EUR).toMinorNumber()).toBe(1234);
  });

  it("toMinorNumber throws when the bigint exceeds JS safe-integer range", () => {
    const huge = BigInt(Number.MAX_SAFE_INTEGER) + 1n;
    // Construct directly — fromMinor's number overload would itself lose
    // precision before getting here.
    expect(() => new Money(huge, EUR).toMinorNumber()).toThrow(/safe integer/);
  });
});
