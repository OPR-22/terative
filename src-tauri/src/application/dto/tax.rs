use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::tax_usecases::UpdateTaxInput;
use crate::domain::tax::{NewTaxDefinition, TaxDefinition, TaxId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct TaxDefinitionDto {
    pub id: Uuid,
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<&TaxDefinition> for TaxDefinitionDto {
    fn from(t: &TaxDefinition) -> Self {
        Self {
            id: t.id.0,
            name: t.name.clone(),
            percentage: t.percentage,
            tax_id_number: t.tax_id_number.clone(),
            archived_at: t.archived_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewTaxDefinitionDto {
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
}

impl From<NewTaxDefinitionDto> for NewTaxDefinition {
    fn from(dto: NewTaxDefinitionDto) -> Self {
        NewTaxDefinition {
            name: dto.name,
            percentage: dto.percentage,
            tax_id_number: dto.tax_id_number,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateTaxDto {
    pub id: Uuid,
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
}

impl From<UpdateTaxDto> for UpdateTaxInput {
    fn from(dto: UpdateTaxDto) -> Self {
        UpdateTaxInput {
            id: TaxId(dto.id),
            name: dto.name,
            percentage: dto.percentage,
            tax_id_number: dto.tax_id_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn tax_round_trip() {
        let domain = TaxDefinition {
            id: TaxId::new(),
            name: "TVA".into(),
            percentage: dec!(21),
            tax_id_number: Some("BE123".into()),
            archived_at: None,
        };
        let dto: TaxDefinitionDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.percentage, dec!(21));
        assert_eq!(dto.tax_id_number.as_deref(), Some("BE123"));
    }

    #[test]
    fn new_tax_dto_maps_to_input() {
        let dto = NewTaxDefinitionDto {
            name: "VAT".into(),
            percentage: dec!(20),
            tax_id_number: None,
        };
        let input: NewTaxDefinition = dto.into();
        assert_eq!(input.name, "VAT");
    }

    #[test]
    fn update_tax_dto_preserves_id() {
        let id = Uuid::new_v4();
        let dto = UpdateTaxDto {
            id,
            name: "X".into(),
            percentage: dec!(5),
            tax_id_number: None,
        };
        let input: UpdateTaxInput = dto.into();
        assert_eq!(input.id.0, id);
    }
}
