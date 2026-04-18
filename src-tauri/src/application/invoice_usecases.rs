use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::application::ports::{
    ClientRepository, InvoiceNumberGenerator, InvoiceRepository, ListInvoicesQuery, PaymentRepository,
    PdfGenerator, PdfRenderInput, PdfStorage, SettingsRepository, TaxRepository, TemplateRepository,
};
use crate::application::AppError;
use crate::domain::invoice::{Invoice, InvoiceId, InvoiceStatus, NewInvoice};
use crate::domain::money::Money;
use crate::domain::line_item::NewLineItem;
use crate::domain::tax::{TaxDefinition, TaxId};
use crate::domain::template::TemplateId;

fn load_taxes(repo: &dyn TaxRepository, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(repo.get_many(ids)?)
}

/// Resolve the template for a render pipeline (finalize / cancel).
///
/// Priority:
///   1. The explicit `template_id` on the invoice, if set.
///   2. The repository's default template (`is_default = true`).
///   3. An implicit in-memory template so the app still works even if the user
///      has no templates configured at all.
///
/// Missing an explicit template_id is a hard error (stale data); missing a
/// default is not.
fn resolve_template(
    repo: &dyn crate::application::ports::TemplateRepository,
    template_id: Option<crate::domain::template::TemplateId>,
) -> Result<crate::domain::template::InvoiceTemplate, AppError> {
    if let Some(tid) = template_id {
        return repo.get(tid)?.ok_or(AppError::NotFound);
    }
    if let Some(tpl) = repo.get_default()? {
        return Ok(tpl);
    }
    Ok(implicit_default_template())
}

pub(crate) fn implicit_default_template() -> crate::domain::template::InvoiceTemplate {
    crate::domain::template::InvoiceTemplate::create(
        crate::domain::template::NewInvoiceTemplate {
            name: "Default".into(),
            ..Default::default()
        },
    )
    .expect("implicit default template is always valid")
}

fn cancelled_watermark(lang: crate::domain::settings::Language) -> &'static str {
    match lang {
        crate::domain::settings::Language::Fr => "ANNULÉ",
        crate::domain::settings::Language::En => "CANCELLED",
    }
}

pub struct CreateDraftInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    taxes: Arc<dyn TaxRepository>,
}

impl CreateDraftInvoice {
    pub fn new(invoices: Arc<dyn InvoiceRepository>, taxes: Arc<dyn TaxRepository>) -> Self {
        Self { invoices, taxes }
    }
    pub fn execute(&self, input: NewInvoice) -> Result<Invoice, AppError> {
        let taxes = load_taxes(self.taxes.as_ref(), &input.tax_ids)?;
        let invoice = Invoice::create_draft(input, &taxes, Utc::now())?;
        self.invoices.insert(&invoice)?;
        Ok(invoice)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateDraftInvoiceInput {
    pub id: InvoiceId,
    pub template_id: Option<TemplateId>,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub line_items: Vec<NewLineItem>,
    pub tax_ids: Vec<TaxId>,
    pub notes: Option<String>,
}

pub struct UpdateDraftInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    taxes: Arc<dyn TaxRepository>,
}

impl UpdateDraftInvoice {
    pub fn new(invoices: Arc<dyn InvoiceRepository>, taxes: Arc<dyn TaxRepository>) -> Self {
        Self { invoices, taxes }
    }
    pub fn execute(&self, input: UpdateDraftInvoiceInput) -> Result<Invoice, AppError> {
        let mut invoice = self.invoices.get(input.id)?.ok_or(AppError::NotFound)?;
        let taxes = load_taxes(self.taxes.as_ref(), &input.tax_ids)?;
        invoice.update_draft(
            input.line_items,
            &taxes,
            input.template_id,
            input.date,
            input.due_date,
            input.notes,
            Utc::now(),
        )?;
        self.invoices.update(&invoice)?;
        Ok(invoice)
    }
}

pub struct FinalizeInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    numbers: Arc<dyn InvoiceNumberGenerator>,
    templates: Arc<dyn TemplateRepository>,
    settings: Arc<dyn SettingsRepository>,
    clients: Arc<dyn ClientRepository>,
    pdf: Arc<dyn PdfGenerator>,
    storage: Arc<dyn PdfStorage>,
}

impl FinalizeInvoice {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invoices: Arc<dyn InvoiceRepository>,
        numbers: Arc<dyn InvoiceNumberGenerator>,
        templates: Arc<dyn TemplateRepository>,
        settings: Arc<dyn SettingsRepository>,
        clients: Arc<dyn ClientRepository>,
        pdf: Arc<dyn PdfGenerator>,
        storage: Arc<dyn PdfStorage>,
    ) -> Self {
        Self {
            invoices,
            numbers,
            templates,
            settings,
            clients,
            pdf,
            storage,
        }
    }

    pub fn execute(&self, id: InvoiceId) -> Result<Invoice, AppError> {
        let mut invoice = self.invoices.get(id)?.ok_or(AppError::NotFound)?;
        let number = self.numbers.next()?;
        invoice.finalize(number, Utc::now())?;

        let template = resolve_template(self.templates.as_ref(), invoice.template_id)?;
        let seller = self.settings.get_seller_profile()?;
        let currency = self.settings.get_currency_config()?;
        let prefs = self.settings.get_app_preferences()?;
        let client = self
            .clients
            .get(invoice.client_id)?
            .ok_or(AppError::NotFound)?;

        let pdf_bytes = self.pdf.render(PdfRenderInput {
            invoice: &invoice,
            template: &template,
            seller: &seller,
            client: &client,
            currency: &currency,
            language: prefs.language,
            is_preview: false,
            watermark: None,
        })?;
        let file_name = format!("invoice-{}.pdf", number.0);
        let path = self.storage.store(&file_name, &pdf_bytes)?;
        invoice.set_pdf_path(path);
        self.invoices.update(&invoice)?;
        Ok(invoice)
    }
}

pub struct DuplicateInvoice {
    invoices: Arc<dyn InvoiceRepository>,
}

impl DuplicateInvoice {
    pub fn new(invoices: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoices }
    }
    pub fn execute(&self, id: InvoiceId) -> Result<Invoice, AppError> {
        let source = self.invoices.get(id)?.ok_or(AppError::NotFound)?;
        let now = Utc::now();
        let draft = Invoice {
            id: InvoiceId::new(),
            number: None,
            status: InvoiceStatus::Draft,
            pdf_path: None,
            created_at: now,
            updated_at: now,
            line_items: source
                .line_items
                .iter()
                .map(|li| crate::domain::line_item::LineItem {
                    id: crate::domain::line_item::LineItemId::new(),
                    description: li.description.clone(),
                    quantity: li.quantity,
                    unit_price: li.unit_price,
                    total: li.total,
                })
                .collect(),
            ..source
        };
        self.invoices.insert(&draft)?;
        Ok(draft)
    }
}

pub struct CancelInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    clients: Arc<dyn ClientRepository>,
    templates: Arc<dyn TemplateRepository>,
    settings: Arc<dyn SettingsRepository>,
    pdf: Arc<dyn PdfGenerator>,
    storage: Arc<dyn PdfStorage>,
}

impl CancelInvoice {
    pub fn new(
        invoices: Arc<dyn InvoiceRepository>,
        clients: Arc<dyn ClientRepository>,
        templates: Arc<dyn TemplateRepository>,
        settings: Arc<dyn SettingsRepository>,
        pdf: Arc<dyn PdfGenerator>,
        storage: Arc<dyn PdfStorage>,
    ) -> Self {
        Self {
            invoices,
            clients,
            templates,
            settings,
            pdf,
            storage,
        }
    }
    pub fn execute(&self, id: InvoiceId) -> Result<Invoice, AppError> {
        let mut invoice = self.invoices.get(id)?.ok_or(AppError::NotFound)?;
        invoice.cancel(Utc::now())?;

        // Re-render the PDF with a CANCELLED watermark. If the invoice was never
        // finalized with a PDF we just skip (cancel on a Finalized-never-rendered
        // path doesn't exist in practice, but the domain does allow cancelling a
        // Sent invoice whose pdf_path was already populated).
        if invoice.pdf_path.is_some() {
            let template = resolve_template(self.templates.as_ref(), invoice.template_id)?;
            let seller = self.settings.get_seller_profile()?;
            let currency = self.settings.get_currency_config()?;
            let prefs = self.settings.get_app_preferences()?;
            let client = self
                .clients
                .get(invoice.client_id)?
                .ok_or(AppError::NotFound)?;
            let watermark = cancelled_watermark(prefs.language);
            let bytes = self.pdf.render(PdfRenderInput {
                invoice: &invoice,
                template: &template,
                seller: &seller,
                client: &client,
                currency: &currency,
                language: prefs.language,
                is_preview: false,
                watermark: Some(watermark),
            })?;
            let file_name = format!(
                "invoice-{}.pdf",
                invoice
                    .number
                    .map(|n| n.0.to_string())
                    .unwrap_or_else(|| invoice.id.to_string())
            );
            let path = self.storage.store(&file_name, &bytes)?;
            invoice.set_pdf_path(path);
        }

        self.invoices.update(&invoice)?;
        Ok(invoice)
    }
}

pub struct ListInvoices {
    invoices: Arc<dyn InvoiceRepository>,
    payments: Arc<dyn PaymentRepository>,
}

impl ListInvoices {
    pub fn new(
        invoices: Arc<dyn InvoiceRepository>,
        payments: Arc<dyn PaymentRepository>,
    ) -> Self {
        Self { invoices, payments }
    }

    /// Returns each invoice alongside its currently allocated amount. The
    /// allocated amount's currency matches the invoice's currency when
    /// payments exist, or defaults to the invoice currency with zero cents.
    pub fn execute(
        &self,
        query: ListInvoicesQuery,
    ) -> Result<Vec<(Invoice, Money)>, AppError> {
        let list = self.invoices.list(query)?;
        let ids: Vec<InvoiceId> = list.iter().map(|i| i.id).collect();
        let totals = self.payments.allocated_for_invoices(&ids)?;
        Ok(list
            .into_iter()
            .map(|inv| {
                let paid = totals
                    .get(&inv.id)
                    .copied()
                    .unwrap_or_else(|| Money::new(0, inv.currency));
                (inv, paid)
            })
            .collect())
    }
}

pub struct GetInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    payments: Arc<dyn PaymentRepository>,
}

impl GetInvoice {
    pub fn new(
        invoices: Arc<dyn InvoiceRepository>,
        payments: Arc<dyn PaymentRepository>,
    ) -> Self {
        Self { invoices, payments }
    }

    pub fn execute(&self, id: InvoiceId) -> Result<(Invoice, Money), AppError> {
        let invoice = self.invoices.get(id)?.ok_or(AppError::NotFound)?;
        let paid = self.payments.allocated_for_invoice(id)?;
        Ok((invoice, paid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::client::ClientId;
    use crate::domain::money::{Currency, Money};
    use crate::domain::tax::{NewTaxDefinition, TaxDefinition};
    use chrono::NaiveDate;
    use parking_lot::Mutex;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryInvoiceRepo {
        inner: Mutex<HashMap<InvoiceId, Invoice>>,
    }

    impl InvoiceRepository for InMemoryInvoiceRepo {
        fn insert(&self, i: &Invoice) -> Result<(), RepoError> {
            self.inner.lock().insert(i.id, i.clone());
            Ok(())
        }
        fn update(&self, i: &Invoice) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&i.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(i.id, i.clone());
            Ok(())
        }
        fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, query: ListInvoicesQuery) -> Result<Vec<Invoice>, RepoError> {
            let mut v: Vec<Invoice> = self
                .inner
                .lock()
                .values()
                .filter(|i| query.status.map(|s| s == i.status).unwrap_or(true))
                .filter(|i| query.client_id.map(|c| c == i.client_id).unwrap_or(true))
                .cloned()
                .collect();
            v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(v)
        }
        fn delete(&self, id: InvoiceId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    /// Minimal stub so invoice use cases that depend on `PaymentRepository`
    /// can be wired in tests without bringing in the full payment aggregate.
    /// It reports zero allocations everywhere; tests that care about payment
    /// state should exercise the sqlite repo or the domain method directly.
    struct StubPaymentRepo;
    impl crate::application::ports::PaymentRepository for StubPaymentRepo {
        fn insert(
            &self,
            _: &crate::domain::payment::Payment,
        ) -> Result<(), RepoError> {
            Ok(())
        }
        fn update(
            &self,
            _: &crate::domain::payment::Payment,
        ) -> Result<(), RepoError> {
            Ok(())
        }
        fn get(
            &self,
            _: crate::domain::payment::PaymentId,
        ) -> Result<Option<crate::domain::payment::Payment>, RepoError> {
            Ok(None)
        }
        fn list(
            &self,
            _: crate::application::ports::ListPaymentsQuery,
        ) -> Result<Vec<crate::domain::payment::Payment>, RepoError> {
            Ok(vec![])
        }
        fn delete(
            &self,
            _: crate::domain::payment::PaymentId,
        ) -> Result<(), RepoError> {
            Ok(())
        }
        fn allocated_for_invoice(
            &self,
            _: InvoiceId,
        ) -> Result<Money, RepoError> {
            Ok(Money::new(0, Currency::new("EUR").unwrap()))
        }
        fn allocated_for_invoices(
            &self,
            _: &[InvoiceId],
        ) -> Result<HashMap<InvoiceId, Money>, RepoError> {
            Ok(HashMap::new())
        }
    }

    fn stub_payments() -> Arc<StubPaymentRepo> {
        Arc::new(StubPaymentRepo)
    }

    #[derive(Default)]
    struct InMemoryTaxRepo {
        inner: Mutex<HashMap<TaxId, TaxDefinition>>,
    }

    impl crate::application::ports::TaxRepository for InMemoryTaxRepo {
        fn insert(&self, t: &TaxDefinition) -> Result<(), RepoError> {
            self.inner.lock().insert(t.id, t.clone());
            Ok(())
        }
        fn update(&self, _: &TaxDefinition) -> Result<(), RepoError> {
            Ok(())
        }
        fn get(&self, id: TaxId) -> Result<Option<TaxDefinition>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, _: bool) -> Result<Vec<TaxDefinition>, RepoError> {
            Ok(self.inner.lock().values().cloned().collect())
        }
        fn get_many(&self, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError> {
            let g = self.inner.lock();
            Ok(ids.iter().filter_map(|id| g.get(id).cloned()).collect())
        }
        fn delete(&self, _: TaxId) -> Result<(), RepoError> {
            Ok(())
        }
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn setup() -> (
        Arc<InMemoryInvoiceRepo>,
        Arc<InMemoryTaxRepo>,
        TaxDefinition,
    ) {
        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let taxes = Arc::new(InMemoryTaxRepo::default());
        let tax = TaxDefinition::create(NewTaxDefinition {
            name: "TVA".into(),
            percentage: dec!(21),
            tax_id_number: None,
        })
        .unwrap();
        crate::application::ports::TaxRepository::insert(taxes.as_ref(), &tax).unwrap();
        (invoices, taxes, tax)
    }

    fn new_invoice_input(client_id: ClientId, tax_id: TaxId) -> NewInvoice {
        NewInvoice {
            client_id,
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: None,
            line_items: vec![NewLineItem {
                description: "Widget".into(),
                quantity: dec!(2),
                unit_price: Money::new(1000, eur()),
            }],
            tax_ids: vec![tax_id],
            notes: None,
            currency: eur(),
        }
    }

    #[test]
    fn create_draft_invoice_persists_with_computed_totals() {
        let (inv_repo, tax_repo, tax) = setup();
        let invoice = CreateDraftInvoice::new(inv_repo.clone(), tax_repo)
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Draft);
        assert_eq!(invoice.subtotal.minor_units(), 2000);
        assert_eq!(invoice.tax_total.minor_units(), 420);
        assert_eq!(invoice.total.minor_units(), 2420);
        assert_eq!(inv_repo.inner.lock().len(), 1);
    }

    #[test]
    fn update_draft_invoice_recomputes_totals() {
        let (inv_repo, tax_repo, tax) = setup();
        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo.clone())
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        let updated = UpdateDraftInvoice::new(inv_repo.clone(), tax_repo)
            .execute(UpdateDraftInvoiceInput {
                id: created.id,
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    description: "Bigger".into(),
                    quantity: dec!(5),
                    unit_price: Money::new(1000, eur()),
                }],
                tax_ids: vec![tax.id],
                notes: Some("updated".into()),
            })
            .unwrap();
        assert_eq!(updated.subtotal.minor_units(), 5000);
        assert_eq!(updated.tax_total.minor_units(), 1050);
        assert_eq!(updated.total.minor_units(), 6050);
        assert_eq!(updated.notes.as_deref(), Some("updated"));
    }

    #[test]
    fn update_draft_invoice_rejects_missing_id() {
        let (inv_repo, tax_repo, tax) = setup();
        let err = UpdateDraftInvoice::new(inv_repo, tax_repo)
            .execute(UpdateDraftInvoiceInput {
                id: InvoiceId::new(),
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![],
                tax_ids: vec![tax.id],
                notes: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    fn make_cancel(
        inv_repo: Arc<InMemoryInvoiceRepo>,
    ) -> (
        CancelInvoice,
        Arc<FakePdfGenerator>,
        Arc<CapturingPdfStorage>,
    ) {
        let client_repo = Arc::new(FakeClientRepo(Mutex::new(HashMap::new())));
        let tmpl_repo = Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new())));
        // Default template so CancelInvoice can resolve it when the invoice had one.
        let mut template = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Default".into(),
            ..Default::default()
        })
        .unwrap();
        template.is_default = true;
        tmpl_repo.insert(&template).unwrap();
        let pdf = Arc::new(FakePdfGenerator(Mutex::new(0)));
        let storage = Arc::new(CapturingPdfStorage::default());
        let uc = CancelInvoice::new(
            inv_repo,
            client_repo,
            tmpl_repo,
            Arc::new(FakeSettingsRepo::default()),
            pdf.clone(),
            storage.clone(),
        );
        (uc, pdf, storage)
    }

    #[test]
    fn cancel_draft_invoice_rejected() {
        let (inv_repo, tax_repo, tax) = setup();
        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo)
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        let (uc, _, _) = make_cancel(inv_repo);
        let err = uc.execute(created.id).unwrap_err();
        assert!(matches!(
            err,
            AppError::Invoice(crate::domain::invoice::InvoiceError::CannotCancelDraft)
        ));
    }

    #[test]
    fn cancel_finalized_invoice_updates_status() {
        let (inv_repo, tax_repo, tax) = setup();
        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo)
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        // Manually finalize via direct repo manipulation (bypassing FinalizeInvoice which needs PDF pipeline).
        {
            let mut g = inv_repo.inner.lock();
            let inv = g.get_mut(&created.id).unwrap();
            inv.finalize(crate::domain::invoice::InvoiceNumber(1), chrono::Utc::now())
                .unwrap();
        }
        let (uc, pdf_gen, storage) = make_cancel(inv_repo);
        let cancelled = uc.execute(created.id).unwrap();
        assert_eq!(cancelled.status, InvoiceStatus::Cancelled);
        // No pdf was rendered because the in-mem invoice had no pdf_path.
        assert_eq!(*pdf_gen.0.lock(), 0);
        assert!(storage.calls.lock().is_empty());
    }

    #[test]
    fn cancel_finalized_invoice_with_pdf_regenerates_with_watermark() {
        let (inv_repo, _tax_repo, tax) = setup();
        let client_repo_concrete = Arc::new(FakeClientRepo(Mutex::new(HashMap::new())));
        // Seed a client so resolving inside cancel succeeds.
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo_concrete.insert(&client).unwrap();

        let tmpl_repo = Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new())));
        let mut template = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Default".into(),
            ..Default::default()
        })
        .unwrap();
        template.is_default = true;
        tmpl_repo.insert(&template).unwrap();
        let pdf_gen = Arc::new(RenderCapturingPdfGenerator::default());
        let storage = Arc::new(CapturingPdfStorage::default());

        // Seed a finalized invoice with a pdf_path already set.
        let mut invoice = Invoice::create_draft(
            NewInvoice {
                client_id: client.id,
                ..new_invoice_input(client.id, tax.id)
            },
            &[tax],
            Utc::now(),
        )
        .unwrap();
        invoice
            .finalize(crate::domain::invoice::InvoiceNumber(1001), Utc::now())
            .unwrap();
        invoice.set_pdf_path("/tmp/invoice-1001.pdf".into());
        inv_repo.inner.lock().insert(invoice.id, invoice.clone());

        let uc = CancelInvoice::new(
            inv_repo.clone(),
            client_repo_concrete,
            tmpl_repo,
            Arc::new(FakeSettingsRepo::default()),
            pdf_gen.clone(),
            storage.clone(),
        );
        uc.execute(invoice.id).unwrap();

        // Fake settings returns the default (Fr), so the localized watermark is "ANNULÉ".
        let observed = pdf_gen.last_watermark.lock();
        assert_eq!(
            observed.as_deref(),
            Some("ANNULÉ"),
            "cancel must render with the localized cancelled watermark"
        );
        let calls = storage.calls.lock();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "invoice-1001.pdf");

        let reloaded = inv_repo.inner.lock().get(&invoice.id).cloned().unwrap();
        assert_eq!(reloaded.status, InvoiceStatus::Cancelled);
        assert!(reloaded.pdf_path.is_some());
    }

    #[test]
    fn cancel_uses_english_watermark_when_language_is_en() {
        let (inv_repo, _tax_repo, tax) = setup();
        let client_repo_concrete = Arc::new(FakeClientRepo(Mutex::new(HashMap::new())));
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo_concrete.insert(&client).unwrap();

        let tmpl_repo = Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new())));
        let mut template = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Default".into(),
            ..Default::default()
        })
        .unwrap();
        template.is_default = true;
        tmpl_repo.insert(&template).unwrap();
        let pdf_gen = Arc::new(RenderCapturingPdfGenerator::default());
        let storage = Arc::new(CapturingPdfStorage::default());

        let mut invoice = Invoice::create_draft(
            NewInvoice {
                client_id: client.id,
                ..new_invoice_input(client.id, tax.id)
            },
            &[tax],
            Utc::now(),
        )
        .unwrap();
        invoice
            .finalize(crate::domain::invoice::InvoiceNumber(2002), Utc::now())
            .unwrap();
        invoice.set_pdf_path("/tmp/invoice-2002.pdf".into());
        inv_repo.inner.lock().insert(invoice.id, invoice.clone());

        let uc = CancelInvoice::new(
            inv_repo,
            client_repo_concrete,
            tmpl_repo,
            Arc::new(FakeSettingsRepo::with_language(
                crate::domain::settings::Language::En,
            )),
            pdf_gen.clone(),
            storage,
        );
        uc.execute(invoice.id).unwrap();

        let observed = pdf_gen.last_watermark.lock();
        assert_eq!(observed.as_deref(), Some("CANCELLED"));
    }

    #[test]
    fn duplicate_invoice_creates_fresh_draft() {
        let (inv_repo, tax_repo, tax) = setup();
        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo)
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        // Finalize the source so we prove duplicate drops the number + status.
        {
            let mut g = inv_repo.inner.lock();
            let inv = g.get_mut(&created.id).unwrap();
            inv.finalize(crate::domain::invoice::InvoiceNumber(77), chrono::Utc::now())
                .unwrap();
        }
        let copy = DuplicateInvoice::new(inv_repo.clone()).execute(created.id).unwrap();
        assert_ne!(copy.id, created.id);
        assert_eq!(copy.status, InvoiceStatus::Draft);
        assert!(copy.number.is_none());
        assert!(copy.pdf_path.is_none());
        assert_eq!(copy.line_items.len(), created.line_items.len());
        assert_eq!(copy.total, created.total);
        assert_eq!(inv_repo.inner.lock().len(), 2);
    }

    #[test]
    fn list_invoices_filters_by_status() {
        let (inv_repo, tax_repo, tax) = setup();
        let create = CreateDraftInvoice::new(inv_repo.clone(), tax_repo);
        let _a = create
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        let b = create
            .execute(new_invoice_input(ClientId::new(), tax.id))
            .unwrap();
        {
            let mut g = inv_repo.inner.lock();
            let inv = g.get_mut(&b.id).unwrap();
            inv.finalize(crate::domain::invoice::InvoiceNumber(1), chrono::Utc::now())
                .unwrap();
        }
        let drafts = ListInvoices::new(inv_repo.clone(), stub_payments())
            .execute(ListInvoicesQuery {
                status: Some(InvoiceStatus::Draft),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(drafts.len(), 1);
        let finalized = ListInvoices::new(inv_repo, stub_payments())
            .execute(ListInvoicesQuery {
                status: Some(InvoiceStatus::Finalized),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(finalized.len(), 1);
    }

    #[test]
    fn get_invoice_returns_not_found() {
        let (inv_repo, _, _) = setup();
        let err = GetInvoice::new(inv_repo, stub_payments())
            .execute(InvoiceId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    // --- FinalizeInvoice pipeline test with fake ports ---

    use crate::application::ports::{
        ClientRepository, PdfError, PdfGenerator, PdfStorage, SettingsRepository,
        TemplateRepository,
    };
    use crate::domain::client::{Client, NewClient};
    use crate::domain::invoice::InvoiceNumber;
    use crate::domain::settings::{AppPreferences, CurrencyConfig, SellerProfile};
    use crate::domain::template::{InvoiceTemplate, NewInvoiceTemplate, TemplateId};

    struct FakeNumberGenerator(Mutex<u64>);
    impl InvoiceNumberGenerator for FakeNumberGenerator {
        fn next(&self) -> Result<InvoiceNumber, RepoError> {
            let mut g = self.0.lock();
            let n = *g;
            *g += 1;
            Ok(InvoiceNumber(n))
        }
    }

    struct FakeClientRepo(Mutex<HashMap<ClientId, Client>>);
    impl ClientRepository for FakeClientRepo {
        fn insert(&self, c: &Client) -> Result<(), RepoError> {
            self.0.lock().insert(c.id, c.clone());
            Ok(())
        }
        fn update(&self, c: &Client) -> Result<(), RepoError> {
            self.0.lock().insert(c.id, c.clone());
            Ok(())
        }
        fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError> {
            Ok(self.0.lock().get(&id).cloned())
        }
        fn list(
            &self,
            _q: crate::application::ports::ListClientsQuery,
        ) -> Result<Vec<Client>, RepoError> {
            Ok(vec![])
        }
    }

    struct FakeTemplateRepo(Mutex<HashMap<TemplateId, InvoiceTemplate>>);
    impl TemplateRepository for FakeTemplateRepo {
        fn insert(&self, t: &InvoiceTemplate) -> Result<(), RepoError> {
            self.0.lock().insert(t.id, t.clone());
            Ok(())
        }
        fn update(&self, _: &InvoiceTemplate) -> Result<(), RepoError> {
            Ok(())
        }
        fn get(&self, id: TemplateId) -> Result<Option<InvoiceTemplate>, RepoError> {
            Ok(self.0.lock().get(&id).cloned())
        }
        fn list(&self) -> Result<Vec<InvoiceTemplate>, RepoError> {
            Ok(vec![])
        }
        fn get_default(&self) -> Result<Option<InvoiceTemplate>, RepoError> {
            Ok(self.0.lock().values().find(|t| t.is_default).cloned())
        }
        fn set_default(&self, _: TemplateId) -> Result<(), RepoError> {
            Ok(())
        }
        fn is_used_by_invoice(&self, _: TemplateId) -> Result<bool, RepoError> {
            Ok(false)
        }
        fn delete(&self, _: TemplateId) -> Result<(), RepoError> {
            Ok(())
        }
    }

    struct FakeSettingsRepo {
        language: crate::domain::settings::Language,
    }
    impl Default for FakeSettingsRepo {
        fn default() -> Self {
            Self {
                language: crate::domain::settings::Language::default(),
            }
        }
    }
    impl FakeSettingsRepo {
        fn with_language(language: crate::domain::settings::Language) -> Self {
            Self { language }
        }
    }
    impl SettingsRepository for FakeSettingsRepo {
        fn get_seller_profile(&self) -> Result<SellerProfile, RepoError> {
            Ok(SellerProfile {
                name: "Me".into(),
                ..Default::default()
            })
        }
        fn set_seller_profile(&self, _: &SellerProfile) -> Result<(), RepoError> {
            Ok(())
        }
        fn get_currency_config(&self) -> Result<CurrencyConfig, RepoError> {
            Ok(CurrencyConfig::default())
        }
        fn set_currency_config(&self, _: &CurrencyConfig) -> Result<(), RepoError> {
            Ok(())
        }
        fn get_app_preferences(&self) -> Result<AppPreferences, RepoError> {
            Ok(AppPreferences {
                language: self.language,
                ..Default::default()
            })
        }
        fn set_app_preferences(&self, _: &AppPreferences) -> Result<(), RepoError> {
            Ok(())
        }
        fn get_email_config(
            &self,
        ) -> Result<crate::domain::settings::EmailConfig, RepoError> {
            Ok(crate::domain::settings::EmailConfig::default())
        }
        fn set_email_config(
            &self,
            _: &crate::domain::settings::EmailConfig,
        ) -> Result<(), RepoError> {
            Ok(())
        }
    }

    struct FakePdfGenerator(Mutex<u32>);
    impl PdfGenerator for FakePdfGenerator {
        fn render(
            &self,
            input: crate::application::ports::PdfRenderInput<'_>,
        ) -> Result<Vec<u8>, PdfError> {
            *self.0.lock() += 1;
            assert!(!input.is_preview, "finalize must render non-preview");
            assert!(input.watermark.is_none(), "finalize must not watermark");
            assert!(input.invoice.number.is_some(), "finalize assigns number before render");
            Ok(b"%PDF-fake".to_vec())
        }
    }

    #[derive(Default)]
    struct RenderCapturingPdfGenerator {
        last_watermark: Mutex<Option<String>>,
        calls: Mutex<u32>,
    }
    impl PdfGenerator for RenderCapturingPdfGenerator {
        fn render(
            &self,
            input: crate::application::ports::PdfRenderInput<'_>,
        ) -> Result<Vec<u8>, PdfError> {
            *self.calls.lock() += 1;
            *self.last_watermark.lock() = input.watermark.map(|s| s.to_string());
            Ok(b"%PDF-cancelled".to_vec())
        }
    }

    #[derive(Default)]
    struct CapturingPdfStorage {
        calls: Mutex<Vec<(String, Vec<u8>)>>,
    }
    impl PdfStorage for CapturingPdfStorage {
        fn store(&self, file_name: &str, bytes: &[u8]) -> Result<String, RepoError> {
            self.calls
                .lock()
                .push((file_name.to_string(), bytes.to_vec()));
            Ok(format!("/tmp/{file_name}"))
        }
    }

    #[test]
    fn finalize_assigns_number_renders_pdf_and_stores_path() {
        let (inv_repo, tax_repo, tax) = setup();
        let client_repo = Arc::new(FakeClientRepo(Mutex::new(HashMap::new())));
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo.insert(&client).unwrap();

        let tmpl_repo = Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new())));
        let mut template = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Default".into(),
            ..Default::default()
        })
        .unwrap();
        template.is_default = true;
        tmpl_repo.insert(&template).unwrap();

        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo.clone())
            .execute(NewInvoice {
                client_id: client.id,
                ..new_invoice_input(client.id, tax.id)
            })
            .unwrap();

        let numbers = Arc::new(FakeNumberGenerator(Mutex::new(1001)));
        let pdf = Arc::new(FakePdfGenerator(Mutex::new(0)));
        let storage = Arc::new(CapturingPdfStorage::default());
        let finalize = FinalizeInvoice::new(
            inv_repo.clone(),
            numbers,
            tmpl_repo,
            Arc::new(FakeSettingsRepo::default()),
            client_repo,
            pdf.clone(),
            storage.clone(),
        );

        let finalized = finalize.execute(created.id).unwrap();

        assert_eq!(finalized.status, InvoiceStatus::Finalized);
        assert_eq!(finalized.number, Some(InvoiceNumber(1001)));
        assert_eq!(finalized.pdf_path.as_deref(), Some("/tmp/invoice-1001.pdf"));
        assert_eq!(*pdf.0.lock(), 1, "pdf render must be called exactly once");
        let calls = storage.calls.lock();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "invoice-1001.pdf");
        assert_eq!(calls[0].1, b"%PDF-fake");

        // Persisted state reflects the finalize.
        let reloaded = inv_repo.inner.lock().get(&created.id).cloned().unwrap();
        assert_eq!(reloaded.status, InvoiceStatus::Finalized);
        assert_eq!(
            reloaded.pdf_path.as_deref(),
            Some("/tmp/invoice-1001.pdf")
        );
    }

    #[test]
    fn finalize_falls_back_to_implicit_template_when_none_configured() {
        let (inv_repo, tax_repo, tax) = setup();
        let client_repo = Arc::new(FakeClientRepo(Mutex::new(HashMap::new())));
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo.insert(&client).unwrap();

        // Template repo is empty — no default configured. Finalize must still succeed.
        let tmpl_repo = Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new())));

        let created = CreateDraftInvoice::new(inv_repo.clone(), tax_repo.clone())
            .execute(NewInvoice {
                client_id: client.id,
                ..new_invoice_input(client.id, tax.id)
            })
            .unwrap();

        let finalize = FinalizeInvoice::new(
            inv_repo.clone(),
            Arc::new(FakeNumberGenerator(Mutex::new(1))),
            tmpl_repo,
            Arc::new(FakeSettingsRepo::default()),
            client_repo,
            Arc::new(FakePdfGenerator(Mutex::new(0))),
            Arc::new(CapturingPdfStorage::default()),
        );
        let finalized = finalize.execute(created.id).unwrap();
        assert_eq!(finalized.status, InvoiceStatus::Finalized);
    }

    #[test]
    fn finalize_rejects_missing_invoice() {
        let (inv_repo, _, _) = setup();
        let finalize = FinalizeInvoice::new(
            inv_repo,
            Arc::new(FakeNumberGenerator(Mutex::new(1))),
            Arc::new(FakeTemplateRepo(Mutex::new(HashMap::new()))),
            Arc::new(FakeSettingsRepo::default()),
            Arc::new(FakeClientRepo(Mutex::new(HashMap::new()))),
            Arc::new(FakePdfGenerator(Mutex::new(0))),
            Arc::new(CapturingPdfStorage::default()),
        );
        let err = finalize.execute(InvoiceId::new()).unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }
}
