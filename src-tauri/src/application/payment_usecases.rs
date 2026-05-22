use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::application::ports::{
    ClientRepository, CommitEvents, EventBus, InvoiceRepository, ListPaymentsQuery,
    NoopEventBus, PaymentRepository,
};
use crate::application::AppError;
#[cfg(test)] use crate::application::ErrorCode;
use crate::domain::events::payment_events::PaymentDeleted;
use crate::domain::invoice::InvoiceId;
use crate::domain::money::Money;
use crate::domain::payment::{
    NewPayment, NewPaymentAllocation, Payment, PaymentId, PaymentMethod,
};

#[derive(Clone)]
pub struct RecordPayment {
    payments: Arc<dyn PaymentRepository>,
    invoices: Arc<dyn InvoiceRepository>,
    events: Arc<dyn EventBus>,
}

impl RecordPayment {
    pub fn new(
        payments: Arc<dyn PaymentRepository>,
        invoices: Arc<dyn InvoiceRepository>,
    ) -> Self {
        Self {
            payments,
            invoices,
            events: Arc::new(NoopEventBus),
        }
    }

    /// Inject the real event bus. Production wiring (`OrgServices::new`) calls
    /// this; tests that don't assert on events keep the no-op default.
    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }

    pub fn execute(&self, input: NewPayment) -> Result<Payment, AppError> {
        validate_cross_aggregate_allocations(
            self.payments.as_ref(),
            self.invoices.as_ref(),
            &input.allocations,
            None,
        )?;
        let mut payment = Payment::create(input, Utc::now())?;
        self.payments.insert(&payment)?;
        payment.commit(self.events.as_ref());
        Ok(payment)
    }
}

#[derive(Debug, Clone)]
pub struct UpdatePaymentInput {
    pub id: PaymentId,
    pub date: NaiveDate,
    pub amount: Money,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocation>,
    pub notes: Option<String>,
}

pub struct UpdatePayment {
    payments: Arc<dyn PaymentRepository>,
    invoices: Arc<dyn InvoiceRepository>,
    events: Arc<dyn EventBus>,
}

impl UpdatePayment {
    pub fn new(
        payments: Arc<dyn PaymentRepository>,
        invoices: Arc<dyn InvoiceRepository>,
    ) -> Self {
        Self {
            payments,
            invoices,
            events: Arc::new(NoopEventBus),
        }
    }

    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }

    pub fn execute(&self, input: UpdatePaymentInput) -> Result<Payment, AppError> {
        let mut payment = self.payments.get(input.id)?.ok_or(AppError::resource_not_found())?;
        // Subtract this payment's current allocations from the "already
        // allocated" totals so a re-save of the same invoice doesn't
        // double-count itself.
        validate_cross_aggregate_allocations(
            self.payments.as_ref(),
            self.invoices.as_ref(),
            &input.allocations,
            Some(&payment),
        )?;
        payment.replace_fields(
            input.date,
            input.amount,
            input.method,
            input.reference,
            input.allocations,
            input.notes,
            Utc::now(),
        )?;
        self.payments.update(&payment)?;
        payment.commit(self.events.as_ref());
        Ok(payment)
    }
}

/// Enforces the cross-aggregate rule that `Payment::create` can't see:
/// `sum(allocations on an invoice across all payments) <= invoice.total`.
///
/// Also enforces the strict-silos rule that payment.currency == invoice.currency.
/// This check is lifted out of the invoice domain on purpose: a future FX
/// layer would relax this single condition (adding a per-allocation recorded
/// rate) without touching domain invariants.
///
/// `previous` is the payment being updated (or `None` for create). When
/// present, its own prior allocation on each invoice is subtracted from the
/// "already allocated" total so re-saving doesn't reject itself.
fn validate_cross_aggregate_allocations(
    payments: &dyn PaymentRepository,
    invoices: &dyn InvoiceRepository,
    new_allocations: &[NewPaymentAllocation],
    previous: Option<&Payment>,
) -> Result<(), AppError> {
    for alloc in new_allocations {
        let invoice = invoices
            .get(alloc.invoice_id)?
            .ok_or(AppError::resource_not_found())?;
        // Strict-silos: the allocation (and therefore the payment, since
        // Payment::create enforces allocation == payment currency) must be
        // in the same currency as the invoice. Reject up front with a
        // dedicated error code rather than letting can_accept_allocation
        // surface the lower-level InvoiceAllocationCurrencyMismatch.
        if alloc.amount.currency() != invoice.currency {
            return Err(AppError::failed_precondition(
                crate::application::ErrorCode::PaymentInvoiceCurrencyMismatch,
            ));
        }
        let total_allocated =
            payments.allocated_for_invoice(alloc.invoice_id, invoice.currency)?;
        let previous_self = previous
            .map(|p| sum_allocations_to(p, alloc.invoice_id, invoice.currency))
            .unwrap_or_else(|| Money::new(0, invoice.currency));
        // total_allocated already includes previous_self (since the existing
        // payment is still in the DB when validating the update), so subtract
        // it to get the allocations from *other* payments.
        let others_cents = total_allocated
            .minor_units()
            .saturating_sub(previous_self.minor_units());
        let others = Money::new(others_cents, invoice.currency);
        invoice.can_accept_allocation(others, alloc.amount)?;
    }
    Ok(())
}

fn sum_allocations_to(
    payment: &Payment,
    invoice_id: InvoiceId,
    currency: crate::domain::money::Currency,
) -> Money {
    let cents: i64 = payment
        .allocations
        .iter()
        .filter(|a| a.invoice_id == invoice_id)
        .map(|a| a.amount.minor_units())
        .sum();
    Money::new(cents, currency)
}

pub struct DeletePayment {
    repo: Arc<dyn PaymentRepository>,
    events: Arc<dyn EventBus>,
}

impl DeletePayment {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventBus),
        }
    }
    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }
    pub fn execute(&self, id: PaymentId) -> Result<(), AppError> {
        // Load before deleting so the `PaymentDeleted` event can carry the
        // client scope — the aggregate is gone by the time we publish, so
        // (unlike create/update) the use case dispatches the event directly
        // instead of draining it off the aggregate.
        let payment = self.repo.get(id)?.ok_or(AppError::resource_not_found())?;
        self.repo.delete(id)?;
        self.events.dispatch(&PaymentDeleted {
            id: payment.id,
            client_id: payment.client_id,
            at: Utc::now(),
        });
        Ok(())
    }
}

pub struct ListPayments {
    repo: Arc<dyn PaymentRepository>,
    clients: Arc<dyn ClientRepository>,
}

impl ListPayments {
    pub fn new(
        repo: Arc<dyn PaymentRepository>,
        clients: Arc<dyn ClientRepository>,
    ) -> Self {
        Self { repo, clients }
    }

    /// Returns each payment paired with its joined client display name.
    /// `None` only when the FK target was deleted (defensive — the schema
    /// normally enforces it).
    pub fn execute(
        &self,
        query: ListPaymentsQuery,
    ) -> Result<Vec<(Payment, Option<String>)>, AppError> {
        let payments = self.repo.list(query)?;
        let ids: Vec<crate::domain::client::ClientId> =
            payments.iter().map(|p| p.client_id).collect();
        let names = self.clients.labels_for(&ids)?;
        Ok(payments
            .into_iter()
            .map(|p| {
                let name = names.get(&p.client_id).cloned();
                (p, name)
            })
            .collect())
    }
}

pub struct GetPayment {
    repo: Arc<dyn PaymentRepository>,
    clients: Arc<dyn ClientRepository>,
}

impl GetPayment {
    pub fn new(
        repo: Arc<dyn PaymentRepository>,
        clients: Arc<dyn ClientRepository>,
    ) -> Self {
        Self { repo, clients }
    }

    pub fn execute(&self, id: PaymentId) -> Result<(Payment, Option<String>), AppError> {
        let payment = self.repo.get(id)?.ok_or(AppError::resource_not_found())?;
        let names = self.clients.labels_for(&[payment.client_id])?;
        let name = names.get(&payment.client_id).cloned();
        Ok((payment, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{ListInvoicesQuery, Page, PaginationParams};
    use crate::application::RepoError;
    use crate::domain::client::ClientId;
    use crate::domain::invoice::{Invoice, InvoiceId, InvoiceNumber, NewInvoice};
    use crate::domain::line_item::NewLineItem;
    use crate::domain::money::Currency;
    use parking_lot::Mutex;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryPaymentRepo {
        inner: Mutex<HashMap<PaymentId, Payment>>,
    }

    impl PaymentRepository for InMemoryPaymentRepo {
        fn insert(&self, p: &Payment) -> Result<(), RepoError> {
            self.inner.lock().insert(p.id, p.clone());
            Ok(())
        }
        fn update(&self, p: &Payment) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&p.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(p.id, p.clone());
            Ok(())
        }
        fn get(&self, id: PaymentId) -> Result<Option<Payment>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Payment> = g
                .values()
                .filter(|p| query.client_id.map(|c| c == p.client_id).unwrap_or(true))
                .cloned()
                .collect();
            v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(v)
        }
        fn delete(&self, id: PaymentId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
        fn allocated_for_invoice(
            &self,
            id: InvoiceId,
            invoice_currency: Currency,
        ) -> Result<Money, RepoError> {
            let g = self.inner.lock();
            let sum: i64 = g
                .values()
                .flat_map(|p| p.allocations.iter())
                .filter(|a| a.invoice_id == id && a.amount.currency() == invoice_currency)
                .map(|a| a.amount.minor_units())
                .sum();
            Ok(Money::new(sum, invoice_currency))
        }
        fn allocated_for_invoices(
            &self,
            ids: &[InvoiceId],
        ) -> Result<std::collections::HashMap<InvoiceId, Money>, RepoError> {
            let g = self.inner.lock();
            let mut out = std::collections::HashMap::new();
            for id in ids {
                let sum: i64 = g
                    .values()
                    .flat_map(|p| p.allocations.iter())
                    .filter(|a| a.invoice_id == *id)
                    .map(|a| a.amount.minor_units())
                    .sum();
                if sum > 0 {
                    out.insert(*id, Money::new(sum, Currency::new("EUR").unwrap()));
                }
            }
            Ok(out)
        }
    }

    /// Minimal client repo for tests — only `labels_for` is exercised by
    /// the read paths under test, so the rest are no-ops.
    #[derive(Default)]
    struct StubClientRepo;

    impl ClientRepository for StubClientRepo {
        fn insert(&self, _: &crate::domain::client::Client) -> Result<(), RepoError> {
            Ok(())
        }
        fn update(&self, _: &crate::domain::client::Client) -> Result<(), RepoError> {
            Ok(())
        }
        fn get(
            &self,
            _: ClientId,
        ) -> Result<Option<crate::domain::client::Client>, RepoError> {
            Ok(None)
        }
        fn list(
            &self,
            _: crate::application::ports::ListClientsQuery,
        ) -> Result<Page<crate::domain::client::Client>, RepoError> {
            Ok(Page::new(vec![], 0, &PaginationParams::default()))
        }
        fn labels_for(
            &self,
            _: &[ClientId],
        ) -> Result<HashMap<ClientId, String>, RepoError> {
            Ok(HashMap::new())
        }
        fn distinct_attribute_values(
            &self,
        ) -> Result<crate::application::ports::ClientAttributeValues, RepoError> {
            Ok(Default::default())
        }
    }

    /// In-memory stub for `InvoiceRepository`. Tests that don't care about
    /// allocation validation (e.g. use `allocations: vec![]`) can skip seeding
    /// entirely; tests that pass allocations must seed the invoice first or
    /// the use case will return `NotFound`.
    #[derive(Default)]
    struct StubInvoiceRepo {
        invoices: Mutex<HashMap<InvoiceId, Invoice>>,
    }

    impl StubInvoiceRepo {
        fn seed(&self, invoice: Invoice) {
            self.invoices.lock().insert(invoice.id, invoice);
        }
    }

    impl InvoiceRepository for StubInvoiceRepo {
        fn insert(&self, _: &Invoice) -> Result<(), RepoError> {
            Ok(())
        }
        fn update(&self, _: &Invoice) -> Result<(), RepoError> {
            Ok(())
        }
        fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError> {
            Ok(self.invoices.lock().get(&id).cloned())
        }
        fn list(&self, _: ListInvoicesQuery) -> Result<Page<Invoice>, RepoError> {
            let items: Vec<Invoice> = self.invoices.lock().values().cloned().collect();
            let total = items.len() as u64;
            Ok(Page::new(items, total, &PaginationParams::default()))
        }
        fn delete(&self, _: InvoiceId) -> Result<(), RepoError> {
            Ok(())
        }
        fn labels_for(
            &self,
            ids: &[InvoiceId],
        ) -> Result<std::collections::HashMap<InvoiceId, String>, RepoError> {
            let g = self.invoices.lock();
            Ok(ids
                .iter()
                .filter_map(|id| {
                    g.get(id).map(|i| {
                        (
                            *id,
                            i.number
                                .map(|n| format!("#{}", n.0))
                                .unwrap_or_else(|| String::new()),
                        )
                    })
                })
                .collect())
        }
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    fn make_finalized_invoice(total_cents: i64) -> Invoice {
        make_finalized_invoice_in(total_cents, eur())
    }

    fn make_finalized_invoice_in(total_cents: i64, currency: Currency) -> Invoice {
        let mut inv = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    id: None,
                    catalog_item_id: None,
                    description: "line".into(),
                    quantity: Decimal::from(1),
                    unit_price: Money::new(total_cents, currency),
                }],
                tax_ids: vec![],
                notes: None,
                currency,
            },
            &[],
            Utc::now(),
        )
        .unwrap();
        inv.finalize(InvoiceNumber(1), Utc::now()).unwrap();
        inv
    }

    fn repos() -> (Arc<InMemoryPaymentRepo>, Arc<StubInvoiceRepo>) {
        (
            Arc::new(InMemoryPaymentRepo::default()),
            Arc::new(StubInvoiceRepo::default()),
        )
    }

    fn new_input(amount: i64, allocations: Vec<NewPaymentAllocation>) -> NewPayment {
        NewPayment {
            client_id: ClientId::new(),
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            amount: Money::new(amount, eur()),
            method: PaymentMethod::BankTransfer,
            reference: None,
            allocations,
            notes: None,
        }
    }

    #[test]
    fn record_payment_persists_entity() {
        let (payments, invoices) = repos();
        let payment = RecordPayment::new(payments.clone(), invoices)
            .execute(new_input(1000, vec![]))
            .unwrap();
        assert_eq!(payment.amount.minor_units(), 1000);
        assert_eq!(payments.inner.lock().len(), 1);
    }

    #[test]
    fn record_payment_rejects_negative_amount() {
        let (payments, invoices) = repos();
        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(-1, vec![]))
            .unwrap_err();
        assert!(err.is(ErrorCode::PaymentNonPositiveAmount));
    }

    #[test]
    fn update_payment_rejects_missing_id() {
        let (payments, invoices) = repos();
        let err = UpdatePayment::new(payments, invoices)
            .execute(UpdatePaymentInput {
                id: PaymentId::new(),
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                amount: Money::new(500, eur()),
                method: PaymentMethod::Cash,
                reference: None,
                allocations: vec![],
                notes: None,
            })
            .unwrap_err();
        assert!(err.is(ErrorCode::ResourceNotFound));
    }

    #[test]
    fn update_payment_replaces_fields() {
        let (payments, invoices) = repos();
        let payment = RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        let updated = UpdatePayment::new(payments, invoices)
            .execute(UpdatePaymentInput {
                id: payment.id,
                date: payment.date,
                amount: Money::new(1500, eur()),
                method: PaymentMethod::Cash,
                reference: Some("REF".into()),
                allocations: vec![],
                notes: None,
            })
            .unwrap();
        assert_eq!(updated.amount.minor_units(), 1500);
        assert_eq!(updated.method, PaymentMethod::Cash);
        assert_eq!(updated.reference.as_deref(), Some("REF"));
    }

    #[test]
    fn delete_payment_removes_entity() {
        let (payments, invoices) = repos();
        let payment = RecordPayment::new(payments.clone(), invoices)
            .execute(new_input(1000, vec![]))
            .unwrap();
        DeletePayment::new(payments.clone())
            .execute(payment.id)
            .unwrap();
        assert!(payments.inner.lock().is_empty());
    }

    #[test]
    fn delete_payment_rejects_missing_id() {
        let (payments, _invoices) = repos();
        let err = DeletePayment::new(payments)
            .execute(PaymentId::new())
            .unwrap_err();
        assert!(err.is(ErrorCode::ResourceNotFound));
    }

    #[test]
    fn list_payments_filters_by_client() {
        let (payments, invoices) = repos();
        let record = RecordPayment::new(payments.clone(), invoices);
        let a = new_input(1000, vec![]);
        let client_a = a.client_id;
        record.execute(a.clone()).unwrap();
        let b = new_input(500, vec![]);
        record.execute(b).unwrap();
        let filtered = ListPayments::new(payments, Arc::new(StubClientRepo))
            .execute(ListPaymentsQuery {
                client_id: Some(client_a),
                invoice_id: None,
                search: None,
            })
            .unwrap();
        assert_eq!(filtered.len(), 1);
    }

    // --- Cross-aggregate over-allocation safeguard ---

    fn alloc(id: InvoiceId, cents: i64) -> NewPaymentAllocation {
        NewPaymentAllocation {
            invoice_id: id,
            amount: Money::new(cents, eur()),
        }
    }

    #[test]
    fn record_payment_rejects_overpayment_across_payments() {
        let (payments, invoices) = repos();
        let invoice = make_finalized_invoice(1000);
        let invoice_id = invoice.id;
        invoices.seed(invoice);

        // First payment uses €700 of the €1000 budget.
        RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(700, vec![alloc(invoice_id, 700)]))
            .unwrap();

        // Second payment tries to allocate another €400 → would total €1100.
        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(400, vec![alloc(invoice_id, 400)]))
            .unwrap_err();
        assert!(err.is(ErrorCode::InvoiceOverAllocated));
    }

    #[test]
    fn record_payment_rejects_any_allocation_when_already_fully_paid() {
        let (payments, invoices) = repos();
        let invoice = make_finalized_invoice(1000);
        let invoice_id = invoice.id;
        invoices.seed(invoice);

        // Fully pay the invoice.
        RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(1000, vec![alloc(invoice_id, 1000)]))
            .unwrap();

        // Second payment of €1 is refused even though the invoice domain
        // doesn't know about the first payment directly.
        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(1, vec![alloc(invoice_id, 1)]))
            .unwrap_err();
        assert!(err.is(ErrorCode::InvoiceOverAllocated));
    }

    #[test]
    fn record_payment_accepts_partial_fill_to_exact_total() {
        let (payments, invoices) = repos();
        let invoice = make_finalized_invoice(1000);
        let invoice_id = invoice.id;
        invoices.seed(invoice);

        RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(600, vec![alloc(invoice_id, 600)]))
            .unwrap();
        // Exact fit: 600 + 400 = 1000.
        RecordPayment::new(payments, invoices)
            .execute(new_input(400, vec![alloc(invoice_id, 400)]))
            .unwrap();
    }

    #[test]
    fn update_payment_can_resave_its_own_allocation() {
        // If a payment already fully covers an invoice, updating that same
        // payment to re-save the same allocation must not fail: the "already
        // allocated" count must subtract this payment's own prior amount.
        let (payments, invoices) = repos();
        let invoice = make_finalized_invoice(1000);
        let invoice_id = invoice.id;
        invoices.seed(invoice);

        let recorded = RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(1000, vec![alloc(invoice_id, 1000)]))
            .unwrap();

        UpdatePayment::new(payments, invoices)
            .execute(UpdatePaymentInput {
                id: recorded.id,
                date: recorded.date,
                amount: Money::new(1000, eur()),
                method: PaymentMethod::Cash,
                reference: Some("updated".into()),
                allocations: vec![alloc(invoice_id, 1000)],
                notes: None,
            })
            .unwrap();
    }

    #[test]
    fn update_payment_rejects_raising_allocation_beyond_available() {
        // Two payments split an invoice 500/500. Updating the second to try
        // to take 600 would push the total to 1100 → reject.
        let (payments, invoices) = repos();
        let invoice = make_finalized_invoice(1000);
        let invoice_id = invoice.id;
        invoices.seed(invoice);

        let record = RecordPayment::new(payments.clone(), invoices.clone());
        record
            .execute(new_input(500, vec![alloc(invoice_id, 500)]))
            .unwrap();
        let second = record
            .execute(new_input(500, vec![alloc(invoice_id, 500)]))
            .unwrap();

        let err = UpdatePayment::new(payments, invoices)
            .execute(UpdatePaymentInput {
                id: second.id,
                date: second.date,
                amount: Money::new(600, eur()),
                method: PaymentMethod::BankTransfer,
                reference: None,
                allocations: vec![alloc(invoice_id, 600)],
                notes: None,
            })
            .unwrap_err();
        assert!(err.is(ErrorCode::InvoiceOverAllocated));
    }

    #[test]
    fn record_payment_rejects_unknown_invoice() {
        let (payments, invoices) = repos();
        let missing = InvoiceId::new();
        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(500, vec![alloc(missing, 500)]))
            .unwrap_err();
        assert!(err.is(ErrorCode::ResourceNotFound));
    }

    #[test]
    fn record_payment_rejects_allocation_to_invoice_in_different_currency() {
        // Strict-silos: a EUR payment cannot be allocated to a USD invoice.
        // The use case must reject this with PaymentInvoiceCurrencyMismatch
        // — not InvoiceAllocationCurrencyMismatch (which is the lower-level
        // domain rule) — so a future FX layer can lift exactly this check.
        let (payments, invoices) = repos();
        let usd_invoice = make_finalized_invoice_in(1000, usd());
        invoices.seed(usd_invoice.clone());
        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(1000, vec![alloc(usd_invoice.id, 1000)]))
            .unwrap_err();
        assert!(
            err.is(ErrorCode::PaymentInvoiceCurrencyMismatch),
            "expected PaymentInvoiceCurrencyMismatch, got {err:?}"
        );
    }

    #[test]
    fn record_payment_rejects_draft_invoice_allocation() {
        let (payments, invoices) = repos();
        let draft = Invoice::create_draft(
            NewInvoice {
                client_id: ClientId::new(),
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    id: None,
                    catalog_item_id: None,
                    description: "x".into(),
                    quantity: Decimal::from(1),
                    unit_price: Money::new(1000, eur()),
                }],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            Utc::now(),
        )
        .unwrap();
        let draft_id = draft.id;
        invoices.seed(draft);

        let err = RecordPayment::new(payments, invoices)
            .execute(new_input(500, vec![alloc(draft_id, 500)]))
            .unwrap_err();
        assert!(err.is(ErrorCode::InvoiceNotAllocatable));
    }

    // === Domain event emission ===

    use crate::application::ports::event_bus::test_support::CollectingEventBus;

    #[test]
    fn record_payment_publishes_payment_recorded() {
        let (payments, invoices) = repos();
        let bus = Arc::new(CollectingEventBus::default());
        RecordPayment::new(payments, invoices)
            .with_events(bus.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        assert_eq!(bus.names(), ["payment.recorded"]);
    }

    #[test]
    fn update_payment_publishes_payment_updated() {
        let (payments, invoices) = repos();
        let created = RecordPayment::new(payments.clone(), invoices.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        let bus = Arc::new(CollectingEventBus::default());
        UpdatePayment::new(payments, invoices)
            .with_events(bus.clone())
            .execute(UpdatePaymentInput {
                id: created.id,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                amount: Money::new(2000, eur()),
                method: PaymentMethod::Cash,
                reference: None,
                allocations: vec![],
                notes: None,
            })
            .unwrap();
        assert_eq!(bus.names(), ["payment.updated"]);
    }

    #[test]
    fn delete_payment_publishes_payment_deleted() {
        let (payments, invoices) = repos();
        let created = RecordPayment::new(payments.clone(), invoices)
            .execute(new_input(1000, vec![]))
            .unwrap();
        let bus = Arc::new(CollectingEventBus::default());
        DeletePayment::new(payments)
            .with_events(bus.clone())
            .execute(created.id)
            .unwrap();
        assert_eq!(bus.names(), ["payment.deleted"]);
    }
}
