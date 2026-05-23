use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::catalog_item::CatalogItemId;
use crate::domain::field_change::{money_to_value, DiffableValue, FieldChange};
use crate::domain::money::{Money, MoneyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineItemId(pub Uuid);

impl LineItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LineItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LineItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineItem {
    pub id: LineItemId,
    /// Optional link back to the catalog item this line was seeded from.
    /// `unit_price` is a *snapshot* taken at creation time — the catalog's
    /// current price is never read back here, so existing invoices stay
    /// frozen if catalog prices later change. The link only powers
    /// per-item stats ("100 units sold of item X").
    pub catalog_item_id: Option<CatalogItemId>,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Money,
    pub total: Money,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LineItemError {
    #[error("line item description cannot be empty")]
    EmptyDescription,
    #[error("line item quantity must be positive")]
    NonPositiveQuantity,
    #[error("line item unit price cannot be negative")]
    NegativeUnitPrice,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone)]
pub struct NewLineItem {
    /// Existing id when editing a draft invoice in place; `None` for newly
    /// added rows. Preserving the id across saves keeps the audit log
    /// accurate (no false "list changed" diffs) and lets per-line
    /// references survive edits.
    pub id: Option<LineItemId>,
    pub catalog_item_id: Option<CatalogItemId>,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Money,
}

impl LineItem {
    pub fn create(input: NewLineItem) -> Result<Self, LineItemError> {
        let description = input.description.trim().to_string();
        if description.is_empty() {
            return Err(LineItemError::EmptyDescription);
        }
        if input.quantity <= Decimal::ZERO {
            return Err(LineItemError::NonPositiveQuantity);
        }
        if input.unit_price.is_negative() {
            return Err(LineItemError::NegativeUnitPrice);
        }
        let total = compute_total(input.quantity, input.unit_price)?;
        Ok(Self {
            id: input.id.unwrap_or_else(LineItemId::new),
            catalog_item_id: input.catalog_item_id,
            description,
            quantity: input.quantity,
            unit_price: input.unit_price,
            total,
        })
    }
}

impl DiffableValue for LineItem {
    fn audit_key(&self) -> String {
        self.id.0.to_string()
    }
    fn audit_label(&self) -> Option<String> {
        // Description is what the user sees on the invoice for this line;
        // makes per-line audit rows immediately readable ("Widget: quantity
        // 3 → 5" rather than "uuid abc-123: quantity 3 → 5").
        if self.description.is_empty() {
            None
        } else {
            Some(self.description.clone())
        }
    }
    fn to_audit_json(&self) -> serde_json::Value {
        serde_json::json!({
            "catalog_item_id": self.catalog_item_id.map(|c| c.0.to_string()),
            "description": self.description,
            "quantity": self.quantity.to_string(),
            "unit_price": money_to_value(&self.unit_price),
            "total": money_to_value(&self.total),
        })
    }
    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        // catalog_item_id intentionally omitted from the per-line sub-diff:
        // it's a stats-only back-link, not user-facing — surfacing it in the
        // audit feed would be noise.
        [
            FieldChange::scalar("description", &before.description, &self.description),
            FieldChange::number("quantity", &before.quantity, &self.quantity),
            FieldChange::money("unit_price", &before.unit_price, &self.unit_price),
            FieldChange::money("total", &before.total, &self.total),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

pub(crate) fn compute_total(quantity: Decimal, unit_price: Money) -> Result<Money, LineItemError> {
    // Money::multiply handles banker's rounding internally.
    unit_price
        .multiply(quantity)
        .map_err(LineItemError::Money)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Currency;
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn new_li(unit_price: Money, qty: Decimal) -> NewLineItem {
        NewLineItem {
            id: None,
            catalog_item_id: None,
            description: "Widget".into(),
            quantity: qty,
            unit_price,
        }
    }

    #[test]
    fn create_valid_line_item_computes_total() {
        let li = LineItem::create(new_li(Money::new(1000, eur()), dec!(3))).unwrap();
        assert_eq!(li.total.minor_units(), 3000);
        assert_eq!(li.catalog_item_id, None);
    }

    #[test]
    fn create_fractional_quantity_rounds() {
        let li = LineItem::create(new_li(Money::new(10000, eur()), dec!(2.5))).unwrap();
        assert_eq!(li.total.minor_units(), 25000);
    }

    #[test]
    fn create_preserves_catalog_item_id() {
        let cat_id = CatalogItemId::new();
        let li = LineItem::create(NewLineItem {
            id: None,
            catalog_item_id: Some(cat_id),
            description: "From catalog".into(),
            quantity: dec!(1),
            unit_price: Money::new(500, eur()),
        })
        .unwrap();
        assert_eq!(li.catalog_item_id, Some(cat_id));
    }

    #[test]
    fn create_rejects_empty_description() {
        let mut li = new_li(Money::zero(eur()), dec!(1));
        li.description = "  ".into();
        let err = LineItem::create(li).unwrap_err();
        assert_eq!(err, LineItemError::EmptyDescription);
    }

    #[test]
    fn create_rejects_zero_or_negative_quantity() {
        for q in [dec!(0), dec!(-1)] {
            let err = LineItem::create(new_li(Money::zero(eur()), q)).unwrap_err();
            assert_eq!(err, LineItemError::NonPositiveQuantity);
        }
    }

    #[test]
    fn create_rejects_negative_unit_price() {
        let err = LineItem::create(new_li(Money::new(-1, eur()), dec!(1))).unwrap_err();
        assert_eq!(err, LineItemError::NegativeUnitPrice);
    }
}
