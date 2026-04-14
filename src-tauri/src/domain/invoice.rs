use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::client::ClientId;
use crate::domain::line_item::{LineItem, LineItemError, NewLineItem};
use crate::domain::money::{Currency, Money, MoneyError};
use crate::domain::tax::{TaxDefinition, TaxId};
use crate::domain::template::TemplateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvoiceId(pub Uuid);

impl InvoiceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for InvoiceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for InvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvoiceNumber(pub u64);

impl std::fmt::Display for InvoiceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum InvoiceStatus {
    Draft,
    Finalized,
    Sent,
    Cancelled,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Finalized => "Finalized",
            Self::Sent => "Sent",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Draft" => Some(Self::Draft),
            "Finalized" => Some(Self::Finalized),
            "Sent" => Some(Self::Sent),
            "Cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedTax {
    pub tax_definition_id: Option<TaxId>,
    pub tax_name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
    pub computed_amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceId,
    pub number: Option<InvoiceNumber>,
    pub client_id: ClientId,
    pub template_id: Option<TemplateId>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<LineItem>,
    pub taxes_applied: Vec<AppliedTax>,
    pub subtotal: Money,
    pub tax_total: Money,
    pub total: Money,
    pub currency: Currency,
    pub status: InvoiceStatus,
    pub pdf_path: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    #[error("invoice must have at least one line item to be finalized")]
    NoLineItems,
    #[error("invoice is not in Draft status")]
    NotDraft,
    #[error("cannot cancel a Draft invoice")]
    CannotCancelDraft,
    #[error("invoice already cancelled")]
    AlreadyCancelled,
    #[error("only Finalized invoices can be sent")]
    NotFinalized,
    #[error(transparent)]
    LineItem(#[from] LineItemError),
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewInvoice {
    pub client_id: ClientId,
    pub template_id: Option<TemplateId>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<NewLineItem>,
    pub tax_ids: Vec<TaxId>,
    pub notes: Option<String>,
    pub currency: Currency,
}

impl Invoice {
    pub fn create_draft(
        input: NewInvoice,
        taxes: &[TaxDefinition],
        now: DateTime<Utc>,
    ) -> Result<Self, InvoiceError> {
        let currency = input.currency;
        let line_items: Vec<LineItem> = input
            .line_items
            .into_iter()
            .map(LineItem::create)
            .collect::<Result<_, _>>()?;

        let (subtotal, tax_total, total, taxes_applied) =
            compute_totals(&line_items, taxes, currency)?;

        Ok(Self {
            id: InvoiceId::new(),
            number: None,
            client_id: input.client_id,
            template_id: input.template_id,
            date: input.date,
            due_date: input.due_date,
            line_items,
            taxes_applied,
            subtotal,
            tax_total,
            total,
            currency,
            status: InvoiceStatus::Draft,
            pdf_path: None,
            notes: input.notes.and_then(non_empty),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_draft(
        &mut self,
        line_items: Vec<NewLineItem>,
        taxes: &[TaxDefinition],
        template_id: Option<TemplateId>,
        date: NaiveDate,
        due_date: Option<NaiveDate>,
        notes: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<(), InvoiceError> {
        if self.status != InvoiceStatus::Draft {
            return Err(InvoiceError::NotDraft);
        }
        let items: Vec<LineItem> = line_items
            .into_iter()
            .map(LineItem::create)
            .collect::<Result<_, _>>()?;
        let (subtotal, tax_total, total, taxes_applied) =
            compute_totals(&items, taxes, self.currency)?;
        self.line_items = items;
        self.taxes_applied = taxes_applied;
        self.subtotal = subtotal;
        self.tax_total = tax_total;
        self.total = total;
        self.template_id = template_id;
        self.date = date;
        self.due_date = due_date;
        self.notes = notes.and_then(non_empty);
        self.updated_at = now;
        Ok(())
    }

    pub fn finalize(
        &mut self,
        number: InvoiceNumber,
        now: DateTime<Utc>,
    ) -> Result<(), InvoiceError> {
        if self.status != InvoiceStatus::Draft {
            return Err(InvoiceError::NotDraft);
        }
        if self.line_items.is_empty() {
            return Err(InvoiceError::NoLineItems);
        }
        self.number = Some(number);
        self.status = InvoiceStatus::Finalized;
        self.updated_at = now;
        Ok(())
    }

    pub fn mark_sent(&mut self, now: DateTime<Utc>) -> Result<(), InvoiceError> {
        if self.status != InvoiceStatus::Finalized {
            return Err(InvoiceError::NotFinalized);
        }
        self.status = InvoiceStatus::Sent;
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), InvoiceError> {
        match self.status {
            InvoiceStatus::Draft => Err(InvoiceError::CannotCancelDraft),
            InvoiceStatus::Cancelled => Err(InvoiceError::AlreadyCancelled),
            InvoiceStatus::Finalized | InvoiceStatus::Sent => {
                self.status = InvoiceStatus::Cancelled;
                self.updated_at = now;
                Ok(())
            }
        }
    }

    pub fn set_pdf_path(&mut self, path: String) {
        self.pdf_path = Some(path);
    }
}

fn compute_totals(
    items: &[LineItem],
    taxes: &[TaxDefinition],
    currency: Currency,
) -> Result<(Money, Money, Money, Vec<AppliedTax>), InvoiceError> {
    let mut subtotal = Money::zero(currency);
    for li in items {
        if li.total.currency != currency {
            return Err(InvoiceError::Money(MoneyError::CurrencyMismatch {
                left: currency.to_string(),
                right: li.total.currency.to_string(),
            }));
        }
        subtotal = subtotal.add(li.total)?;
    }

    let mut taxes_applied: Vec<AppliedTax> = Vec::with_capacity(taxes.len());
    let mut tax_total = Money::zero(currency);
    let subtotal_dec = Decimal::from(subtotal.amount_cents);
    for t in taxes {
        let computed = (subtotal_dec * t.percentage / dec!(100))
            .round()
            .to_i64()
            .ok_or(InvoiceError::Money(MoneyError::Overflow))?;
        let amount = Money::new(computed, currency);
        tax_total = tax_total.add(amount)?;
        taxes_applied.push(AppliedTax {
            tax_definition_id: Some(t.id),
            tax_name: t.name.clone(),
            percentage: t.percentage,
            tax_id_number: t.tax_id_number.clone(),
            computed_amount: amount,
        });
    }

    let total = subtotal.add(tax_total)?;
    Ok((subtotal, tax_total, total, taxes_applied))
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
    use crate::domain::tax::NewTaxDefinition;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-04-14T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 14).unwrap()
    }

    fn sample_tax() -> TaxDefinition {
        TaxDefinition::create(NewTaxDefinition {
            name: "TVA".into(),
            percentage: dec!(21),
            tax_id_number: Some("BE0123".into()),
        })
        .unwrap()
    }

    fn line(desc: &str, qty: i64, price: i64) -> NewLineItem {
        NewLineItem {
            description: desc.into(),
            quantity: Decimal::from(qty),
            unit_price: Money::new(price, eur()),
        }
    }

    #[test]
    fn create_draft_computes_subtotal_and_taxes() {
        let tax = sample_tax();
        let invoice = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 2, 1000), line("B", 1, 500)],
                tax_ids: vec![tax.id],
                notes: None,
                currency: eur(),
            },
            &[tax],
            now(),
        )
        .unwrap();
        assert_eq!(invoice.subtotal.amount_cents, 2500);
        assert_eq!(invoice.tax_total.amount_cents, 525);
        assert_eq!(invoice.total.amount_cents, 3025);
        assert_eq!(invoice.status, InvoiceStatus::Draft);
        assert!(invoice.number.is_none());
    }

    #[test]
    fn finalize_assigns_number_and_locks_draft() {
        let tax = sample_tax();
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![tax.id],
                notes: None,
                currency: eur(),
            },
            &[tax],
            now(),
        )
        .unwrap();
        inv.finalize(InvoiceNumber(42), now()).unwrap();
        assert_eq!(inv.status, InvoiceStatus::Finalized);
        assert_eq!(inv.number, Some(InvoiceNumber(42)));
        let err = inv.finalize(InvoiceNumber(43), now()).unwrap_err();
        assert!(matches!(err, InvoiceError::NotDraft));
    }

    #[test]
    fn finalize_rejects_empty() {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        let err = inv.finalize(InvoiceNumber(1), now()).unwrap_err();
        assert!(matches!(err, InvoiceError::NoLineItems));
    }

    #[test]
    fn cancel_draft_rejected() {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        let err = inv.cancel(now()).unwrap_err();
        assert!(matches!(err, InvoiceError::CannotCancelDraft));
    }

    #[test]
    fn cancel_finalized_succeeds() {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        inv.finalize(InvoiceNumber(1), now()).unwrap();
        inv.cancel(now()).unwrap();
        assert_eq!(inv.status, InvoiceStatus::Cancelled);
    }

    #[test]
    fn mark_sent_requires_finalized() {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        assert!(matches!(
            inv.mark_sent(now()).unwrap_err(),
            InvoiceError::NotFinalized
        ));
        inv.finalize(InvoiceNumber(1), now()).unwrap();
        inv.mark_sent(now()).unwrap();
        assert_eq!(inv.status, InvoiceStatus::Sent);
    }

    #[test]
    fn update_draft_recomputes_totals() {
        let tax = sample_tax();
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![tax.id],
                notes: None,
                currency: eur(),
            },
            &[tax.clone()],
            now(),
        )
        .unwrap();
        inv.update_draft(
            vec![line("B", 2, 500)],
            &[tax],
            None,
            date(),
            None,
            None,
            now(),
        )
        .unwrap();
        assert_eq!(inv.subtotal.amount_cents, 1000);
        assert_eq!(inv.tax_total.amount_cents, 210);
        assert_eq!(inv.total.amount_cents, 1210);
    }

    #[test]
    fn update_draft_rejected_when_finalized() {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, 1000)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        inv.finalize(InvoiceNumber(1), now()).unwrap();
        let err = inv
            .update_draft(vec![line("A", 1, 1000)], &[], None, date(), None, None, now())
            .unwrap_err();
        assert!(matches!(err, InvoiceError::NotDraft));
    }
}
