use serde::{Deserialize, Serialize};

use super::DtoConvertError;
use crate::domain::money::{Currency, Money};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct MoneyDto {
    pub amount_cents: i64,
    pub currency: String,
}

impl From<&Money> for MoneyDto {
    fn from(m: &Money) -> Self {
        Self {
            amount_cents: m.amount_cents,
            currency: m.currency.code().to_string(),
        }
    }
}

impl From<Money> for MoneyDto {
    fn from(m: Money) -> Self {
        (&m).into()
    }
}

impl TryFrom<&MoneyDto> for Money {
    type Error = DtoConvertError;
    fn try_from(dto: &MoneyDto) -> Result<Self, Self::Error> {
        let currency = Currency::new(&dto.currency)
            .map_err(|e| DtoConvertError::InvalidCurrency(e.to_string()))?;
        Ok(Money::new(dto.amount_cents, currency))
    }
}

impl TryFrom<MoneyDto> for Money {
    type Error = DtoConvertError;
    fn try_from(dto: MoneyDto) -> Result<Self, Self::Error> {
        Money::try_from(&dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    #[test]
    fn money_round_trip() {
        let domain = Money::new(12345, eur());
        let dto: MoneyDto = (&domain).into();
        assert_eq!(dto.amount_cents, 12345);
        assert_eq!(dto.currency, "EUR");
        let back: Money = (&dto).try_into().unwrap();
        assert_eq!(back, domain);
    }

    #[test]
    fn money_dto_rejects_invalid_currency() {
        let dto = MoneyDto {
            amount_cents: 100,
            currency: "eur".into(),
        };
        let err = Money::try_from(&dto).unwrap_err();
        assert!(matches!(err, DtoConvertError::InvalidCurrency(_)));
    }
}
