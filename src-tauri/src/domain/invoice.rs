use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::domain::aggregate_root::AggregateRoot;
use crate::domain::client::ClientId;
use crate::domain::events::invoice_events::{
    InvoiceCancelled, InvoiceDraftCreated, InvoiceDraftUpdated, InvoiceFinalized,
};
use crate::domain::events::EventBuffer;
use crate::domain::field_change::{money_to_value, DiffableValue, FieldChange};
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

/// Width invoice numbers are zero-padded to when displayed (PDF, UI,
/// filenames). `1` renders as `0000001`. Numbers wider than this are shown
/// in full rather than truncated.
pub const INVOICE_NUMBER_DISPLAY_WIDTH: usize = 7;

impl std::fmt::Display for InvoiceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0width$}", self.0, width = INVOICE_NUMBER_DISPLAY_WIDTH)
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

impl DiffableValue for AppliedTax {
    fn audit_key(&self) -> String {
        // Stable identity: the tax definition's id when present, else the
        // name (legacy / hand-typed taxes have no FK back to a definition).
        // The name is a reasonable fallback because the unique-by-name
        // constraint at the use case layer keeps it from colliding.
        self.tax_definition_id
            .map(|id| id.0.to_string())
            .unwrap_or_else(|| self.tax_name.clone())
    }
    fn audit_label(&self) -> Option<String> {
        Some(self.tax_name.clone())
    }
    fn to_audit_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tax_definition_id": self.tax_definition_id.map(|id| id.0.to_string()),
            "tax_name": self.tax_name,
            "percentage": self.percentage.to_string(),
            "tax_id_number": self.tax_id_number,
            "computed_amount": money_to_value(&self.computed_amount),
        })
    }
    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        [
            FieldChange::scalar("tax_name", &before.tax_name, &self.tax_name),
            FieldChange::number("percentage", &before.percentage, &self.percentage),
            FieldChange::opt(
                "tax_id_number",
                &before.tax_id_number,
                &self.tax_id_number,
            ),
            FieldChange::money(
                "computed_amount",
                &before.computed_amount,
                &self.computed_amount,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
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
    /// Domain events buffered by mutating methods, drained by the use case
    /// after persistence. Not persisted; a row loaded from SQLite always has
    /// this empty. See [`EventBuffer`] for why this keeps the `derive`s intact.
    pub pending_events: EventBuffer,
}

impl AggregateRoot for Invoice {
    fn pending_events_mut(&mut self) -> &mut EventBuffer {
        &mut self.pending_events
    }

    fn diff_against(&self, before: &Self) -> Vec<FieldChange> {
        // Covers everything `update_draft` can mutate plus the derived
        // money totals (so a quantity bump shows "subtotal: 100 → 200" in
        // the audit row). `id`, `client_id`, `status`, `number`, `pdf_path`,
        // `created_at`, and `updated_at` are intentionally omitted: they
        // either don't change in `update_draft` or they're internal
        // bookkeeping rather than user-visible state.
        [
            FieldChange::opt("template_id", &before.template_id, &self.template_id),
            FieldChange::scalar("date", &before.date, &self.date),
            FieldChange::opt("due_date", &before.due_date, &self.due_date),
            FieldChange::scalar(
                "currency",
                before.currency.code(),
                self.currency.code(),
            ),
            FieldChange::money("subtotal", &before.subtotal, &self.subtotal),
            FieldChange::money("tax_total", &before.tax_total, &self.tax_total),
            FieldChange::money("total", &before.total, &self.total),
            FieldChange::opt("notes", &before.notes, &self.notes),
            FieldChange::diffable_collection("line_items", &before.line_items, &self.line_items),
            FieldChange::diffable_collection(
                "taxes_applied",
                &before.taxes_applied,
                &self.taxes_applied,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
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

        let mut invoice = Self {
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
            pending_events: EventBuffer::default(),
        };
        invoice.apply(InvoiceDraftCreated {
            id: invoice.id,
            client_id: invoice.client_id,
            total: invoice.total,
            at: now,
        });
        Ok(invoice)
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
        // Snapshot prior state for the audit diff. The clone is shallow over
        // line items + taxes_applied (small Vecs of Money/Decimal).
        let before = self.clone();
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
        let changes = self.diff_against(&before);
        self.apply(InvoiceDraftUpdated {
            id: self.id,
            client_id: self.client_id,
            changes,
            at: now,
        });
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
        self.apply(InvoiceFinalized {
            id: self.id,
            client_id: self.client_id,
            number,
            total: self.total,
            at: now,
        });
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
                self.apply(InvoiceCancelled {
                    id: self.id,
                    client_id: self.client_id,
                    number: self.number,
                    at: now,
                });
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
            id: None,
            catalog_item_id: None,
            description: desc.into(),
            quantity: Decimal::from(qty),
            unit_price: Money::new(price, eur()),
        }
    }

    #[test]
    fn invoice_number_displays_zero_padded_to_seven_digits() {
        assert_eq!(InvoiceNumber(1).to_string(), "0000001");
        assert_eq!(InvoiceNumber(42).to_string(), "0000042");
        assert_eq!(InvoiceNumber(1_234_567).to_string(), "1234567");
        // Numbers wider than the pad width are shown in full, never truncated.
        assert_eq!(InvoiceNumber(12_345_678).to_string(), "12345678");
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

    // === Domain event emission ===

    fn draft(total_cents: i64) -> Invoice {
        Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: date(),
                due_date: None,
                line_items: vec![line("A", 1, total_cents)],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            now(),
        )
        .unwrap()
    }

    #[test]
    fn create_draft_buffers_invoice_draft_created_event() {
        let mut inv = draft(1000);
        let events = inv.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<InvoiceDraftCreated>()
            .expect("InvoiceDraftCreated");
        assert_eq!(ev.id, inv.id);
        assert_eq!(ev.client_id, inv.client_id);
        assert_eq!(ev.total, inv.total);
    }

    #[test]
    fn update_draft_buffers_invoice_draft_updated_event() {
        let mut inv = draft(1000);
        let _ = inv.take_events(); // discard the draft-created event
        inv.update_draft(eur(), vec![line("B", 2, 500)], &[], None, date(), None, None, now())
            .unwrap();
        let events = inv.take_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].downcast_ref::<InvoiceDraftUpdated>().is_some());
    }

    #[test]
    fn update_draft_event_carries_field_diff_with_money_totals_and_line_count() {
        let mut inv = draft(1000); // 1 line, total 1000 EUR
        let _ = inv.take_events();
        // Replace one line with two cheaper ones — total goes 1000 → 1000
        // (2×500), but line_items count goes 1 → 2 and notes is added.
        inv.update_draft(
            eur(),
            vec![line("B", 1, 500), line("C", 1, 500)],
            &[],
            None,
            date(),
            None,
            Some("VIP".into()),
            now(),
        )
        .unwrap();

        let events = inv.take_events();
        let ev = events[0]
            .downcast_ref::<InvoiceDraftUpdated>()
            .expect("InvoiceDraftUpdated");

        let fields: Vec<&str> = ev.changes.iter().map(FieldChange::field).collect();
        assert!(fields.contains(&"notes"));        // None → Some
        assert!(fields.contains(&"line_items"));   // 1 → 2
        // Money totals didn't change (still 1000 + 0 + 1000), so they must
        // not appear:
        assert!(!fields.contains(&"subtotal"));
        assert!(!fields.contains(&"tax_total"));
        assert!(!fields.contains(&"total"));
        assert!(!fields.contains(&"currency"));

        // line_items: the original "A" line was dropped and two fresh
        // lines "B" and "C" replaced it, so the diff reports 2 added +
        // 1 removed. Element-level (not just count-only).
        let li = ev.changes.iter().find(|c| c.field() == "line_items").unwrap();
        match li {
            FieldChange::IndexedCollection { added, removed, changed, .. } => {
                assert_eq!(added.len(), 2);
                assert_eq!(removed.len(), 1);
                assert!(changed.is_empty());
            }
            _ => panic!("expected IndexedCollection for line_items"),
        }
    }

    #[test]
    fn finalize_buffers_invoice_finalized_event() {
        let mut inv = draft(1000);
        let _ = inv.take_events(); // discard the draft-created event
        inv.finalize(InvoiceNumber(7), now()).unwrap();
        let events = inv.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<InvoiceFinalized>()
            .expect("InvoiceFinalized");
        assert_eq!(ev.number, InvoiceNumber(7));
        assert_eq!(ev.total, inv.total);
    }

    #[test]
    fn cancel_buffers_invoice_cancelled_event() {
        let mut inv = draft(1000);
        inv.finalize(InvoiceNumber(7), now()).unwrap();
        let _ = inv.take_events(); // discard draft-created + finalized
        inv.cancel(now()).unwrap();
        let events = inv.take_events();
        assert_eq!(events.len(), 1);
        let ev = events[0]
            .downcast_ref::<InvoiceCancelled>()
            .expect("InvoiceCancelled");
        assert_eq!(ev.number, Some(InvoiceNumber(7)));
    }
}
