use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::domain::client::ClientId;
use crate::domain::line_item::{LineItem, LineItemError, NewLineItem};
use crate::domain::money::{Currency, Money, MoneyError};
use crate::domain::tax::{TaxDefinition, TaxId};
use crate::domain::template::TemplateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvoiceNumber(pub u64);

impl std::fmt::Display for InvoiceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// The user-facing payment state of an invoice, combining its raw lifecycle
/// (`InvoiceStatus`) with what's been allocated against it. This is the single
/// source of truth for the classification — the SQL view in
/// `migrations/001_initial.sql` mirrors these rules but the domain is
/// authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedPaymentStatus {
    Draft,
    Unpaid,
    Partial,
    Paid,
    Overdue,
    Cancelled,
}

impl DerivedPaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Unpaid => "Unpaid",
            Self::Partial => "Partial",
            Self::Paid => "Paid",
            Self::Overdue => "Overdue",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Draft" => Some(Self::Draft),
            "Unpaid" => Some(Self::Unpaid),
            "Partial" => Some(Self::Partial),
            "Paid" => Some(Self::Paid),
            "Overdue" => Some(Self::Overdue),
            "Cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedTax {
    pub tax_definition_id: Option<TaxId>,
    pub tax_name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
    pub computed_amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[error("invoice must be Finalized or Sent to send an email")]
    NotSendable,
    #[error("allocation exceeds invoice remaining balance")]
    OverAllocated,
    #[error("allocation currency does not match invoice currency")]
    AllocationCurrencyMismatch,
    #[error("cannot allocate payments to a {0:?} invoice")]
    NotAllocatable(InvoiceStatus),
    #[error(transparent)]
    LineItem(#[from] LineItemError),
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[derive(Debug, Clone)]
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
        currency: Currency,
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
        // Each line item's unit_price must already be in the target
        // currency. `compute_totals` below validates this transitively
        // (line.total inherits unit_price's currency and is compared to
        // `currency`), so an explicit pre-check would be redundant.
        let items: Vec<LineItem> = line_items
            .into_iter()
            .map(LineItem::create)
            .collect::<Result<_, _>>()?;
        let (subtotal, tax_total, total, taxes_applied) =
            compute_totals(&items, taxes, currency)?;
        self.currency = currency;
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

    /// Transitions a Finalized invoice to Sent (or leaves Sent as Sent).
    /// Persistence of which emails were sent and when lives in the
    /// `email_logs` table now; this method only owns the lifecycle bit.
    pub fn mark_sent(&mut self, now: DateTime<Utc>) -> Result<(), InvoiceError> {
        match self.status {
            InvoiceStatus::Finalized | InvoiceStatus::Sent => {
                self.status = InvoiceStatus::Sent;
                self.updated_at = now;
                Ok(())
            }
            _ => Err(InvoiceError::NotSendable),
        }
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

    /// Checks whether a new allocation against this invoice is legal, given
    /// everything that's *already* allocated to it across all other payments.
    ///
    /// This is the cross-aggregate invariant that `Payment::create` can't
    /// enforce alone (it has no visibility into other payments). Use cases
    /// that record or update a payment call this for each allocation to stop
    /// over-payment through stale UI, racing clicks, or direct IPC calls.
    ///
    /// Rules:
    /// 1. The invoice must be in an allocatable state (Finalized or Sent).
    ///    Draft and Cancelled invoices refuse all allocations.
    /// 2. All money values must be in the invoice's currency.
    /// 3. `already_allocated + new_allocation` must not exceed `total`.
    ///
    /// For the update path, callers are expected to pass `already_allocated`
    /// *net of this payment's existing allocation on this invoice*, so that a
    /// payment re-saving its own allocation doesn't count itself twice.
    pub fn can_accept_allocation(
        &self,
        already_allocated: Money,
        new_allocation: Money,
    ) -> Result<(), InvoiceError> {
        match self.status {
            InvoiceStatus::Finalized | InvoiceStatus::Sent => {}
            other => return Err(InvoiceError::NotAllocatable(other)),
        }
        if already_allocated.currency() != self.total.currency()
            || new_allocation.currency() != self.total.currency()
        {
            return Err(InvoiceError::AllocationCurrencyMismatch);
        }
        let sum = already_allocated
            .minor_units()
            .checked_add(new_allocation.minor_units())
            .ok_or(InvoiceError::Money(MoneyError::Overflow))?;
        if sum > self.total.minor_units() {
            return Err(InvoiceError::OverAllocated);
        }
        Ok(())
    }

    /// Classifies the invoice into its user-facing payment state.
    ///
    /// Rules (in order):
    /// 1. Raw `Draft` → `Draft`.
    /// 2. Raw `Cancelled` → `Cancelled`.
    /// 3. `amount_paid >= total` → `Paid`.
    /// 4. Overdue (past `due_date`) with a non-zero remainder → `Overdue`.
    /// 5. `amount_paid > 0` → `Partial`.
    /// 6. Otherwise → `Unpaid`.
    ///
    /// `amount_paid` is expected to be in the same currency as `total`; if it
    /// isn't, it's treated as zero (defensive — a currency mismatch here means
    /// the caller has stale data).
    pub fn payment_status(
        &self,
        amount_paid: Money,
        today: NaiveDate,
    ) -> DerivedPaymentStatus {
        match self.status {
            InvoiceStatus::Draft => return DerivedPaymentStatus::Draft,
            InvoiceStatus::Cancelled => return DerivedPaymentStatus::Cancelled,
            InvoiceStatus::Finalized | InvoiceStatus::Sent => {}
        }
        let paid_cents = if amount_paid.currency() == self.total.currency() {
            amount_paid.minor_units()
        } else {
            0
        };
        if paid_cents >= self.total.minor_units() {
            return DerivedPaymentStatus::Paid;
        }
        let is_overdue = self
            .due_date
            .map(|d| d < today)
            .unwrap_or(false);
        if is_overdue {
            return DerivedPaymentStatus::Overdue;
        }
        if paid_cents > 0 {
            DerivedPaymentStatus::Partial
        } else {
            DerivedPaymentStatus::Unpaid
        }
    }
}

fn compute_totals(
    items: &[LineItem],
    taxes: &[TaxDefinition],
    currency: Currency,
) -> Result<(Money, Money, Money, Vec<AppliedTax>), InvoiceError> {
    let mut subtotal = Money::zero(currency);
    for li in items {
        if li.total.currency() != currency {
            return Err(InvoiceError::Money(MoneyError::CurrencyMismatch {
                left: currency.to_string(),
                right: li.total.currency().to_string(),
            }));
        }
        subtotal = subtotal.add(li.total)?;
    }

    let mut taxes_applied: Vec<AppliedTax> = Vec::with_capacity(taxes.len());
    let mut tax_total = Money::zero(currency);
    let subtotal_dec = Decimal::from(subtotal.minor_units());
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
            catalog_item_id: None,
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
        assert_eq!(invoice.subtotal.minor_units(), 2500);
        assert_eq!(invoice.tax_total.minor_units(), 525);
        assert_eq!(invoice.total.minor_units(), 3025);
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
    fn mark_sent_requires_finalized_or_sent() {
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
            InvoiceError::NotSendable
        ));
        inv.finalize(InvoiceNumber(1), now()).unwrap();
        inv.mark_sent(now()).unwrap();
        assert_eq!(inv.status, InvoiceStatus::Sent);
        // Idempotent when already Sent.
        inv.mark_sent(now()).unwrap();
        assert_eq!(inv.status, InvoiceStatus::Sent);
    }

    fn finalized_invoice(due: Option<NaiveDate>, total_cents: i64) -> Invoice {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: due,
                line_items: vec![line("A", 1, total_cents)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap();
        inv.finalize(InvoiceNumber(1), now()).unwrap();
        inv
    }

    #[test]
    fn payment_status_draft_passthrough() {
        let inv = Invoice::create_draft(
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
        assert_eq!(
            inv.payment_status(Money::zero(eur()), date()),
            DerivedPaymentStatus::Draft,
        );
    }

    #[test]
    fn payment_status_cancelled_passthrough() {
        let mut inv = finalized_invoice(None, 1000);
        inv.cancel(now()).unwrap();
        assert_eq!(
            inv.payment_status(Money::zero(eur()), date()),
            DerivedPaymentStatus::Cancelled,
        );
    }

    #[test]
    fn payment_status_unpaid_when_no_allocations() {
        let inv = finalized_invoice(None, 1000);
        assert_eq!(
            inv.payment_status(Money::zero(eur()), date()),
            DerivedPaymentStatus::Unpaid,
        );
    }

    #[test]
    fn payment_status_partial_when_partly_allocated() {
        let inv = finalized_invoice(None, 1000);
        assert_eq!(
            inv.payment_status(Money::new(300, eur()), date()),
            DerivedPaymentStatus::Partial,
        );
    }

    #[test]
    fn payment_status_paid_when_fully_allocated() {
        let inv = finalized_invoice(None, 1000);
        assert_eq!(
            inv.payment_status(Money::new(1000, eur()), date()),
            DerivedPaymentStatus::Paid,
        );
    }

    #[test]
    fn payment_status_paid_when_over_allocated() {
        // Overpayment still classifies as Paid — the domain doesn't moralize
        // about the excess.
        let inv = finalized_invoice(None, 1000);
        assert_eq!(
            inv.payment_status(Money::new(1200, eur()), date()),
            DerivedPaymentStatus::Paid,
        );
    }

    #[test]
    fn payment_status_overdue_when_past_due_and_unpaid() {
        let due = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
        let inv = finalized_invoice(Some(due), 1000);
        assert_eq!(
            inv.payment_status(Money::zero(eur()), today),
            DerivedPaymentStatus::Overdue,
        );
    }

    #[test]
    fn payment_status_overdue_when_past_due_and_partial() {
        let due = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
        let inv = finalized_invoice(Some(due), 1000);
        assert_eq!(
            inv.payment_status(Money::new(300, eur()), today),
            DerivedPaymentStatus::Overdue,
        );
    }

    #[test]
    fn payment_status_paid_overrides_overdue() {
        let due = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
        let inv = finalized_invoice(Some(due), 1000);
        assert_eq!(
            inv.payment_status(Money::new(1000, eur()), today),
            DerivedPaymentStatus::Paid,
        );
    }

    #[test]
    fn payment_status_ignores_currency_mismatch_as_zero() {
        let inv = finalized_invoice(None, 1000);
        let usd = Currency::new("USD").unwrap();
        assert_eq!(
            inv.payment_status(Money::new(500, usd), date()),
            DerivedPaymentStatus::Unpaid,
        );
    }

    #[test]
    fn can_accept_allocation_accepts_partial() {
        let inv = finalized_invoice(None, 1000);
        inv.can_accept_allocation(Money::zero(eur()), Money::new(300, eur()))
            .unwrap();
    }

    #[test]
    fn can_accept_allocation_accepts_exact_fit() {
        let inv = finalized_invoice(None, 1000);
        inv.can_accept_allocation(Money::new(700, eur()), Money::new(300, eur()))
            .unwrap();
    }

    #[test]
    fn can_accept_allocation_rejects_exceeding_total() {
        let inv = finalized_invoice(None, 1000);
        let err = inv
            .can_accept_allocation(Money::new(800, eur()), Money::new(300, eur()))
            .unwrap_err();
        assert!(matches!(err, InvoiceError::OverAllocated));
    }

    #[test]
    fn can_accept_allocation_rejects_already_paid() {
        // Invoice already fully allocated; any new allocation is rejected.
        let inv = finalized_invoice(None, 1000);
        let err = inv
            .can_accept_allocation(Money::new(1000, eur()), Money::new(1, eur()))
            .unwrap_err();
        assert!(matches!(err, InvoiceError::OverAllocated));
    }

    #[test]
    fn can_accept_allocation_rejects_draft() {
        let inv = Invoice::create_draft(
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
        let err = inv
            .can_accept_allocation(Money::zero(eur()), Money::new(100, eur()))
            .unwrap_err();
        assert!(matches!(
            err,
            InvoiceError::NotAllocatable(InvoiceStatus::Draft)
        ));
    }

    #[test]
    fn can_accept_allocation_rejects_cancelled() {
        let mut inv = finalized_invoice(None, 1000);
        inv.cancel(now()).unwrap();
        let err = inv
            .can_accept_allocation(Money::zero(eur()), Money::new(100, eur()))
            .unwrap_err();
        assert!(matches!(
            err,
            InvoiceError::NotAllocatable(InvoiceStatus::Cancelled)
        ));
    }

    #[test]
    fn can_accept_allocation_rejects_currency_mismatch_on_new() {
        let inv = finalized_invoice(None, 1000);
        let usd = Currency::new("USD").unwrap();
        let err = inv
            .can_accept_allocation(Money::zero(eur()), Money::new(100, usd))
            .unwrap_err();
        assert!(matches!(err, InvoiceError::AllocationCurrencyMismatch));
    }

    #[test]
    fn can_accept_allocation_rejects_currency_mismatch_on_existing() {
        let inv = finalized_invoice(None, 1000);
        let usd = Currency::new("USD").unwrap();
        let err = inv
            .can_accept_allocation(Money::new(0, usd), Money::new(100, eur()))
            .unwrap_err();
        assert!(matches!(err, InvoiceError::AllocationCurrencyMismatch));
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
            eur(),
            vec![line("B", 2, 500)],
            &[tax],
            None,
            date(),
            None,
            None,
            now(),
        )
        .unwrap();
        assert_eq!(inv.subtotal.minor_units(), 1000);
        assert_eq!(inv.tax_total.minor_units(), 210);
        assert_eq!(inv.total.minor_units(), 1210);
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
            .update_draft(eur(), vec![line("A", 1, 1000)], &[], None, date(), None, None, now())
            .unwrap_err();
        assert!(matches!(err, InvoiceError::NotDraft));
    }
}
