//! `Money` is the app's canonical monetary value object.
//!
//! # Storage model
//!
//! A `Money` value is stored as `minor_units: i64` where "minor units" means
//! the smallest indivisible unit of its own currency:
//!
//! - For EUR, USD, GBP, etc. (`fraction_digits = 2`) → cents
//! - For JPY, KRW (`fraction_digits = 0`)            → whole yen / whole won
//!
//! The rule is universal: `display_value = minor_units / 10^fraction_digits`.
//! This means:
//! - `Money::from_minor(100, Eur)` → €1.00
//! - `Money::from_minor(100, Jpy)` → ¥100
//! - `Money::from_minor(12345, Eur)` → €123.45
//! - `Money::from_minor(12345, Jpy)` → ¥12,345
//!
//! # Arithmetic
//!
//! Addition and subtraction are exact i64 operations gated by a currency
//! check. Multiplication by a non-integer scalar (quantity × unit price, tax
//! × rate) is the only place rounding happens; it uses banker's rounding
//! (`RoundingStrategy::MidpointNearestEven`) which is the accounting standard.
//!
//! The public API has no `amount_cents` field — every consumer goes through
//! `.minor_units()`, which makes the "minor units in the currency's scale"
//! semantic explicit.

use std::fmt;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

/// The full list of currencies the app supports. This is an enum rather than
/// an open-ended string so every consumer (domain, repo, DTO, frontend) sees
/// the same finite set and the metadata can live here as static data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    Usd,
    Eur,
    Jpy,
    Gbp,
    Cny,
    Aud,
    Cad,
    Chf,
    Hkd,
    Sgd,
    Sek,
    Krw,
    Nok,
    Nzd,
    Inr,
    Mxn,
    Twd,
    Zar,
    Brl,
    Dkk,
}

/// Where the symbol sits relative to the numeric value when formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolPosition {
    Before,
    After,
}

/// Static metadata describing a currency. Held by value because everything is
/// `&'static str` / primitive and the table is tiny.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyMeta {
    pub code: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub symbol_position: SymbolPosition,
    pub fraction_digits: u8,
    pub main_unit_name: &'static str,
    pub sub_unit_name: Option<&'static str>,
}

impl Currency {
    /// Returns the full metadata for this currency.
    pub const fn meta(&self) -> CurrencyMeta {
        match self {
            Self::Usd => CurrencyMeta {
                code: "USD",
                name: "US Dollar",
                symbol: "$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Eur => CurrencyMeta {
                code: "EUR",
                name: "Euro",
                symbol: "€",
                symbol_position: SymbolPosition::After,
                fraction_digits: 2,
                main_unit_name: "euros",
                sub_unit_name: Some("cents"),
            },
            Self::Jpy => CurrencyMeta {
                code: "JPY",
                name: "Japanese Yen",
                symbol: "¥",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 0,
                main_unit_name: "yen",
                sub_unit_name: None,
            },
            Self::Gbp => CurrencyMeta {
                code: "GBP",
                name: "British Pound",
                symbol: "£",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "pounds",
                sub_unit_name: Some("pence"),
            },
            Self::Cny => CurrencyMeta {
                code: "CNY",
                name: "Chinese Renminbi",
                symbol: "¥",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "yuan",
                sub_unit_name: Some("fen"),
            },
            Self::Aud => CurrencyMeta {
                code: "AUD",
                name: "Australian Dollar",
                symbol: "A$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Cad => CurrencyMeta {
                code: "CAD",
                name: "Canadian Dollar",
                symbol: "C$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Chf => CurrencyMeta {
                code: "CHF",
                name: "Swiss Franc",
                symbol: "CHF",
                symbol_position: SymbolPosition::After,
                fraction_digits: 2,
                main_unit_name: "francs",
                sub_unit_name: Some("centimes"),
            },
            Self::Hkd => CurrencyMeta {
                code: "HKD",
                name: "Hong Kong Dollar",
                symbol: "HK$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Sgd => CurrencyMeta {
                code: "SGD",
                name: "Singapore Dollar",
                symbol: "S$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Sek => CurrencyMeta {
                code: "SEK",
                name: "Swedish Krona",
                symbol: "kr",
                symbol_position: SymbolPosition::After,
                fraction_digits: 2,
                main_unit_name: "kronor",
                sub_unit_name: Some("öre"),
            },
            Self::Krw => CurrencyMeta {
                code: "KRW",
                name: "South Korean Won",
                symbol: "₩",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 0,
                main_unit_name: "won",
                sub_unit_name: None,
            },
            Self::Nok => CurrencyMeta {
                code: "NOK",
                name: "Norwegian Krone",
                symbol: "kr",
                symbol_position: SymbolPosition::After,
                fraction_digits: 2,
                main_unit_name: "kroner",
                sub_unit_name: Some("øre"),
            },
            Self::Nzd => CurrencyMeta {
                code: "NZD",
                name: "New Zealand Dollar",
                symbol: "NZ$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Inr => CurrencyMeta {
                code: "INR",
                name: "Indian Rupee",
                symbol: "₹",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "rupees",
                sub_unit_name: Some("paise"),
            },
            Self::Mxn => CurrencyMeta {
                code: "MXN",
                name: "Mexican Peso",
                symbol: "$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "pesos",
                sub_unit_name: Some("centavos"),
            },
            Self::Twd => CurrencyMeta {
                code: "TWD",
                name: "New Taiwan Dollar",
                symbol: "NT$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "dollars",
                sub_unit_name: Some("cents"),
            },
            Self::Zar => CurrencyMeta {
                code: "ZAR",
                name: "South African Rand",
                symbol: "R",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "rand",
                sub_unit_name: Some("cents"),
            },
            Self::Brl => CurrencyMeta {
                code: "BRL",
                name: "Brazilian Real",
                symbol: "R$",
                symbol_position: SymbolPosition::Before,
                fraction_digits: 2,
                main_unit_name: "reais",
                sub_unit_name: Some("centavos"),
            },
            Self::Dkk => CurrencyMeta {
                code: "DKK",
                name: "Danish Krone",
                symbol: "kr",
                symbol_position: SymbolPosition::After,
                fraction_digits: 2,
                main_unit_name: "kroner",
                sub_unit_name: Some("øre"),
            },
        }
    }

    pub const fn code(&self) -> &'static str {
        self.meta().code
    }

    pub const fn symbol(&self) -> &'static str {
        self.meta().symbol
    }

    pub const fn fraction_digits(&self) -> u8 {
        self.meta().fraction_digits
    }

    pub const fn name(&self) -> &'static str {
        self.meta().name
    }

    /// 10^fraction_digits. Used internally for display math and conversions.
    pub const fn minor_unit_scale(&self) -> i64 {
        match self.meta().fraction_digits {
            0 => 1,
            1 => 10,
            2 => 100,
            3 => 1_000,
            4 => 10_000,
            _ => unreachable!(),
        }
    }

    /// Shim for the legacy `Currency::new(code)` API. New code should prefer
    /// matching on the enum directly (`Currency::Eur`) or calling `parse`.
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        Self::parse(code).ok_or_else(|| MoneyError::UnsupportedCurrency(code.to_string()))
    }

    /// Parses an ISO 4217 code (e.g. `"EUR"`) into the enum. Returns `None`
    /// for unsupported currencies.
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "USD" => Some(Self::Usd),
            "EUR" => Some(Self::Eur),
            "JPY" => Some(Self::Jpy),
            "GBP" => Some(Self::Gbp),
            "CNY" => Some(Self::Cny),
            "AUD" => Some(Self::Aud),
            "CAD" => Some(Self::Cad),
            "CHF" => Some(Self::Chf),
            "HKD" => Some(Self::Hkd),
            "SGD" => Some(Self::Sgd),
            "SEK" => Some(Self::Sek),
            "KRW" => Some(Self::Krw),
            "NOK" => Some(Self::Nok),
            "NZD" => Some(Self::Nzd),
            "INR" => Some(Self::Inr),
            "MXN" => Some(Self::Mxn),
            "TWD" => Some(Self::Twd),
            "ZAR" => Some(Self::Zar),
            "BRL" => Some(Self::Brl),
            "DKK" => Some(Self::Dkk),
            _ => None,
        }
    }

    /// Every supported currency, in the order they'll appear in UIs. Kept
    /// aligned with the BIS Triennial Survey's trading-volume ranking.
    pub const fn all() -> &'static [Currency] {
        &[
            Self::Usd,
            Self::Eur,
            Self::Jpy,
            Self::Gbp,
            Self::Cny,
            Self::Aud,
            Self::Cad,
            Self::Chf,
            Self::Hkd,
            Self::Sgd,
            Self::Sek,
            Self::Krw,
            Self::Nok,
            Self::Nzd,
            Self::Inr,
            Self::Mxn,
            Self::Twd,
            Self::Zar,
            Self::Brl,
            Self::Dkk,
        ]
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Money {
    minor_units: i64,
    currency: Currency,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("unsupported currency code: {0:?}")]
    UnsupportedCurrency(String),
    #[error("currency mismatch: {left} vs {right}")]
    CurrencyMismatch { left: String, right: String },
    #[error("money arithmetic overflow")]
    Overflow,
}

impl Money {
    /// Build directly from a minor-unit integer (cents for EUR, yen for JPY…).
    /// This is the canonical constructor — every call site should know what
    /// "minor units of this currency" means for the currency it passes.
    pub const fn from_minor(minor_units: i64, currency: Currency) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    /// Legacy alias for `from_minor`. The old API called its integer argument
    /// `amount_cents`; semantically it's now "minor units of the currency",
    /// which is the same thing for every 2-decimal currency. Kept to avoid
    /// churning every existing call site.
    pub const fn new(minor_units: i64, currency: Currency) -> Self {
        Self::from_minor(minor_units, currency)
    }

    /// Convenience for integer "whole currency" amounts — e.g. `from_major(12, Eur)`
    /// is €12.00. For zero-fraction currencies this is identical to `from_minor`.
    /// Returns `Overflow` if the multiplication overflows i64.
    pub fn from_major(major: i64, currency: Currency) -> Result<Self, MoneyError> {
        let scale = currency.minor_unit_scale();
        let minor = major.checked_mul(scale).ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor(minor, currency))
    }

    pub const fn zero(currency: Currency) -> Self {
        Self::from_minor(0, currency)
    }

    pub const fn minor_units(&self) -> i64 {
        self.minor_units
    }

    pub const fn currency(&self) -> Currency {
        self.currency
    }

    pub const fn is_zero(&self) -> bool {
        self.minor_units == 0
    }

    pub const fn is_negative(&self) -> bool {
        self.minor_units < 0
    }

    pub const fn is_positive(&self) -> bool {
        self.minor_units > 0
    }

    pub fn try_add(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(&other)?;
        let sum = self
            .minor_units
            .checked_add(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor(sum, self.currency))
    }

    pub fn try_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(&other)?;
        let diff = self
            .minor_units
            .checked_sub(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor(diff, self.currency))
    }

    /// Legacy aliases for `try_add` / `try_sub`. The old API used bare
    /// `add` / `sub` names with the same signature.
    pub fn add(self, other: Self) -> Result<Self, MoneyError> {
        self.try_add(other)
    }

    pub fn sub(self, other: Self) -> Result<Self, MoneyError> {
        self.try_sub(other)
    }

    pub fn negate(self) -> Self {
        Self::from_minor(-self.minor_units, self.currency)
    }

    /// Multiply by an arbitrary-precision scalar and round the result back to
    /// whole minor units using **banker's rounding** (`MidpointNearestEven`).
    ///
    /// This is the single rounding point in the domain. Line-item totals
    /// (`quantity × unit_price`), tax amounts (`subtotal × rate / 100`), and
    /// any other multiplicative operation go through here so the rounding
    /// strategy is consistent and auditable.
    pub fn multiply(self, multiplier: Decimal) -> Result<Self, MoneyError> {
        let intermediate = Decimal::from(self.minor_units) * multiplier;
        let rounded = intermediate
            .round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
        let minor = rounded.to_i64().ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor(minor, self.currency))
    }

    /// Format as a human-readable string using the currency's own symbol and
    /// fraction-digit count. Examples:
    ///
    /// - `Money::from_minor(12345, Eur).format()` → `"123.45 €"`
    /// - `Money::from_minor(12345, Jpy).format()` → `"¥12345"`
    /// - `Money::from_minor(-500, Usd).format()`  → `"$-5.00"`
    pub fn format(&self) -> String {
        let meta = self.currency.meta();
        let scale = self.currency.minor_unit_scale();
        let sign = if self.minor_units < 0 { "-" } else { "" };
        let abs = self.minor_units.unsigned_abs() as i64;
        let whole = abs / scale;
        let number = if meta.fraction_digits == 0 {
            format!("{sign}{whole}")
        } else {
            let frac = abs % scale;
            format!(
                "{sign}{whole}.{frac:0width$}",
                width = meta.fraction_digits as usize
            )
        };
        match meta.symbol_position {
            SymbolPosition::Before => format!("{}{}", meta.symbol, number),
            SymbolPosition::After => format!("{} {}", number, meta.symbol),
        }
    }

    fn ensure_same_currency(&self, other: &Money) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency.to_string(),
                right: other.currency.to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn parse_accepts_supported_codes() {
        assert_eq!(Currency::parse("EUR"), Some(Currency::Eur));
        assert_eq!(Currency::parse("USD"), Some(Currency::Usd));
        assert_eq!(Currency::parse("JPY"), Some(Currency::Jpy));
    }

    #[test]
    fn parse_rejects_unknown_codes() {
        assert_eq!(Currency::parse("XXX"), None);
        assert_eq!(Currency::parse("eur"), None);
        assert_eq!(Currency::parse("EU"), None);
        assert_eq!(Currency::parse("EURO"), None);
    }

    #[test]
    fn every_variant_has_unique_code_and_a_name() {
        let mut seen = std::collections::HashSet::new();
        for c in Currency::all() {
            assert!(seen.insert(c.code()), "duplicate code: {}", c.code());
            assert!(!c.name().is_empty());
            assert!(!c.symbol().is_empty());
            assert!(c.fraction_digits() <= 4);
        }
        assert_eq!(seen.len(), 20);
    }

    #[test]
    fn round_trip_every_variant_through_parse() {
        for &c in Currency::all() {
            assert_eq!(Currency::parse(c.code()), Some(c));
        }
    }

    #[test]
    fn jpy_and_krw_are_zero_decimal() {
        assert_eq!(Currency::Jpy.fraction_digits(), 0);
        assert_eq!(Currency::Krw.fraction_digits(), 0);
    }

    #[test]
    fn all_other_currencies_are_two_decimal() {
        for &c in Currency::all() {
            if matches!(c, Currency::Jpy | Currency::Krw) {
                continue;
            }
            assert_eq!(
                c.fraction_digits(),
                2,
                "{} should be 2-decimal",
                c.code()
            );
        }
    }

    #[test]
    fn minor_unit_scale_matches_fraction_digits() {
        assert_eq!(Currency::Eur.minor_unit_scale(), 100);
        assert_eq!(Currency::Jpy.minor_unit_scale(), 1);
    }

    // --- Money construction ---

    #[test]
    fn from_minor_eur() {
        let m = Money::from_minor(1234, Currency::Eur);
        assert_eq!(m.minor_units(), 1234);
        assert_eq!(m.currency(), Currency::Eur);
    }

    #[test]
    fn from_major_eur_scales_by_100() {
        let m = Money::from_major(12, Currency::Eur).unwrap();
        assert_eq!(m.minor_units(), 1200);
    }

    #[test]
    fn from_major_jpy_is_identity() {
        let m = Money::from_major(100, Currency::Jpy).unwrap();
        assert_eq!(m.minor_units(), 100);
    }

    #[test]
    fn from_major_detects_overflow() {
        let err = Money::from_major(i64::MAX, Currency::Eur).unwrap_err();
        assert_eq!(err, MoneyError::Overflow);
    }

    // --- Predicates ---

    #[test]
    fn zero_is_zero() {
        let m = Money::zero(Currency::Eur);
        assert!(m.is_zero());
        assert!(!m.is_negative());
        assert!(!m.is_positive());
    }

    #[test]
    fn sign_predicates() {
        assert!(Money::from_minor(-1, Currency::Eur).is_negative());
        assert!(Money::from_minor(1, Currency::Eur).is_positive());
    }

    // --- Addition / subtraction ---

    #[test]
    fn add_same_currency_exact() {
        let a = Money::from_minor(1000, Currency::Eur);
        let b = Money::from_minor(250, Currency::Eur);
        assert_eq!(
            a.try_add(b).unwrap(),
            Money::from_minor(1250, Currency::Eur)
        );
    }

    #[test]
    fn sub_can_go_negative() {
        let a = Money::from_minor(500, Currency::Eur);
        let b = Money::from_minor(800, Currency::Eur);
        let r = a.try_sub(b).unwrap();
        assert_eq!(r.minor_units(), -300);
        assert!(r.is_negative());
    }

    #[test]
    fn add_rejects_currency_mismatch() {
        let a = Money::from_minor(1000, Currency::Eur);
        let b = Money::from_minor(500, Currency::Usd);
        assert!(matches!(
            a.try_add(b),
            Err(MoneyError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn add_detects_overflow() {
        let a = Money::from_minor(i64::MAX, Currency::Eur);
        let b = Money::from_minor(1, Currency::Eur);
        assert_eq!(a.try_add(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn sub_detects_overflow() {
        let a = Money::from_minor(i64::MIN, Currency::Eur);
        let b = Money::from_minor(1, Currency::Eur);
        assert_eq!(a.try_sub(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn negate_flips_sign() {
        let m = Money::from_minor(500, Currency::Eur);
        assert_eq!(m.negate(), Money::from_minor(-500, Currency::Eur));
    }

    // --- Multiplication & rounding ---

    #[test]
    fn multiply_exact_result_no_rounding() {
        let m = Money::from_minor(1000, Currency::Eur); // €10.00
        let result = m.multiply(dec!(2)).unwrap();
        assert_eq!(result, Money::from_minor(2000, Currency::Eur));
    }

    #[test]
    fn multiply_fractional_quantity() {
        let m = Money::from_minor(1000, Currency::Eur); // €10.00 per unit
        let result = m.multiply(dec!(3.5)).unwrap();
        assert_eq!(result, Money::from_minor(3500, Currency::Eur));
    }

    #[test]
    fn multiply_banker_rounds_half_to_even_up() {
        // 125 × 0.5 = 62.5 → 62 (nearest even)
        let m = Money::from_minor(125, Currency::Eur);
        let result = m.multiply(dec!(0.5)).unwrap();
        assert_eq!(result.minor_units(), 62);
    }

    #[test]
    fn multiply_banker_rounds_half_to_even_down() {
        // 255 × 0.5 = 127.5 → 128 (nearest even)
        let m = Money::from_minor(255, Currency::Eur);
        let result = m.multiply(dec!(0.5)).unwrap();
        assert_eq!(result.minor_units(), 128);
    }

    #[test]
    fn multiply_tax_computation() {
        // 21% of €123.45 = €25.9245 → €25.92 (round half even)
        let subtotal = Money::from_minor(12345, Currency::Eur);
        let rate = dec!(21) / dec!(100);
        let tax = subtotal.multiply(rate).unwrap();
        assert_eq!(tax, Money::from_minor(2592, Currency::Eur));
    }

    #[test]
    fn multiply_preserves_currency() {
        let m = Money::from_minor(100, Currency::Jpy);
        let result = m.multiply(dec!(1.5)).unwrap();
        assert_eq!(result.currency(), Currency::Jpy);
    }

    // --- Formatting ---

    #[test]
    fn format_eur_has_two_fraction_digits_and_symbol_after() {
        assert_eq!(Money::from_minor(12345, Currency::Eur).format(), "123.45 €");
        assert_eq!(Money::from_minor(0, Currency::Eur).format(), "0.00 €");
        assert_eq!(Money::from_minor(5, Currency::Eur).format(), "0.05 €");
    }

    #[test]
    fn format_usd_has_symbol_before() {
        assert_eq!(Money::from_minor(12345, Currency::Usd).format(), "$123.45");
    }

    #[test]
    fn format_jpy_has_no_fraction_digits() {
        assert_eq!(Money::from_minor(12345, Currency::Jpy).format(), "¥12345");
        assert_eq!(Money::from_minor(0, Currency::Jpy).format(), "¥0");
    }

    #[test]
    fn format_krw_has_no_fraction_digits() {
        assert_eq!(Money::from_minor(1000, Currency::Krw).format(), "₩1000");
    }

    #[test]
    fn format_negative_eur() {
        assert_eq!(
            Money::from_minor(-12345, Currency::Eur).format(),
            "-123.45 €"
        );
    }
}
