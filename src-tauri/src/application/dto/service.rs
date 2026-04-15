use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use super::common::MoneyDto;
use crate::application::service_usecases::UpdateServiceInput;
use crate::domain::service::{NewService, Service, ServiceId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ServiceDto {
    pub id: Uuid,
    pub name: String,
    pub default_price: MoneyDto,
    pub active: bool,
}

impl From<&Service> for ServiceDto {
    fn from(s: &Service) -> Self {
        Self {
            id: s.id.0,
            name: s.name.clone(),
            default_price: (&s.default_price).into(),
            active: s.active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewServiceDto {
    pub name: String,
    pub default_price: MoneyDto,
}

impl TryFrom<NewServiceDto> for NewService {
    type Error = DtoConvertError;
    fn try_from(dto: NewServiceDto) -> Result<Self, Self::Error> {
        Ok(NewService {
            name: dto.name,
            default_price: (&dto.default_price).try_into()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateServiceDto {
    pub id: Uuid,
    pub name: String,
    pub default_price: MoneyDto,
}

impl TryFrom<UpdateServiceDto> for UpdateServiceInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdateServiceDto) -> Result<Self, Self::Error> {
        Ok(UpdateServiceInput {
            id: ServiceId(dto.id),
            name: dto.name,
            default_price: (&dto.default_price).try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::{Currency, Money};

    #[test]
    fn service_round_trip() {
        let eur = Currency::new("EUR").unwrap();
        let domain = Service {
            id: ServiceId::new(),
            name: "Consulting".into(),
            default_price: Money::new(15000, eur),
            active: true,
        };
        let dto: ServiceDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.default_price.amount_cents, 15000);
        assert_eq!(dto.default_price.currency, "EUR");
    }

    #[test]
    fn new_service_dto_maps_to_domain_input() {
        let dto = NewServiceDto {
            name: "Coaching".into(),
            default_price: MoneyDto {
                amount_cents: 20000,
                currency: "EUR".into(),
            },
        };
        let input: NewService = dto.try_into().unwrap();
        assert_eq!(input.name, "Coaching");
        assert_eq!(input.default_price.amount_cents, 20000);
    }

    #[test]
    fn update_service_dto_rejects_invalid_currency() {
        let dto = UpdateServiceDto {
            id: Uuid::new_v4(),
            name: "X".into(),
            default_price: MoneyDto {
                amount_cents: 100,
                currency: "xx".into(),
            },
        };
        let err = UpdateServiceInput::try_from(dto).unwrap_err();
        assert!(matches!(err, DtoConvertError::InvalidCurrency(_)));
    }
}
