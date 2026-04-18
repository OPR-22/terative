use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

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
            id: LineItemId::new(),
            description,
            quantity: input.quantity,
            unit_price: input.unit_price,
            total,
        })
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

    #[test]
    fn create_valid_line_item_computes_total() {
        let li = LineItem::create(NewLineItem {
            description: "Widget".into(),
            quantity: dec!(3),
            unit_price: Money::new(1000, eur()),
        })
        .unwrap();
        assert_eq!(li.total.minor_units(), 3000);
    }

    #[test]
    fn create_fractional_quantity_rounds() {
        let li = LineItem::create(NewLineItem {
            description: "Hours".into(),
            quantity: dec!(2.5),
            unit_price: Money::new(10000, eur()),
        })
        .unwrap();
        assert_eq!(li.total.minor_units(), 25000);
    }

    #[test]
    fn create_rejects_empty_description() {
        let err = LineItem::create(NewLineItem {
            description: "  ".into(),
            quantity: dec!(1),
            unit_price: Money::zero(eur()),
        })
        .unwrap_err();
        assert_eq!(err, LineItemError::EmptyDescription);
    }

    #[test]
    fn create_rejects_zero_or_negative_quantity() {
        for q in [dec!(0), dec!(-1)] {
            let err = LineItem::create(NewLineItem {
                description: "W".into(),
                quantity: q,
                unit_price: Money::zero(eur()),
            })
            .unwrap_err();
            assert_eq!(err, LineItemError::NonPositiveQuantity);
        }
    }

    #[test]
    fn create_rejects_negative_unit_price() {
        let err = LineItem::create(NewLineItem {
            description: "W".into(),
            quantity: dec!(1),
            unit_price: Money::new(-1, eur()),
        })
        .unwrap_err();
        assert_eq!(err, LineItemError::NegativeUnitPrice);
    }
}
