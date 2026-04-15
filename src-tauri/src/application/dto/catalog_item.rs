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
            CatalogItemKindDto::Product => Self::Product,
            CatalogItemKindDto::Service => Self::Service,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CatalogItemDto {
    pub id: Uuid,
    pub name: String,
    pub kind: CatalogItemKindDto,
    pub default_price: MoneyDto,
    pub unit: Option<String>,
    pub reference: Option<String>,
    pub active: bool,
}

impl From<&CatalogItem> for CatalogItemDto {
    fn from(s: &CatalogItem) -> Self {
        Self {
            id: s.id.0,
            name: s.name.clone(),
            kind: s.kind.into(),
            default_price: (&s.default_price).into(),
            unit: s.unit.clone(),
            reference: s.reference.clone(),
            active: s.active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewCatalogItemDto {
    pub name: String,
    pub kind: CatalogItemKindDto,
    pub default_price: MoneyDto,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl TryFrom<NewCatalogItemDto> for NewCatalogItem {
    type Error = DtoConvertError;
    fn try_from(dto: NewCatalogItemDto) -> Result<Self, Self::Error> {
        Ok(NewCatalogItem {
            name: dto.name,
            kind: dto.kind.into(),
            default_price: (&dto.default_price).try_into()?,
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
    pub default_price: MoneyDto,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl TryFrom<UpdateCatalogItemDto> for UpdateCatalogItemInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdateCatalogItemDto) -> Result<Self, Self::Error> {
        Ok(UpdateCatalogItemInput {
            id: CatalogItemId(dto.id),
            name: dto.name,
            kind: dto.kind.into(),
            default_price: (&dto.default_price).try_into()?,
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
        let domain = CatalogItem {
            id: CatalogItemId::new(),
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            default_price: Money::new(15000, eur),
            unit: Some("hour".into()),
            reference: None,
            active: true,
        };
        let dto: CatalogItemDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.default_price.amount_cents, 15000);
        assert_eq!(dto.default_price.currency, "EUR");
        assert_eq!(dto.unit.as_deref(), Some("hour"));
        assert!(matches!(dto.kind, CatalogItemKindDto::Service));
    }

    #[test]
    fn new_catalog_item_dto_maps_to_domain_input() {
        let dto = NewCatalogItemDto {
            name: "Coaching".into(),
            kind: CatalogItemKindDto::Service,
            default_price: MoneyDto {
                amount_cents: 20000,
                currency: "EUR".into(),
            },
            unit: Some("session".into()),
            reference: Some("COACH-1".into()),
        };
        let input: NewCatalogItem = dto.try_into().unwrap();
        assert_eq!(input.name, "Coaching");
        assert_eq!(input.default_price.amount_cents, 20000);
        assert_eq!(input.unit.as_deref(), Some("session"));
        assert_eq!(input.reference.as_deref(), Some("COACH-1"));
    }

    #[test]
    fn update_catalog_item_dto_rejects_invalid_currency() {
        let dto = UpdateCatalogItemDto {
            id: Uuid::new_v4(),
            name: "X".into(),
            kind: CatalogItemKindDto::Product,
            default_price: MoneyDto {
                amount_cents: 100,
                currency: "xx".into(),
            },
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
