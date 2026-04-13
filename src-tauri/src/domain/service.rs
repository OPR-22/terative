use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::money::{Money, MoneyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(pub Uuid);

impl ServiceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ServiceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub id: ServiceId,
    pub name: String,
    pub default_price: Money,
    pub active: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ServiceError {
    #[error("service name cannot be empty")]
    EmptyName,
    #[error("service default price cannot be negative")]
    NegativePrice,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewService {
    pub name: String,
    pub default_price: Money,
}

impl Service {
    pub fn create(input: NewService) -> Result<Self, ServiceError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ServiceError::EmptyName);
        }
        if input.default_price.is_negative() {
            return Err(ServiceError::NegativePrice);
        }
        Ok(Self {
            id: ServiceId::new(),
            name,
            default_price: input.default_price,
            active: true,
        })
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Currency;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    #[test]
    fn create_service_with_valid_fields() {
        let s = Service::create(NewService {
            name: "Consulting".into(),
            default_price: Money::new(15000, eur()),
        })
        .unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(s.default_price.amount_cents, 15000);
        assert!(s.active);
    }

    #[test]
    fn create_service_trims_name() {
        let s = Service::create(NewService {
            name: "  Consulting  ".into(),
            default_price: Money::zero(eur()),
        })
        .unwrap();
        assert_eq!(s.name, "Consulting");
    }

    #[test]
    fn create_service_rejects_empty_name() {
        let err = Service::create(NewService {
            name: "".into(),
            default_price: Money::zero(eur()),
        })
        .unwrap_err();
        assert_eq!(err, ServiceError::EmptyName);
    }

    #[test]
    fn create_service_rejects_negative_price() {
        let err = Service::create(NewService {
            name: "Consulting".into(),
            default_price: Money::new(-1, eur()),
        })
        .unwrap_err();
        assert_eq!(err, ServiceError::NegativePrice);
    }

    #[test]
    fn create_service_allows_zero_price() {
        let s = Service::create(NewService {
            name: "Freebie".into(),
            default_price: Money::zero(eur()),
        })
        .unwrap();
        assert!(s.default_price.is_zero());
    }

    #[test]
    fn deactivate_flips_active_flag() {
        let mut s = Service::create(NewService {
            name: "Consulting".into(),
            default_price: Money::zero(eur()),
        })
        .unwrap();
        s.deactivate();
        assert!(!s.active);
    }
}
