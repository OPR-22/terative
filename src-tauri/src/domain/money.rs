use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Money {
    pub amount_cents: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Currency([u8; 3]);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("currency code must be exactly 3 ASCII uppercase letters, got {0:?}")]
    InvalidCurrencyCode(String),
    #[error("currency mismatch: {left} vs {right}")]
    CurrencyMismatch { left: String, right: String },
    #[error("money arithmetic overflow")]
    Overflow,
}

impl Currency {
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(MoneyError::InvalidCurrencyCode(code.to_string()));
        }
        let b = code.as_bytes();
        Ok(Self([b[0], b[1], b[2]]))
    }

    pub fn code(&self) -> &str {
        std::str::from_utf8(&self.0).expect("currency bytes are ASCII by construction")
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl Money {
    pub fn new(amount_cents: i64, currency: Currency) -> Self {
        Self { amount_cents, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self::new(0, currency)
    }

    pub fn add(&self, other: Money) -> Result<Money, MoneyError> {
        self.ensure_same_currency(&other)?;
        let sum = self
            .amount_cents
            .checked_add(other.amount_cents)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(sum, self.currency))
    }

    pub fn sub(&self, other: Money) -> Result<Money, MoneyError> {
        self.ensure_same_currency(&other)?;
        let diff = self
            .amount_cents
            .checked_sub(other.amount_cents)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(diff, self.currency))
    }

    pub fn is_zero(&self) -> bool {
        self.amount_cents == 0
    }

    pub fn is_negative(&self) -> bool {
        self.amount_cents < 0
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

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    #[test]
    fn currency_accepts_three_uppercase_letters() {
        assert_eq!(Currency::new("EUR").unwrap().code(), "EUR");
        assert_eq!(Currency::new("USD").unwrap().code(), "USD");
    }

    #[test]
    fn currency_rejects_invalid_codes() {
        assert!(matches!(
            Currency::new("eur"),
            Err(MoneyError::InvalidCurrencyCode(_))
        ));
        assert!(matches!(
            Currency::new("EU"),
            Err(MoneyError::InvalidCurrencyCode(_))
        ));
        assert!(matches!(
            Currency::new("EURO"),
            Err(MoneyError::InvalidCurrencyCode(_))
        ));
        assert!(matches!(
            Currency::new("E1R"),
            Err(MoneyError::InvalidCurrencyCode(_))
        ));
    }

    #[test]
    fn money_zero_is_zero() {
        let m = Money::zero(eur());
        assert!(m.is_zero());
        assert_eq!(m.amount_cents, 0);
    }

    #[test]
    fn money_add_same_currency() {
        let a = Money::new(1000, eur());
        let b = Money::new(250, eur());
        assert_eq!(a.add(b).unwrap(), Money::new(1250, eur()));
    }

    #[test]
    fn money_sub_same_currency_can_go_negative() {
        let a = Money::new(500, eur());
        let b = Money::new(800, eur());
        let r = a.sub(b).unwrap();
        assert_eq!(r.amount_cents, -300);
        assert!(r.is_negative());
    }

    #[test]
    fn money_add_rejects_currency_mismatch() {
        let a = Money::new(1000, eur());
        let b = Money::new(500, usd());
        assert!(matches!(a.add(b), Err(MoneyError::CurrencyMismatch { .. })));
    }

    #[test]
    fn money_add_detects_overflow() {
        let a = Money::new(i64::MAX, eur());
        let b = Money::new(1, eur());
        assert_eq!(a.add(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn money_sub_detects_overflow() {
        let a = Money::new(i64::MIN, eur());
        let b = Money::new(1, eur());
        assert_eq!(a.sub(b), Err(MoneyError::Overflow));
    }
}
