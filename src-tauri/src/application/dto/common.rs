use serde::{Deserialize, Serialize};

use super::DtoConvertError;
use crate::domain::money::{Currency, Money};

/// Wire format for a monetary amount. `amount_minor` is the integer count of
/// the currency's smallest unit (cents for EUR, yen for JPY, etc.) — see the
/// doc on [`crate::domain::money::Money`] for the full contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct MoneyDto {
    pub amount_minor: i64,
    pub currency: String,
}

impl From<&Money> for MoneyDto {
    fn from(m: &Money) -> Self {
        Self {
            amount_minor: m.minor_units(),
            currency: m.currency().code().to_string(),
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
        let currency = Currency::parse(&dto.currency)
            .ok_or_else(|| DtoConvertError::InvalidCurrency(dto.currency.clone()))?;
        Ok(Money::from_minor(dto.amount_minor, currency))
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

    #[test]
    fn money_round_trip_eur() {
        let domain = Money::from_minor(12345, Currency::Eur);
        let dto: MoneyDto = (&domain).into();
        assert_eq!(dto.amount_minor, 12345);
        assert_eq!(dto.currency, "EUR");
        let back: Money = (&dto).try_into().unwrap();
        assert_eq!(back, domain);
    }

    #[test]
    fn money_round_trip_jpy() {
        let domain = Money::from_minor(100, Currency::Jpy);
        let dto: MoneyDto = (&domain).into();
        assert_eq!(dto.amount_minor, 100);
        assert_eq!(dto.currency, "JPY");
        let back: Money = (&dto).try_into().unwrap();
        assert_eq!(back, domain);
    }

    #[test]
    fn money_dto_rejects_invalid_currency() {
        let dto = MoneyDto {
            amount_minor: 100,
            currency: "xxx".into(),
        };
        let err = Money::try_from(&dto).unwrap_err();
        assert!(matches!(err, DtoConvertError::InvalidCurrency(_)));
    }
}
