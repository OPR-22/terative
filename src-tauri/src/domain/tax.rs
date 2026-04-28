use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaxId(pub Uuid);

impl TaxId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaxId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxDefinition {
    pub id: TaxId,
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
    /// `None` = active. `Some(timestamp)` = archived; the timestamp records
    /// when the user clicked "archive".
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TaxError {
    #[error("tax name cannot be empty")]
    EmptyName,
    #[error("tax percentage cannot be negative")]
    NegativePercentage,
}

#[derive(Debug, Clone)]
pub struct NewTaxDefinition {
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
}

impl TaxDefinition {
    pub fn create(input: NewTaxDefinition) -> Result<Self, TaxError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(TaxError::EmptyName);
        }
        if input.percentage.is_sign_negative() {
            return Err(TaxError::NegativePercentage);
        }
        Ok(Self {
            id: TaxId::new(),
            name,
            percentage: input.percentage,
            tax_id_number: input
                .tax_id_number
                .and_then(|s| {
                    let t = s.trim().to_string();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                }),
            archived_at: None,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn create_tax_valid() {
        let t = TaxDefinition::create(NewTaxDefinition {
            name: "TVA".into(),
            percentage: dec!(21.0),
            tax_id_number: Some("BE0123".into()),
        })
        .unwrap();
        assert_eq!(t.name, "TVA");
        assert!(!t.is_archived());
    }

    #[test]
    fn create_tax_rejects_empty_name() {
        let err = TaxDefinition::create(NewTaxDefinition {
            name: "  ".into(),
            percentage: dec!(21),
            tax_id_number: None,
        })
        .unwrap_err();
        assert_eq!(err, TaxError::EmptyName);
    }

    #[test]
    fn create_tax_rejects_negative_percentage() {
        let err = TaxDefinition::create(NewTaxDefinition {
            name: "X".into(),
            percentage: dec!(-1),
            tax_id_number: None,
        })
        .unwrap_err();
        assert_eq!(err, TaxError::NegativePercentage);
    }

    #[test]
    fn create_tax_allows_zero_percentage() {
        let t = TaxDefinition::create(NewTaxDefinition {
            name: "Exempt".into(),
            percentage: dec!(0),
            tax_id_number: None,
        })
        .unwrap();
        assert_eq!(t.percentage, dec!(0));
    }

    #[test]
    fn unarchive_restores_active_state() {
        let mut t = TaxDefinition::create(NewTaxDefinition {
            name: "TVA".into(),
            percentage: dec!(21),
            tax_id_number: None,
        })
        .unwrap();
        t.archive(Utc::now());
        t.unarchive();
        assert!(!t.is_archived());
    }
}
