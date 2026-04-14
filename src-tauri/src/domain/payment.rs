use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;
use crate::domain::money::{Currency, Money, MoneyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentId(pub Uuid);

impl PaymentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PaymentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PaymentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail")]
pub enum PaymentMethod {
    BankTransfer,
    Cash,
    Check,
    Card,
    Other(String),
}

impl PaymentMethod {
    pub fn to_db_string(&self) -> String {
        match self {
            Self::BankTransfer => "BankTransfer".into(),
            Self::Cash => "Cash".into(),
            Self::Check => "Check".into(),
            Self::Card => "Card".into(),
            Self::Other(s) => format!("Other:{s}"),
        }
    }

    pub fn parse_db_string(s: &str) -> Self {
        match s {
            "BankTransfer" => Self::BankTransfer,
            "Cash" => Self::Cash,
            "Check" => Self::Check,
            "Card" => Self::Card,
            other => {
                if let Some(rest) = other.strip_prefix("Other:") {
                    Self::Other(rest.to_string())
                } else {
                    Self::Other(other.to_string())
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAllocation {
    pub invoice_id: InvoiceId,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payment {
    pub id: PaymentId,
    pub client_id: ClientId,
    pub date: NaiveDate,
    pub amount: Money,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub allocations: Vec<PaymentAllocation>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PaymentError {
    #[error("payment amount must be positive")]
    NonPositiveAmount,
    #[error("allocation amount must be positive")]
    NonPositiveAllocation,
    #[error("allocations sum exceeds payment amount")]
    AllocationsExceedPayment,
    #[error("allocation currency does not match payment currency")]
    CurrencyMismatch,
    #[error("duplicate allocation for the same invoice")]
    DuplicateAllocation,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPayment {
    pub client_id: ClientId,
    pub date: NaiveDate,
    pub amount: Money,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocation>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPaymentAllocation {
    pub invoice_id: InvoiceId,
    pub amount: Money,
}

impl Payment {
    pub fn create(input: NewPayment, now: DateTime<Utc>) -> Result<Self, PaymentError> {
        if !input.amount.amount_cents.is_positive() {
            return Err(PaymentError::NonPositiveAmount);
        }
        let allocations = validate_allocations(&input.allocations, input.amount)?;
        Ok(Self {
            id: PaymentId::new(),
            client_id: input.client_id,
            date: input.date,
            amount: input.amount,
            method: input.method,
            reference: input.reference.and_then(non_empty),
            allocations,
            notes: input.notes.and_then(non_empty),
            created_at: now,
        })
    }

    pub fn replace_fields(
        &mut self,
        date: NaiveDate,
        amount: Money,
        method: PaymentMethod,
        reference: Option<String>,
        allocations: Vec<NewPaymentAllocation>,
        notes: Option<String>,
    ) -> Result<(), PaymentError> {
        if !amount.amount_cents.is_positive() {
            return Err(PaymentError::NonPositiveAmount);
        }
        let allocations = validate_allocations(&allocations, amount)?;
        self.date = date;
        self.amount = amount;
        self.method = method;
        self.reference = reference.and_then(non_empty);
        self.allocations = allocations;
        self.notes = notes.and_then(non_empty);
        Ok(())
    }

    pub fn unallocated(&self) -> Money {
        let sum: i64 = self.allocations.iter().map(|a| a.amount.amount_cents).sum();
        Money::new(self.amount.amount_cents - sum, self.amount.currency)
    }
}

fn validate_allocations(
    allocations: &[NewPaymentAllocation],
    payment_amount: Money,
) -> Result<Vec<PaymentAllocation>, PaymentError> {
    let mut seen: std::collections::HashSet<InvoiceId> = std::collections::HashSet::new();
    let mut total: i64 = 0;
    let mut out = Vec::with_capacity(allocations.len());
    for a in allocations {
        if a.amount.currency != payment_amount.currency {
            return Err(PaymentError::CurrencyMismatch);
        }
        if !a.amount.amount_cents.is_positive() {
            return Err(PaymentError::NonPositiveAllocation);
        }
        if !seen.insert(a.invoice_id) {
            return Err(PaymentError::DuplicateAllocation);
        }
        total = total
            .checked_add(a.amount.amount_cents)
            .ok_or(PaymentError::Money(MoneyError::Overflow))?;
        out.push(PaymentAllocation {
            invoice_id: a.invoice_id,
            amount: a.amount,
        });
    }
    if total > payment_amount.amount_cents {
        return Err(PaymentError::AllocationsExceedPayment);
    }
    Ok(out)
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

pub fn compute_allocated_for_invoice(payments: &[Payment], invoice_id: InvoiceId) -> Money {
    let mut sum: i64 = 0;
    let mut currency: Option<Currency> = None;
    for p in payments {
        for a in &p.allocations {
            if a.invoice_id == invoice_id {
                if currency.is_none() {
                    currency = Some(a.amount.currency);
                }
                sum += a.amount.amount_cents;
            }
        }
    }
    let currency = currency.unwrap_or_else(|| Currency::new("EUR").unwrap());
    Money::new(sum, currency)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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

    fn new_payment(
        amount_cents: i64,
        allocations: Vec<NewPaymentAllocation>,
    ) -> NewPayment {
        NewPayment {
            client_id: ClientId::new(),
            date: date(),
            amount: Money::new(amount_cents, eur()),
            method: PaymentMethod::BankTransfer,
            reference: None,
            allocations,
            notes: None,
        }
    }

    fn alloc(invoice: InvoiceId, cents: i64) -> NewPaymentAllocation {
        NewPaymentAllocation {
            invoice_id: invoice,
            amount: Money::new(cents, eur()),
        }
    }

    #[test]
    fn create_payment_with_no_allocations() {
        let p = Payment::create(new_payment(1000, vec![]), now()).unwrap();
        assert_eq!(p.amount.amount_cents, 1000);
        assert!(p.allocations.is_empty());
        assert_eq!(p.unallocated().amount_cents, 1000);
    }

    #[test]
    fn create_payment_with_exact_allocation() {
        let invoice = InvoiceId::new();
        let p =
            Payment::create(new_payment(1000, vec![alloc(invoice, 1000)]), now()).unwrap();
        assert_eq!(p.allocations.len(), 1);
        assert_eq!(p.unallocated().amount_cents, 0);
    }

    #[test]
    fn create_payment_with_partial_allocation() {
        let invoice = InvoiceId::new();
        let p =
            Payment::create(new_payment(1000, vec![alloc(invoice, 600)]), now()).unwrap();
        assert_eq!(p.unallocated().amount_cents, 400);
    }

    #[test]
    fn create_payment_rejects_zero_amount() {
        let err = Payment::create(new_payment(0, vec![]), now()).unwrap_err();
        assert_eq!(err, PaymentError::NonPositiveAmount);
    }

    #[test]
    fn create_payment_rejects_negative_amount() {
        let err = Payment::create(new_payment(-1, vec![]), now()).unwrap_err();
        assert_eq!(err, PaymentError::NonPositiveAmount);
    }

    #[test]
    fn create_payment_rejects_zero_allocation() {
        let invoice = InvoiceId::new();
        let err = Payment::create(new_payment(1000, vec![alloc(invoice, 0)]), now()).unwrap_err();
        assert_eq!(err, PaymentError::NonPositiveAllocation);
    }

    #[test]
    fn create_payment_rejects_over_allocation() {
        let invoice = InvoiceId::new();
        let err = Payment::create(new_payment(1000, vec![alloc(invoice, 1500)]), now())
            .unwrap_err();
        assert_eq!(err, PaymentError::AllocationsExceedPayment);
    }

    #[test]
    fn create_payment_rejects_duplicate_invoice_allocation() {
        let invoice = InvoiceId::new();
        let err = Payment::create(
            new_payment(1000, vec![alloc(invoice, 300), alloc(invoice, 300)]),
            now(),
        )
        .unwrap_err();
        assert_eq!(err, PaymentError::DuplicateAllocation);
    }

    #[test]
    fn create_payment_rejects_currency_mismatch() {
        let invoice = InvoiceId::new();
        let usd = Currency::new("USD").unwrap();
        let mut input = new_payment(1000, vec![]);
        input.allocations.push(NewPaymentAllocation {
            invoice_id: invoice,
            amount: Money::new(500, usd),
        });
        let err = Payment::create(input, now()).unwrap_err();
        assert_eq!(err, PaymentError::CurrencyMismatch);
    }

    #[test]
    fn payment_method_db_round_trip() {
        for method in [
            PaymentMethod::BankTransfer,
            PaymentMethod::Cash,
            PaymentMethod::Check,
            PaymentMethod::Card,
            PaymentMethod::Other("Crypto".into()),
        ] {
            let s = method.to_db_string();
            assert_eq!(PaymentMethod::parse_db_string(&s), method);
        }
    }

    #[test]
    fn replace_fields_updates_and_revalidates() {
        let invoice_a = InvoiceId::new();
        let invoice_b = InvoiceId::new();
        let mut p =
            Payment::create(new_payment(1000, vec![alloc(invoice_a, 500)]), now()).unwrap();
        p.replace_fields(
            date(),
            Money::new(2000, eur()),
            PaymentMethod::Cash,
            Some("INV-42".into()),
            vec![alloc(invoice_b, 2000)],
            None,
        )
        .unwrap();
        assert_eq!(p.amount.amount_cents, 2000);
        assert_eq!(p.method, PaymentMethod::Cash);
        assert_eq!(p.reference.as_deref(), Some("INV-42"));
        assert_eq!(p.allocations.len(), 1);
        assert_eq!(p.allocations[0].invoice_id, invoice_b);
    }

    #[test]
    fn replace_fields_rejects_over_allocation() {
        let invoice = InvoiceId::new();
        let mut p = Payment::create(new_payment(1000, vec![]), now()).unwrap();
        let err = p
            .replace_fields(
                date(),
                Money::new(500, eur()),
                PaymentMethod::Cash,
                None,
                vec![alloc(invoice, 600)],
                None,
            )
            .unwrap_err();
        assert_eq!(err, PaymentError::AllocationsExceedPayment);
        // State must be unchanged on error.
        assert_eq!(p.amount.amount_cents, 1000);
    }

    #[test]
    fn compute_allocated_sums_across_payments() {
        let invoice = InvoiceId::new();
        let other = InvoiceId::new();
        let p1 = Payment::create(
            new_payment(
                1000,
                vec![alloc(invoice, 300), alloc(other, 200)],
            ),
            now(),
        )
        .unwrap();
        let p2 = Payment::create(new_payment(1000, vec![alloc(invoice, 400)]), now()).unwrap();
        let allocated = compute_allocated_for_invoice(&[p1, p2], invoice);
        assert_eq!(allocated.amount_cents, 700);
    }
}
