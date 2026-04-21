use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DtoConvertError;
use super::accounting::DerivedPaymentStatusDto;
use super::common::MoneyDto;
use crate::application::invoice_usecases::UpdateDraftInvoiceInput;
use crate::application::ports::ListInvoicesQuery;
use crate::domain::client::ClientId;
use crate::application::dto::email_template::EmailTemplateTypeDto;
use crate::domain::invoice::{
    AppliedTax, EmailSend, Invoice, InvoiceId, InvoiceStatus, NewInvoice,
};
use crate::domain::line_item::{LineItem, NewLineItem};
#[cfg(test)]
use crate::domain::line_item::LineItemId;
use crate::domain::money::{Currency, Money};
use crate::domain::tax::TaxId;
use crate::domain::template::TemplateId;

// ---- InvoiceStatus ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum InvoiceStatusDto {
    Draft,
    Finalized,
    Sent,
    Cancelled,
}

impl From<InvoiceStatus> for InvoiceStatusDto {
    fn from(status: InvoiceStatus) -> Self {
        match status {
            InvoiceStatus::Draft => Self::Draft,
            InvoiceStatus::Finalized => Self::Finalized,
            InvoiceStatus::Sent => Self::Sent,
            InvoiceStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<InvoiceStatusDto> for InvoiceStatus {
    fn from(dto: InvoiceStatusDto) -> Self {
        match dto {
            InvoiceStatusDto::Draft => Self::Draft,
            InvoiceStatusDto::Finalized => Self::Finalized,
            InvoiceStatusDto::Sent => Self::Sent,
            InvoiceStatusDto::Cancelled => Self::Cancelled,
        }
    }
}

// ---- LineItemDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct LineItemDto {
    pub id: Uuid,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: MoneyDto,
    pub total: MoneyDto,
}

impl From<&LineItem> for LineItemDto {
    fn from(li: &LineItem) -> Self {
        Self {
            id: li.id.0,
            description: li.description.clone(),
            quantity: li.quantity,
            unit_price: (&li.unit_price).into(),
            total: (&li.total).into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewLineItemDto {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: MoneyDto,
}

impl TryFrom<NewLineItemDto> for NewLineItem {
    type Error = DtoConvertError;
    fn try_from(dto: NewLineItemDto) -> Result<Self, Self::Error> {
        Ok(NewLineItem {
            description: dto.description,
            quantity: dto.quantity,
            unit_price: (&dto.unit_price).try_into()?,
        })
    }
}

// ---- AppliedTaxDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AppliedTaxDto {
    pub tax_definition_id: Option<Uuid>,
    pub tax_name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
    pub computed_amount: MoneyDto,
}

impl From<&AppliedTax> for AppliedTaxDto {
    fn from(t: &AppliedTax) -> Self {
        Self {
            tax_definition_id: t.tax_definition_id.map(|id| id.0),
            tax_name: t.tax_name.clone(),
            percentage: t.percentage,
            tax_id_number: t.tax_id_number.clone(),
            computed_amount: (&t.computed_amount).into(),
        }
    }
}

// ---- EmailSendDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct EmailSendDto {
    pub id: Uuid,
    pub template_type: EmailTemplateTypeDto,
    pub template_name: String,
    pub to_address: String,
    pub subject: String,
    pub sent_at: DateTime<Utc>,
}

impl From<&EmailSend> for EmailSendDto {
    fn from(s: &EmailSend) -> Self {
        Self {
            id: s.id.0,
            template_type: s.template_type.into(),
            template_name: s.template_name.clone(),
            to_address: s.to_address.clone(),
            subject: s.subject.clone(),
            sent_at: s.sent_at,
        }
    }
}

// ---- InvoiceDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct InvoiceDto {
    pub id: Uuid,
    pub number: Option<u64>,
    pub client_id: Uuid,
    pub template_id: Option<Uuid>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<LineItemDto>,
    pub taxes_applied: Vec<AppliedTaxDto>,
    pub subtotal: MoneyDto,
    pub tax_total: MoneyDto,
    pub total: MoneyDto,
    pub amount_paid: MoneyDto,
    pub currency: String,
    pub status: InvoiceStatusDto,
    /// Populated by the list/get read paths, where the repo can afford to
    /// fetch the allocated total alongside the invoice. `None` on write paths
    /// (create/update/finalize/send/cancel) where callers don't need it.
    pub payment_status: Option<DerivedPaymentStatusDto>,
    pub pdf_path: Option<String>,
    pub notes: Option<String>,
    pub email_sends: Vec<EmailSendDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvoiceDto {
    /// Conversion for write paths where payment state is unknown. Sets
    /// `amount_paid` to zero and `payment_status` to `None`.
    pub fn from_invoice_basic(invoice: &Invoice) -> Self {
        let zero = Money::new(0, invoice.currency);
        Self::build(invoice, zero, None)
    }

    /// Conversion for read paths that know the allocated total. Computes the
    /// derived payment status via [`Invoice::payment_status`] so the domain
    /// owns the classification.
    pub fn from_invoice_enriched(
        invoice: &Invoice,
        amount_paid: Money,
        today: NaiveDate,
    ) -> Self {
        let status = invoice.payment_status(amount_paid, today).into();
        Self::build(invoice, amount_paid, Some(status))
    }

    fn build(
        i: &Invoice,
        amount_paid: Money,
        payment_status: Option<DerivedPaymentStatusDto>,
    ) -> Self {
        Self {
            id: i.id.0,
            number: i.number.map(|n| n.0),
            client_id: i.client_id.0,
            template_id: i.template_id.map(|t| t.0),
            date: i.date,
            due_date: i.due_date,
            line_items: i.line_items.iter().map(Into::into).collect(),
            taxes_applied: i.taxes_applied.iter().map(Into::into).collect(),
            subtotal: (&i.subtotal).into(),
            tax_total: (&i.tax_total).into(),
            total: (&i.total).into(),
            amount_paid: (&amount_paid).into(),
            currency: i.currency.code().to_string(),
            status: i.status.into(),
            payment_status,
            pdf_path: i.pdf_path.clone(),
            notes: i.notes.clone(),
            email_sends: i.email_sends.iter().map(Into::into).collect(),
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

// ---- NewInvoiceDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewInvoiceDto {
    pub client_id: Uuid,
    pub template_id: Option<Uuid>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<NewLineItemDto>,
    pub tax_ids: Vec<Uuid>,
    pub notes: Option<String>,
    pub currency: String,
}

impl TryFrom<NewInvoiceDto> for NewInvoice {
    type Error = DtoConvertError;
    fn try_from(dto: NewInvoiceDto) -> Result<Self, Self::Error> {
        let currency = Currency::new(&dto.currency)
            .map_err(|e| DtoConvertError::InvalidCurrency(e.to_string()))?;
        let line_items = dto
            .line_items
            .into_iter()
            .map(NewLineItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NewInvoice {
            client_id: ClientId(dto.client_id),
            template_id: dto.template_id.map(TemplateId),
            date: dto.date,
            due_date: dto.due_date,
            line_items,
            tax_ids: dto.tax_ids.into_iter().map(TaxId).collect(),
            notes: dto.notes,
            currency,
        })
    }
}

// ---- UpdateDraftInvoiceDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateDraftInvoiceDto {
    pub id: Uuid,
    pub template_id: Option<Uuid>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<NewLineItemDto>,
    pub tax_ids: Vec<Uuid>,
    pub notes: Option<String>,
}

impl TryFrom<UpdateDraftInvoiceDto> for UpdateDraftInvoiceInput {
    type Error = DtoConvertError;
    fn try_from(dto: UpdateDraftInvoiceDto) -> Result<Self, Self::Error> {
        let line_items = dto
            .line_items
            .into_iter()
            .map(NewLineItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UpdateDraftInvoiceInput {
            id: InvoiceId(dto.id),
            template_id: dto.template_id.map(TemplateId),
            date: dto.date,
            due_date: dto.due_date,
            line_items,
            tax_ids: dto.tax_ids.into_iter().map(TaxId).collect(),
            notes: dto.notes,
        })
    }
}

// ---- ListInvoicesQueryDto ----

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ListInvoicesQueryDto {
    #[serde(default)]
    pub status: Option<InvoiceStatusDto>,
    #[serde(default)]
    pub client_id: Option<Uuid>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub pagination: Option<super::PaginationParamsDto>,
}

impl ListInvoicesQueryDto {
    pub fn pagination_params(&self) -> crate::application::ports::PaginationParams {
        self.pagination.clone().into()
    }
}

impl From<ListInvoicesQueryDto> for ListInvoicesQuery {
    fn from(dto: ListInvoicesQueryDto) -> Self {
        ListInvoicesQuery {
            status: dto.status.map(Into::into),
            client_id: dto.client_id.map(ClientId),
            search: dto.search,
            pagination: dto.pagination.into(),
        }
    }
}

fn _unused_money_helper() -> Money {
    Money::new(0, Currency::new("EUR").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::invoice::InvoiceNumber;
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn sample_invoice() -> Invoice {
        Invoice {
            id: InvoiceId::new(),
            number: Some(InvoiceNumber(42)),
            client_id: ClientId::new(),
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
            line_items: vec![LineItem {
                id: LineItemId::new(),
                description: "Widget".into(),
                quantity: dec!(2),
                unit_price: Money::new(1000, eur()),
                total: Money::new(2000, eur()),
            }],
            taxes_applied: vec![AppliedTax {
                tax_definition_id: None,
                tax_name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: Some("BE0123".into()),
                computed_amount: Money::new(420, eur()),
            }],
            subtotal: Money::new(2000, eur()),
            tax_total: Money::new(420, eur()),
            total: Money::new(2420, eur()),
            currency: eur(),
            status: InvoiceStatus::Finalized,
            pdf_path: None,
            notes: None,
            email_sends: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn invoice_round_trip_preserves_fields() {
        let domain = sample_invoice();
        let dto = InvoiceDto::from_invoice_basic(&domain);
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.number, Some(42));
        assert_eq!(dto.line_items.len(), 1);
        assert_eq!(dto.line_items[0].total.amount_minor, 2000);
        assert_eq!(dto.taxes_applied.len(), 1);
        assert_eq!(dto.taxes_applied[0].computed_amount.amount_minor, 420);
        assert_eq!(dto.currency, "EUR");
        assert!(matches!(dto.status, InvoiceStatusDto::Finalized));
        assert_eq!(dto.amount_paid.amount_minor, 0);
        assert!(dto.payment_status.is_none());
    }

    #[test]
    fn enriched_dto_carries_computed_payment_status() {
        let domain = sample_invoice();
        let today = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
        let dto = InvoiceDto::from_invoice_enriched(
            &domain,
            Money::new(1000, eur()),
            today,
        );
        assert_eq!(dto.amount_paid.amount_minor, 1000);
        assert!(matches!(
            dto.payment_status,
            Some(DerivedPaymentStatusDto::Partial)
        ));
    }

    #[test]
    fn invoice_status_round_trips_through_dto() {
        for status in [
            InvoiceStatus::Draft,
            InvoiceStatus::Finalized,
            InvoiceStatus::Sent,
            InvoiceStatus::Cancelled,
        ] {
            let dto: InvoiceStatusDto = status.into();
            let back: InvoiceStatus = dto.into();
            assert_eq!(back, status);
        }
    }

    #[test]
    fn new_invoice_dto_converts_to_domain_input() {
        let dto = NewInvoiceDto {
            client_id: Uuid::new_v4(),
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: None,
            line_items: vec![NewLineItemDto {
                description: "Widget".into(),
                quantity: dec!(2),
                unit_price: MoneyDto {
                    amount_minor: 1000,
                    currency: "EUR".into(),
                },
            }],
            tax_ids: vec![],
            notes: None,
            currency: "EUR".into(),
        };
        let input: NewInvoice = dto.try_into().unwrap();
        assert_eq!(input.line_items.len(), 1);
        assert_eq!(input.currency.code(), "EUR");
    }

    #[test]
    fn new_invoice_dto_rejects_bad_currency() {
        let dto = NewInvoiceDto {
            client_id: Uuid::new_v4(),
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: None,
            line_items: vec![],
            tax_ids: vec![],
            notes: None,
            currency: "euro".into(),
        };
        assert!(matches!(
            NewInvoice::try_from(dto),
            Err(DtoConvertError::InvalidCurrency(_))
        ));
    }

    #[test]
    fn list_invoices_query_dto_default() {
        let dto: ListInvoicesQueryDto = serde_json::from_str("{}").unwrap();
        let q: ListInvoicesQuery = dto.into();
        assert!(q.status.is_none());
        assert!(q.client_id.is_none());
    }
}
