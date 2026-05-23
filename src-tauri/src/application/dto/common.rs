use serde::{Deserialize, Serialize};

use super::{CurrencyConfigDto, DtoConvertError};
use crate::domain::money::{Currency, Money};

/// Wire format for a monetary amount.
///
/// `amount` is the integer count of the currency's smallest unit (cents for
/// EUR, yen for JPY, etc.) — see the doc on [`crate::domain::money::Money`]
/// for the contract. The full [`CurrencyConfigDto`] travels with every value
/// so the frontend never has to re-lookup display metadata to render it.
///
/// On the **input** side (forms / write commands) the frontend may build a
/// `MoneyDto` with a partially-populated `currency` — only `currency.code`
/// is read by [`TryFrom<&MoneyDto>`], everything else is derived server-side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct MoneyDto {
    pub amount: i64,
    pub currency: CurrencyConfigDto,
}

impl From<&Money> for MoneyDto {
    fn from(m: &Money) -> Self {
        Self {
            amount: m.minor_units(),
            currency: m.currency().into(),
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
        let currency = Currency::parse(&dto.currency.code)
            .ok_or_else(|| DtoConvertError::InvalidCurrency(dto.currency.code.clone()))?;
        Ok(Money::from_minor(dto.amount, currency))
    }
}

impl TryFrom<MoneyDto> for Money {
    type Error = DtoConvertError;
    fn try_from(dto: MoneyDto) -> Result<Self, Self::Error> {
        Money::try_from(&dto)
    }
}

// ---- PageDto ----

/// Serializable mirror of [`crate::application::ports::Page`] for the IPC boundary.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PageDto<T: specta::Type> {
    pub first: u32,
    pub last: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub total: u64,
    pub data: Vec<T>,
}

impl<T: specta::Type> From<crate::application::ports::Page<T>> for PageDto<T> {
    fn from(page: crate::application::ports::Page<T>) -> Self {
        Self {
            first: page.first,
            last: page.last,
            previous: page.previous,
            next: page.next,
            total: page.total,
            data: page.data,
        }
    }
}

/// Pagination input parameters sent from the frontend.
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct PaginationParamsDto {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

impl From<PaginationParamsDto> for crate::application::ports::PaginationParams {
    fn from(dto: PaginationParamsDto) -> Self {
        let defaults = Self::default();
        Self {
            page: dto.page.unwrap_or(defaults.page).max(1),
            per_page: dto.per_page.unwrap_or(defaults.per_page).clamp(1, 200),
        }
    }
}

impl From<Option<PaginationParamsDto>> for crate::application::ports::PaginationParams {
    fn from(opt: Option<PaginationParamsDto>) -> Self {
        opt.unwrap_or_default().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_round_trip_eur() {
        let domain = Money::from_minor(12345, Currency::Eur);
        let dto: MoneyDto = (&domain).into();
        assert_eq!(dto.amount, 12345);
        assert_eq!(dto.currency.code, "EUR");
        assert_eq!(dto.currency.symbol, "€");
        let back: Money = (&dto).try_into().unwrap();
        assert_eq!(back, domain);
    }

    #[test]
    fn money_round_trip_jpy() {
        let domain = Money::from_minor(100, Currency::Jpy);
        let dto: MoneyDto = (&domain).into();
        assert_eq!(dto.amount, 100);
        assert_eq!(dto.currency.code, "JPY");
        assert_eq!(dto.currency.fraction_digits, 0);
        let back: Money = (&dto).try_into().unwrap();
        assert_eq!(back, domain);
    }

    #[test]
    fn money_dto_rejects_invalid_currency() {
        let dto = MoneyDto {
            amount: 100,
            currency: CurrencyConfigDto {
                code: "xxx".into(),
                name: String::new(),
                symbol: String::new(),
                symbol_before: false,
                fraction_digits: 2,
                main_unit_name: String::new(),
                sub_unit_name: None,
            },
        };
        let err = Money::try_from(&dto).unwrap_err();
        assert!(matches!(err, DtoConvertError::InvalidCurrency(_)));
    }
}
