use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use super::common::MoneyDto;
use crate::application::payment_usecases::UpdatePaymentInput;
use crate::application::ports::ListPaymentsQuery;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;
use crate::domain::payment::{
    NewPayment, NewPaymentAllocation, Payment, PaymentAllocation, PaymentId, PaymentMethod,
};

// ---- PaymentMethodDto (tagged enum) ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", content = "detail")]
pub enum PaymentMethodDto {
    BankTransfer,
    Cash,
    Check,
    Card,
    Other(String),
}

impl From<&PaymentMethod> for PaymentMethodDto {
    fn from(m: &PaymentMethod) -> Self {
        match m {
            PaymentMethod::BankTransfer => Self::BankTransfer,
            PaymentMethod::Cash => Self::Cash,
            PaymentMethod::Check => Self::Check,
            PaymentMethod::Card => Self::Card,
            PaymentMethod::Other(detail) => Self::Other(detail.clone()),
        }
    }
}

impl From<PaymentMethodDto> for PaymentMethod {
    fn from(dto: PaymentMethodDto) -> Self {
        match dto {
            PaymentMethodDto::BankTransfer => Self::BankTransfer,
            PaymentMethodDto::Cash => Self::Cash,
            PaymentMethodDto::Check => Self::Check,
            PaymentMethodDto::Card => Self::Card,
            PaymentMethodDto::Other(detail) => Self::Other(detail),
        }
    }
}

// ---- PaymentAllocationDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct PaymentAllocationDto {
    pub invoice_id: Uuid,
    pub amount: MoneyDto,
}

impl From<&PaymentAllocation> for PaymentAllocationDto {
    fn from(a: &PaymentAllocation) -> Self {
        Self {
            invoice_id: a.invoice_id.0,
            amount: (&a.amount).into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewPaymentAllocationDto {
    pub invoice_id: Uuid,
    pub amount: MoneyDto,
}

impl TryFrom<NewPaymentAllocationDto> for NewPaymentAllocation {
    type Error = DtoConvertError;
    fn try_from(dto: NewPaymentAllocationDto) -> Result<Self, Self::Error> {
        Ok(NewPaymentAllocation {
            invoice_id: InvoiceId(dto.invoice_id),
            amount: (&dto.amount).try_into()?,
        })
    }
}

// ---- PaymentDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct PaymentDto {
    pub id: Uuid,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub date: NaiveDate,
    pub amount: MoneyDto,
    pub method: PaymentMethodDto,
    pub reference: Option<String>,
    pub allocations: Vec<PaymentAllocationDto>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PaymentDto {
    /// Conversion for write paths where the joined client name isn't
    /// available. Leaves `client_name` as `None`; callers that need it
    /// must use [`PaymentDto::from_payment_enriched`].
    pub fn from_payment_basic(p: &Payment) -> Self {
        Self::build(p, None)
    }

    /// Conversion for read paths that know the joined client name.
    pub fn from_payment_enriched(p: &Payment, client_name: Option<String>) -> Self {
        Self::build(p, client_name)
    }

    fn build(p: &Payment, client_name: Option<String>) -> Self {
        Self {
            id: p.id.0,
            client_id: p.client_id.0,
            client_name,
            date: p.date,
            amount: (&p.amount).into(),
            method: (&p.method).into(),
            reference: p.reference.clone(),
            allocations: p.allocations.iter().map(Into::into).collect(),
            notes: p.notes.clone(),
            created_at: p.created_at,
        }
    }
}

impl From<&Payment> for PaymentDto {
    fn from(p: &Payment) -> Self {
        Self::from_payment_basic(p)
    }
}

impl From<Payment> for PaymentDto {
    fn from(p: Payment) -> Self {
        (&p).into()
    }
}

// ---- NewPaymentDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewPaymentDto {
    pub client_id: Uuid,
    pub date: NaiveDate,
    pub amount: MoneyDto,
    pub method: PaymentMethodDto,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocationDto>,
    pub notes: Option<String>,
}

impl TryFrom<NewPaymentDto> for NewPayment {
    type Error = DtoConvertError;
    fn try_from(dto: NewPaymentDto) -> Result<Self, Self::Error> {
        let allocations = dto
            .allocations
            .into_iter()
            .map(NewPaymentAllocation::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NewPayment {
            client_id: ClientId(dto.client_id),
            date: dto.date,
            amount: (&dto.amount).try_into()?,
            method: dto.method.into(),
            reference: dto.reference,
            allocations,
            notes: dto.notes,
        })
    }
}

// ---- UpdatePaymentDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdatePaymentDto {
    pub id: Uuid,
    pub date: NaiveDate,
    pub amount: MoneyDto,
    pub method: PaymentMethodDto,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocationDto>,
    pub notes: Option<String>,
}

impl TryFrom<UpdatePaymentDto> for UpdatePaymentInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdatePaymentDto) -> Result<Self, Self::Error> {
        let allocations = dto
            .allocations
            .into_iter()
            .map(NewPaymentAllocation::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UpdatePaymentInput {
            id: PaymentId(dto.id),
            date: dto.date,
            amount: (&dto.amount).try_into()?,
            method: dto.method.into(),
            reference: dto.reference,
            allocations,
            notes: dto.notes,
        })
    }
}

// ---- ListPaymentsQueryDto ----

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ListPaymentsQueryDto {
    #[serde(default)]
    pub client_id: Option<Uuid>,
    #[serde(default)]
    pub invoice_id: Option<Uuid>,
    #[serde(default)]
    pub search: Option<String>,
}

impl From<ListPaymentsQueryDto> for ListPaymentsQuery {
    fn from(dto: ListPaymentsQueryDto) -> Self {
        ListPaymentsQuery {
            client_id: dto.client_id.map(ClientId),
            invoice_id: dto.invoice_id.map(InvoiceId),
            search: dto.search,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::{Currency, Money};

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    #[test]
    fn payment_method_round_trip_covers_all_variants() {
        for method in [
            PaymentMethod::BankTransfer,
            PaymentMethod::Cash,
            PaymentMethod::Check,
            PaymentMethod::Card,
            PaymentMethod::Other("Crypto".into()),
        ] {
            let dto: PaymentMethodDto = (&method).into();
            let back: PaymentMethod = dto.into();
            assert_eq!(back, method);
        }
    }

    #[test]
    fn payment_to_dto_preserves_allocations() {
        let invoice_id = InvoiceId::new();
        let domain = Payment {
            id: PaymentId::new(),
            client_id: ClientId::new(),
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            amount: Money::new(1000, eur()),
            method: PaymentMethod::BankTransfer,
            reference: Some("WIRE-1".into()),
            allocations: vec![PaymentAllocation {
                invoice_id,
                amount: Money::new(1000, eur()),
            }],
            notes: None,
            created_at: Utc::now(),
        };
        let dto: PaymentDto = (&domain).into();
        assert_eq!(dto.allocations.len(), 1);
        assert_eq!(dto.allocations[0].invoice_id, invoice_id.0);
        assert_eq!(dto.allocations[0].amount.amount, 1000);
    }

    #[test]
    fn new_payment_dto_converts_all_allocations() {
        let dto = NewPaymentDto {
            client_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            amount: MoneyDto::from(Money::from_minor(500, Currency::Eur)),
            method: PaymentMethodDto::Cash,
            reference: None,
            allocations: vec![NewPaymentAllocationDto {
                invoice_id: Uuid::new_v4(),
                amount: MoneyDto::from(Money::from_minor(500, Currency::Eur)),
            }],
            notes: None,
        };
        let input: NewPayment = dto.try_into().unwrap();
        assert_eq!(input.allocations.len(), 1);
        assert_eq!(input.amount.minor_units(), 500);
    }
}
