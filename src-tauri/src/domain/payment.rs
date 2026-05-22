use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::aggregate_root::AggregateRoot;
use crate::domain::client::ClientId;
use crate::domain::events::payment_events::{PaymentRecorded, PaymentUpdated};
use crate::domain::events::EventBuffer;
use crate::domain::field_change::{money_to_value, DiffableValue, FieldChange};
use crate::domain::invoice::InvoiceId;
use crate::domain::money::{Currency, Money, MoneyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentAllocation {
    pub invoice_id: InvoiceId,
    pub amount: Money,
}

impl DiffableValue for PaymentAllocation {
    fn audit_key(&self) -> String {
        // Domain forbids duplicate invoice_id within a single payment, so
        // it is a stable per-element identity. The audit handler enriches
        // the label post-hoc with the invoice number (e.g. `"#1001"`)
        // since it needs a repo to resolve.
        self.invoice_id.to_string()
    }
    fn to_audit_json(&self) -> serde_json::Value {
        money_to_value(&self.amount)
    }
    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        // Only `amount` can change in place — `invoice_id` is the identity
        // key, so a different invoice_id means a different row entirely
        // (handled by added/removed, not changed).
        FieldChange::money("amount", &before.amount, &self.amount)
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Domain events buffered by mutating methods, drained by the use case
    /// after persistence. Not persisted; a row loaded from SQLite always has
    /// this empty. See [`EventBuffer`] for why this keeps the `derive`s intact.
    pub pending_events: EventBuffer,
}

impl AggregateRoot for Payment {
    fn pending_events_mut(&mut self) -> &mut EventBuffer {
        &mut self.pending_events
    }

    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        // `id`, `client_id`, and `created_at` are immutable post-creation
        // and intentionally omitted.
        [
            FieldChange::scalar("date", &before.date, &self.date),
            FieldChange::money("amount", &before.amount, &self.amount),
            // `PaymentMethod` has no Display impl; use the DB-form string.
            FieldChange::scalar(
                "method",
                &before.method.to_db_string(),
                &self.method.to_db_string(),
            ),
            FieldChange::opt("reference", &before.reference, &self.reference),
            FieldChange::opt("notes", &before.notes, &self.notes),
            // Allocations: per-element diff via `DiffableValue` keyed by
            // `invoice_id`. Changed entries carry a `Money` sub-diff
            // (`amount: €50 → €75`); added/removed entries carry the full
            // Money payload. Audit handler enriches each delta's `label`
            // with the resolved invoice number (e.g. `"#1001"`).
            FieldChange::diffable_collection(
                "allocations",
                &before.allocations,
                &self.allocations,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
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

#[derive(Debug, Clone)]
pub struct NewPayment {
    pub client_id: ClientId,
    pub date: NaiveDate,
    pub amount: Money,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocation>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewPaymentAllocation {
    pub invoice_id: InvoiceId,
    pub amount: Money,
}

impl Payment {
    pub fn create(input: NewPayment, now: DateTime<Utc>) -> Result<Self, PaymentError> {
        if !input.amount.minor_units().is_positive() {
            return Err(PaymentError::NonPositiveAmount);
        }
        let allocations = validate_allocations(&input.allocations, input.amount)?;
        let mut payment = Self {
            id: PaymentId::new(),
            client_id: input.client_id,
            date: input.date,
            amount: input.amount,
            method: input.method,
            reference: input.reference.and_then(non_empty),
            allocations,
            notes: input.notes.and_then(non_empty),
            created_at: now,
            pending_events: EventBuffer::default(),
        };
        payment.apply(PaymentRecorded {
            id: payment.id,
            client_id: payment.client_id,
            amount: payment.amount,
            allocations: payment.allocations.clone(),
            at: now,
        });
        Ok(payment)
    }

    pub fn replace_fields(
        &mut self,
        date: NaiveDate,
        amount: Money,
        method: PaymentMethod,
        reference: Option<String>,
        allocations: Vec<NewPaymentAllocation>,
        notes: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<(), PaymentError> {
        if !amount.minor_units().is_positive() {
            return Err(PaymentError::NonPositiveAmount);
        }
        let allocations = validate_allocations(&allocations, amount)?;
        // Snapshot prior state for the audit diff before any mutation.
        let before = self.clone();
        self.date = date;
        self.amount = amount;
        self.method = method;
        self.reference = reference.and_then(non_empty);
        self.allocations = allocations;
        self.notes = notes.and_then(non_empty);
        let changes = self.diff_against(&before);
        self.apply(PaymentUpdated {
            id: self.id,
            client_id: self.client_id,
            amount: self.amount,
            allocations: self.allocations.clone(),
            changes,
            at: now,
        });
        Ok(())
    }

    pub fn unallocated(&self) -> Money {
        let sum: i64 = self.allocations.iter().map(|a| a.amount.minor_units()).sum();
        Money::new(self.amount.minor_units() - sum, self.amount.currency())
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
        if a.amount.currency() != payment_amount.currency() {
            return Err(PaymentError::CurrencyMismatch);
        }
        if !a.amount.minor_units().is_positive() {
            return Err(PaymentError::NonPositiveAllocation);
        }
        if !seen.insert(a.invoice_id) {
            return Err(PaymentError::DuplicateAllocation);
        }
        total = total
            .checked_add(a.amount.minor_units())
            .ok_or(PaymentError::Money(MoneyError::Overflow))?;
        out.push(PaymentAllocation {
            invoice_id: a.invoice_id,
            amount: a.amount,
        });
    }
    if total > payment_amount.minor_units() {
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

/// Sum every allocation that targets `invoice_id`, returning the result in
/// `invoice_currency`. Callers must pass the invoice's own currency so the
/// zero-allocations case (no matching payments yet) returns a correctly-
/// typed `Money(0, invoice_currency)` rather than a misleading default.
///
/// Allocations that don't match `invoice_currency` are a programmer error —
/// strict silos forbid them at the use-case layer
/// ([`crate::application::payment_usecases::validate_cross_aggregate_allocations`])
/// and at the domain layer ([`Payment::validate_allocations`]). If one
/// slips through anyway, this function will detect the mismatch and return
/// `Money(0, invoice_currency)` rather than silently summing across
/// currencies.
pub fn compute_allocated_for_invoice(
    payments: &[Payment],
    invoice_id: InvoiceId,
    invoice_currency: Currency,
) -> Money {
    let mut sum: i64 = 0;
    for p in payments {
        for a in &p.allocations {
            if a.invoice_id == invoice_id && a.amount.currency() == invoice_currency {
                sum += a.amount.minor_units();
            }
        }
    }
    Money::new(sum, invoice_currency)
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
        amount_minor: i64,
        allocations: Vec<NewPaymentAllocation>,
    ) -> NewPayment {
        NewPayment {
            client_id: ClientId::new(),
            date: date(),
            amount: Money::new(amount_minor, eur()),
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
        assert_eq!(p.amount.minor_units(), 1000);
        assert!(p.allocations.is_empty());
        assert_eq!(p.unallocated().minor_units(), 1000);
    }

    #[test]
    fn create_payment_with_exact_allocation() {
        let invoice = InvoiceId::new();
        let p =
            Payment::create(new_payment(1000, vec![alloc(invoice, 1000)]), now()).unwrap();
        assert_eq!(p.allocations.len(), 1);
        assert_eq!(p.unallocated().minor_units(), 0);
    }

    #[test]
    fn create_payment_with_partial_allocation() {
        let invoice = InvoiceId::new();
        let p =
            Payment::create(new_payment(1000, vec![alloc(invoice, 600)]), now()).unwrap();
        assert_eq!(p.unallocated().minor_units(), 400);
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
            now(),
        )
        .unwrap();
        assert_eq!(p.amount.minor_units(), 2000);
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
                now(),
            )
            .unwrap_err();
        assert_eq!(err, PaymentError::AllocationsExceedPayment);
        // State must be unchanged on error.
        assert_eq!(p.amount.minor_units(), 1000);
    }

    // === Domain event emission ===

    #[test]
    fn create_buffers_payment_recorded_event() {
        let invoice = InvoiceId::new();
        let mut p =
            Payment::create(new_payment(1000, vec![alloc(invoice, 600)]), now()).unwrap();
        let events = p.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<PaymentRecorded>()
            .expect("PaymentRecorded");
        assert_eq!(ev.id, p.id);
        assert_eq!(ev.amount, p.amount);
        assert_eq!(ev.allocations.len(), 1);
        assert_eq!(ev.allocations[0].invoice_id, invoice);
    }

    #[test]
    fn replace_fields_buffers_payment_updated_event() {
        let mut p = Payment::create(new_payment(1000, vec![]), now()).unwrap();
        let _ = p.take_events(); // discard the recorded event
        p.replace_fields(
            date(),
            Money::new(2000, eur()),
            PaymentMethod::Cash,
            None,
            vec![],
            None,
            now(),
        )
        .unwrap();
        let events = p.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<PaymentUpdated>()
            .expect("PaymentUpdated");
        assert_eq!(ev.amount.minor_units(), 2000);
    }

    #[test]
    fn replace_fields_event_carries_diff_with_money_and_indexed_allocations() {
        // Start with a payment of 1000 EUR allocated entirely to invoice A.
        let inv_a = InvoiceId::new();
        let inv_b = InvoiceId::new();
        let mut p = Payment::create(
            new_payment(1000, vec![alloc(inv_a, 1000)]),
            now(),
        )
        .unwrap();
        let _ = p.take_events();

        // Update: bump amount to 1500, change method, drop A's allocation,
        // add a 1500 allocation to B.
        p.replace_fields(
            date(),
            Money::new(1500, eur()),
            PaymentMethod::Cash,
            Some("WIRE-9".into()),
            vec![alloc(inv_b, 1500)],
            None,
            now(),
        )
        .unwrap();

        let events = p.take_events();
        let ev = events[0].downcast_ref::<PaymentUpdated>().unwrap();
        let fields: Vec<&str> = ev.changes.iter().map(FieldChange::field).collect();

        assert!(fields.contains(&"amount"));
        assert!(fields.contains(&"method"));
        assert!(fields.contains(&"reference"));
        assert!(fields.contains(&"allocations"));
        // `date` and `notes` weren't touched.
        assert!(!fields.contains(&"date"));
        assert!(!fields.contains(&"notes"));

        // Allocations: A removed, B added, none changed.
        let allocs = ev.changes.iter().find(|c| c.field() == "allocations").unwrap();
        match allocs {
            FieldChange::IndexedCollection { added, removed, changed, .. } => {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].key, inv_b.to_string());
                assert_eq!(removed.len(), 1);
                assert_eq!(removed[0].key, inv_a.to_string());
                assert!(changed.is_empty());
            }
            _ => panic!("expected IndexedCollection for allocations"),
        }
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
        let allocated = compute_allocated_for_invoice(&[p1, p2], invoice, eur());
        assert_eq!(allocated.minor_units(), 700);
        assert_eq!(allocated.currency(), eur());
    }

    #[test]
    fn compute_allocated_returns_zero_in_invoice_currency_when_no_payments() {
        let invoice = InvoiceId::new();
        let allocated = compute_allocated_for_invoice(&[], invoice, eur());
        assert_eq!(allocated.minor_units(), 0);
        assert_eq!(allocated.currency(), eur());
    }
}
