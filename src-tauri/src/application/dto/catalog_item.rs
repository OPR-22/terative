use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use super::common::MoneyDto;
use crate::application::catalog_item_usecases::UpdateCatalogItemInput;
use crate::domain::catalog_item::{
    CatalogItem, CatalogItemId, CatalogItemKind, NewCatalogItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum CatalogItemKindDto {
    Product,
    Service,
}

impl From<CatalogItemKind> for CatalogItemKindDto {
    fn from(k: CatalogItemKind) -> Self {
        match k {
            CatalogItemKind::Product => Self::Product,
            CatalogItemKind::Service => Self::Service,
        }
    }
}

impl From<CatalogItemKindDto> for CatalogItemKind {
    fn from(dto: CatalogItemKindDto) -> Self {
        match dto {
            CatalogItemKindDto::Product => CatalogItemKind::Product,
            CatalogItemKindDto::Service => CatalogItemKind::Service,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CatalogItemDto {
    pub id: Uuid,
    pub name: String,
    pub kind: CatalogItemKindDto,
    /// One entry per currency the item is priced in. May be empty.
    pub prices: Vec<MoneyDto>,
    pub unit: Option<String>,
    pub reference: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<&CatalogItem> for CatalogItemDto {
    fn from(s: &CatalogItem) -> Self {
        Self {
            id: s.id.0,
            name: s.name.clone(),
            kind: s.kind.into(),
            prices: s.prices.iter().map(MoneyDto::from).collect(),
            unit: s.unit.clone(),
            reference: s.reference.clone(),
            archived_at: s.archived_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewCatalogItemDto {
    pub name: String,
    pub kind: CatalogItemKindDto,
    pub prices: Vec<MoneyDto>,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl TryFrom<NewCatalogItemDto> for NewCatalogItem {
    type Error = DtoConvertError;
    fn try_from(dto: NewCatalogItemDto) -> Result<Self, Self::Error> {
        let prices = dto
            .prices
            .iter()
            .map(|m| m.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NewCatalogItem {
            name: dto.name,
            kind: dto.kind.into(),
            prices,
            unit: dto.unit,
            reference: dto.reference,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateCatalogItemDto {
    pub id: Uuid,
    pub name: String,
    pub kind: CatalogItemKindDto,
    pub prices: Vec<MoneyDto>,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl TryFrom<UpdateCatalogItemDto> for UpdateCatalogItemInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdateCatalogItemDto) -> Result<Self, Self::Error> {
        let prices = dto
            .prices
            .iter()
            .map(|m| m.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UpdateCatalogItemInput {
            id: CatalogItemId(dto.id),
            name: dto.name,
            kind: dto.kind.into(),
            prices,
            unit: dto.unit,
            reference: dto.reference,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::{Currency, Money};

    #[test]
    fn catalog_item_round_trip() {
        let eur = Currency::new("EUR").unwrap();
        let usd = Currency::new("USD").unwrap();
        let domain = CatalogItem {
            id: CatalogItemId::new(),
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur), Money::new(17000, usd)],
            unit: Some("hour".into()),
            reference: None,
            archived_at: None,
        };
        let dto: CatalogItemDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.prices.len(), 2);
        assert_eq!(dto.prices[0].amount, 15000);
        assert_eq!(dto.prices[0].currency.code, "EUR");
        assert_eq!(dto.prices[1].currency.code, "USD");
        assert_eq!(dto.unit.as_deref(), Some("hour"));
        assert!(matches!(dto.kind, CatalogItemKindDto::Service));
    }

    #[test]
    fn new_catalog_item_dto_maps_to_domain_input() {
        let dto = NewCatalogItemDto {
            name: "Coaching".into(),
            kind: CatalogItemKindDto::Service,
            prices: vec![MoneyDto::from(Money::from_minor(20000, Currency::Eur))],
            unit: Some("session".into()),
            reference: Some("COACH-1".into()),
        };
        let input: NewCatalogItem = dto.try_into().unwrap();
        assert_eq!(input.name, "Coaching");
        assert_eq!(input.prices.len(), 1);
        assert_eq!(input.prices[0].minor_units(), 20000);
        assert_eq!(input.unit.as_deref(), Some("session"));
        assert_eq!(input.reference.as_deref(), Some("COACH-1"));
    }

    #[test]
    fn update_catalog_item_dto_rejects_invalid_currency() {
        let dto = UpdateCatalogItemDto {
            id: Uuid::new_v4(),
            name: "X".into(),
            kind: CatalogItemKindDto::Product,
            prices: vec![MoneyDto {
                amount: 100,
                currency: crate::application::dto::CurrencyConfigDto {
                    code: "xx".into(),
                    name: String::new(),
                    symbol: String::new(),
                    symbol_before: false,
                    fraction_digits: 2,
                    main_unit_name: String::new(),
                    sub_unit_name: None,
                },
            }],
            unit: None,
            reference: None,
        };
        let err = UpdateCatalogItemInput::try_from(dto).unwrap_err();
        assert!(matches!(err, DtoConvertError::InvalidCurrency(_)));
    }

    #[test]
    fn kind_dto_round_trips() {
        for k in [CatalogItemKind::Product, CatalogItemKind::Service] {
            let dto: CatalogItemKindDto = k.into();
            let back: CatalogItemKind = dto.into();
            assert_eq!(back, k);
        }
    }
}
