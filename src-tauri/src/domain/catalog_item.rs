use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::money::{Currency, Money, MoneyError};

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
    /// Per-currency prices. Invariant: at most one `Money` per `Currency`,
    /// every entry is non-negative. May be empty — an item with no prices
    /// is valid (and means the user enters the unit price by hand each
    /// time the item is added to an invoice).
    pub prices: Vec<Money>,
    pub unit: Option<String>,
    pub reference: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CatalogItemError {
    #[error("catalog item name cannot be empty")]
    EmptyName,
    #[error("catalog item price cannot be negative")]
    NegativePrice,
    #[error("catalog item has more than one price for the same currency")]
    DuplicateCurrency,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone)]
pub struct NewCatalogItem {
    pub name: String,
    pub kind: CatalogItemKind,
    pub prices: Vec<Money>,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl CatalogItem {
    pub fn create(input: NewCatalogItem) -> Result<Self, CatalogItemError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(CatalogItemError::EmptyName);
        }
        let prices = validate_prices(input.prices)?;
        Ok(Self {
            id: CatalogItemId::new(),
            name,
            kind: input.kind,
            prices,
            unit: input.unit.and_then(non_empty),
            reference: input.reference.and_then(non_empty),
            archived_at: None,
        })
    }

    /// Returns the stored price for `currency`, or `None` if this item has
    /// no entry for that currency. Lookup is linear — the price list is
    /// expected to be small (one per supported currency at most).
    pub fn price_for(&self, currency: Currency) -> Option<Money> {
        self.prices.iter().find(|m| m.currency() == currency).copied()
    }

    pub fn replace_prices(&mut self, new_prices: Vec<Money>) -> Result<(), CatalogItemError> {
        self.prices = validate_prices(new_prices)?;
        Ok(())
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    pub fn archive(&mut self, now: DateTime<Utc>) {
        self.archived_at = Some(now);
    }

    pub fn unarchive(&mut self) {
        self.archived_at = None;
    }
}

fn validate_prices(input: Vec<Money>) -> Result<Vec<Money>, CatalogItemError> {
    let mut seen: std::collections::HashSet<Currency> = std::collections::HashSet::new();
    for m in &input {
        if m.is_negative() {
            return Err(CatalogItemError::NegativePrice);
        }
        if !seen.insert(m.currency()) {
            return Err(CatalogItemError::DuplicateCurrency);
        }
    }
    Ok(input)
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

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    fn new_service(name: &str) -> NewCatalogItem {
        NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur())],
            unit: Some("hour".into()),
            reference: None,
        }
    }

    #[test]
    fn create_with_valid_fields() {
        let s = CatalogItem::create(new_service("Consulting")).unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(s.kind, CatalogItemKind::Service);
        assert_eq!(s.prices.len(), 1);
        assert_eq!(s.prices[0].minor_units(), 15000);
        assert_eq!(s.unit.as_deref(), Some("hour"));
        assert!(!s.is_archived());
    }

    #[test]
    fn create_trims_name_and_optional_fields() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "  Consulting  ".into(),
            kind: CatalogItemKind::Service,
            prices: vec![],
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
            prices: vec![],
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
            prices: vec![Money::new(-1, eur())],
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
            prices: vec![Money::zero(eur())],
            unit: None,
            reference: None,
        })
        .unwrap();
        assert!(s.prices[0].is_zero());
    }

    #[test]
    fn create_allows_empty_prices_list() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "Custom Quote".into(),
            kind: CatalogItemKind::Service,
            prices: vec![],
            unit: None,
            reference: None,
        })
        .unwrap();
        assert!(s.prices.is_empty());
    }

    #[test]
    fn create_accepts_multiple_currencies() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur()), Money::new(17000, usd())],
            unit: None,
            reference: None,
        })
        .unwrap();
        assert_eq!(s.prices.len(), 2);
    }

    #[test]
    fn create_rejects_duplicate_currency() {
        let err = CatalogItem::create(NewCatalogItem {
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur()), Money::new(20000, eur())],
            unit: None,
            reference: None,
        })
        .unwrap_err();
        assert_eq!(err, CatalogItemError::DuplicateCurrency);
    }

    #[test]
    fn price_for_returns_matching_currency() {
        let s = CatalogItem::create(NewCatalogItem {
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur()), Money::new(17000, usd())],
            unit: None,
            reference: None,
        })
        .unwrap();
        assert_eq!(s.price_for(eur()).unwrap().minor_units(), 15000);
        assert_eq!(s.price_for(usd()).unwrap().minor_units(), 17000);
    }

    #[test]
    fn price_for_returns_none_for_missing_currency() {
        let s = CatalogItem::create(new_service("Consulting")).unwrap();
        let jpy = Currency::new("JPY").unwrap();
        assert!(s.price_for(jpy).is_none());
    }

    #[test]
    fn create_product_kind() {
        let p = CatalogItem::create(NewCatalogItem {
            name: "Book".into(),
            kind: CatalogItemKind::Product,
            prices: vec![Money::new(2500, eur())],
            unit: Some("piece".into()),
            reference: Some("SKU-042".into()),
        })
        .unwrap();
        assert_eq!(p.kind, CatalogItemKind::Product);
        assert_eq!(p.reference.as_deref(), Some("SKU-042"));
    }

    #[test]
    fn replace_prices_swaps_list_and_revalidates() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        s.replace_prices(vec![Money::new(20000, usd())]).unwrap();
        assert_eq!(s.prices.len(), 1);
        assert_eq!(s.prices[0].currency(), usd());
    }

    #[test]
    fn replace_prices_rejects_duplicate_currency() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        let err = s
            .replace_prices(vec![
                Money::new(1, eur()),
                Money::new(2, eur()),
            ])
            .unwrap_err();
        assert_eq!(err, CatalogItemError::DuplicateCurrency);
    }

    #[test]
    fn archive_stamps_timestamp() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        let now = Utc::now();
        s.archive(now);
        assert_eq!(s.archived_at, Some(now));
        assert!(s.is_archived());
    }

    #[test]
    fn unarchive_clears_timestamp() {
        let mut s = CatalogItem::create(new_service("Consulting")).unwrap();
        s.archive(Utc::now());
        s.unarchive();
        assert!(s.archived_at.is_none());
        assert!(!s.is_archived());
    }

    #[test]
    fn kind_round_trips_through_string() {
        for kind in [CatalogItemKind::Product, CatalogItemKind::Service] {
            assert_eq!(CatalogItemKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(CatalogItemKind::parse("unknown"), None);
    }
}
