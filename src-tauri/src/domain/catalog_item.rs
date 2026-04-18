use uuid::Uuid;

use crate::domain::money::{Money, MoneyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CatalogItemId(pub Uuid);

impl CatalogItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CatalogItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CatalogItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogItemKind {
    Product,
    Service,
}

impl CatalogItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Service => "Service",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Product" => Some(Self::Product),
            "Service" => Some(Self::Service),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItem {
    pub id: CatalogItemId,
    pub name: String,
    pub kind: CatalogItemKind,
    pub default_price: Money,
    /// Free-text billing unit, e.g. "hour", "day", "piece", "kg". `None` when
    /// the item has no natural unit (or the user didn't bother).
    pub unit: Option<String>,
    /// Optional internal reference / SKU. Free-text, searchable.
    pub reference: Option<String>,
    pub active: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CatalogItemError {
    #[error("catalog item name cannot be empty")]
    EmptyName,
    #[error("catalog item default price cannot be negative")]
    NegativePrice,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone)]
pub struct NewCatalogItem {
    pub name: String,
    pub kind: CatalogItemKind,
    pub default_price: Money,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl CatalogItem {
    pub fn create(input: NewCatalogItem) -> Result<Self, CatalogItemError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(CatalogItemError::EmptyName);
        }
        if input.default_price.is_negative() {
            return Err(CatalogItemError::NegativePrice);
        }
        Ok(Self {
            id: CatalogItemId::new(),
            name,
            kind: input.kind,
            default_price: input.default_price,
            unit: input.unit.and_then(non_empty),
            reference: input.reference.and_then(non_empty),
            active: true,
        })
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn reactivate(&mut self) {
        self.active = true;
    }
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Currency;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn new_service(name: &str) -> NewCatalogItem {
        NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Service,
            default_price: Money::new(15000, eur()),
            unit: Some("hour".into()),
            reference: None,
        }
    }

    #[test]
    fn create_with_valid_fields() {
        let s = CatalogItem::create(new_service("Consulting")).unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(s.kind, CatalogItemKind::Service);
        assert_eq!(s.default_price.minor_units(), 15000);
        assert_eq!(s.unit.as_deref(), Some("hour"));
        assert!(s.active);
    }

    #[test]
    fn create_trims_name_and_optional_fields() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "  Consulting  ".into(),
            kind: CatalogItemKind::Service,
            default_price: Money::zero(eur()),
            unit: Some("  hour  ".into()),
            reference: Some("   ".into()),
        })
        .unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(s.unit.as_deref(), Some("hour"));
        assert_eq!(s.reference, None, "blank reference becomes None");
    }

    #[test]
    fn create_rejects_empty_name() {
        let err = CatalogItem::create(NewCatalogItem {
            name: "".into(),
            kind: CatalogItemKind::Service,
            default_price: Money::zero(eur()),
            unit: None,
            reference: None,
        })
        .unwrap_err();
        assert_eq!(err, CatalogItemError::EmptyName);
    }

    #[test]
    fn create_rejects_negative_price() {
        let err = CatalogItem::create(NewCatalogItem {
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            default_price: Money::new(-1, eur()),
            unit: None,
            reference: None,
        })
        .unwrap_err();
        assert_eq!(err, CatalogItemError::NegativePrice);
    }

    #[test]
    fn create_allows_zero_price() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "Freebie".into(),
            kind: CatalogItemKind::Service,
            default_price: Money::zero(eur()),
            unit: None,
            reference: None,
        })
        .unwrap();
        assert!(s.default_price.is_zero());
    }

    #[test]
    fn create_product_kind() {
        let p = CatalogItem::create(NewCatalogItem {
            name: "Book".into(),
            kind: CatalogItemKind::Product,
            default_price: Money::new(2500, eur()),
            unit: Some("piece".into()),
            reference: Some("SKU-042".into()),
        })
        .unwrap();
        assert_eq!(p.kind, CatalogItemKind::Product);
        assert_eq!(p.reference.as_deref(), Some("SKU-042"));
    }

    #[test]
    fn deactivate_flips_active_flag() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        s.deactivate();
        assert!(!s.active);
    }

    #[test]
    fn reactivate_restores_active_flag() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        s.deactivate();
        s.reactivate();
        assert!(s.active);
    }

    #[test]
    fn kind_round_trips_through_string() {
        for kind in [CatalogItemKind::Product, CatalogItemKind::Service] {
            assert_eq!(CatalogItemKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(CatalogItemKind::parse("unknown"), None);
    }
}
