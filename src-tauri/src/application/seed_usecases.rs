//! Bulk fake-data seeder for development. Manually triggered from the
//! Settings → Developer section in the UI; never compiled into release
//! builds (the entire module is `#[cfg(debug_assertions)]`-gated by its
//! `pub mod` line in `application/mod.rs`).
//!
//! ## Maintenance rule
//!
//! When you add a new entity to the system, add a `seed_<entity>(...)`
//! helper here, call it from `SeedDatabase::execute`, and add a count
//! field to `SeedCounts` and `SeedReport`. The orchestrator threads
//! everything through use cases (not raw SQL) so the compiler catches
//! drift: when a domain field is added, the use case input struct
//! changes, and this file fails to compile until updated.
//!
//! ## Why use cases (not raw SQL)
//!
//! Each `seed_<entity>` calls the same `Create<Entity>` use case the UI
//! does. Validation, normalization, and side effects (auto-default flags
//! on contacts, sort_order assignment on bookmarks, etc.) match real-app
//! behaviour. The cost — Typst PDF generation when finalizing invoices —
//! is mitigated by finalizing only ~25% of seeded invoices.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use fake::faker::address::fr_fr::{CityName, StreetName, ZipCode};
use fake::faker::company::fr_fr::CompanyName;
use fake::faker::internet::en::FreeEmail;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::fr_fr::{FirstName, LastName, Name};
use fake::faker::phone_number::fr_fr::PhoneNumber;
use fake::Fake;
use rand::seq::SliceRandom;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use std::sync::Arc;

use crate::application::bookmark_usecases::{CreateBookmark, CreateBookmarkInput};
use crate::application::catalog_item_usecases::CreateCatalogItem;
use crate::application::client_usecases::CreateClient;
use crate::application::invoice_usecases::{CancelInvoice, CreateDraftInvoice, FinalizeInvoice};
use crate::application::notebook_usecases::CreateJournalEntry;
use crate::application::payment_usecases::RecordPayment;
use crate::application::ports::{ClientRepository, EmailLogRepository, InvoiceRepository};
use crate::application::tax_usecases::CreateTax;
use crate::application::AppError;
use crate::domain::catalog_item::{CatalogItemKind, NewCatalogItem};
use crate::domain::client::{
    ClientId, ClientKind, NewClient, NewClientAddress, NewContactEntry,
};
use crate::domain::email_log::{EmailLog, NewEmailLog};
use crate::domain::email_template::EmailTemplateType;
use crate::domain::invoice::{InvoiceId, NewInvoice};
use crate::domain::line_item::NewLineItem;
use crate::domain::money::{Currency, Money};
use crate::domain::notebook::NewJournalEntry;
use crate::domain::payment::{NewPayment, NewPaymentAllocation, PaymentMethod};
use crate::domain::tax::{NewTaxDefinition, TaxId};

/// Number of entities to create per run. Each field is a *target* —
/// individual seed helpers may produce slightly fewer if they depend on
/// data that wasn't seeded (e.g. payments need finalized invoices).
#[derive(Debug, Clone)]
pub struct SeedCounts {
    pub clients: u32,
    pub catalog_items: u32,
    pub taxes: u32,
    pub invoices: u32,
    pub bookmarks: u32,
    pub journal_entries_per_client: u32,
}

impl Default for SeedCounts {
    fn default() -> Self {
        Self {
            clients: 20,
            catalog_items: 10,
            taxes: 2,
            invoices: 50,
            bookmarks: 3,
            journal_entries_per_client: 2,
        }
    }
}

/// Tally of what was actually inserted. Returned to the UI for display.
#[derive(Debug, Clone, Default)]
pub struct SeedReport {
    pub clients_added: u32,
    pub catalog_items_added: u32,
    pub taxes_added: u32,
    pub invoices_drafted: u32,
    pub invoices_finalized: u32,
    pub invoices_cancelled: u32,
    pub invoices_sent: u32,
    pub payments_added: u32,
    pub emails_added: u32,
    pub bookmarks_added: u32,
    pub journal_entries_added: u32,
}

#[derive(Clone)]
pub struct SeedDatabase {
    create_client: CreateClient,
    create_catalog_item: CreateCatalogItem,
    create_tax: CreateTax,
    create_bookmark: CreateBookmark,
    create_draft_invoice: CreateDraftInvoice,
    finalize_invoice: FinalizeInvoice,
    cancel_invoice: CancelInvoice,
    record_payment: RecordPayment,
    create_journal_entry: CreateJournalEntry,
    // The "send invoice email" path is the one place we deviate from the
    // use-case-only rule — `SendInvoice` would actually call SMTP, which we
    // can't do in a seed run. We bypass the use-case and write directly via
    // these repos: `Invoice::mark_sent` enforces the lifecycle invariant,
    // `EmailLog::record` validates the row, and the inserts mirror what the
    // production `SendInvoice` use-case does after the network call returns.
    invoices: Arc<dyn InvoiceRepository>,
    clients: Arc<dyn ClientRepository>,
    email_logs: Arc<dyn EmailLogRepository>,
}

impl SeedDatabase {
    pub fn new(
        create_client: CreateClient,
        create_catalog_item: CreateCatalogItem,
        create_tax: CreateTax,
        create_bookmark: CreateBookmark,
        create_draft_invoice: CreateDraftInvoice,
        finalize_invoice: FinalizeInvoice,
        cancel_invoice: CancelInvoice,
        record_payment: RecordPayment,
        create_journal_entry: CreateJournalEntry,
        invoices: Arc<dyn InvoiceRepository>,
        clients: Arc<dyn ClientRepository>,
        email_logs: Arc<dyn EmailLogRepository>,
    ) -> Self {
        Self {
            create_client,
            create_catalog_item,
            create_tax,
            create_bookmark,
            create_draft_invoice,
            finalize_invoice,
            cancel_invoice,
            record_payment,
            create_journal_entry,
            invoices,
            clients,
            email_logs,
        }
    }

    pub fn execute(&self, counts: SeedCounts) -> Result<SeedReport, AppError> {
        let mut report = SeedReport::default();
        let mut rng = rand::thread_rng();

        // Pick a per-run pool of 2–10 currencies from the supported set.
        // Every downstream seed step draws from this pool so the produced
        // data exercises the multi-currency UI (stacked dashboard rows,
        // per-currency Accounting tabs, catalog items with prices in
        // several currencies). Strict silos: every invoice/payment chain
        // is single-currency, but the dataset as a whole spans the pool.
        let pool: Vec<Currency> = {
            let mut all: Vec<Currency> = Currency::all().to_vec();
            all.shuffle(&mut rng);
            let n = rng.gen_range(2..=10).min(all.len());
            all.into_iter().take(n).collect()
        };

        // FK order: clients first, then catalog/tax/bookmark independents,
        // then invoices (need clients + taxes), then payments (need
        // invoices), then journal entries (need clients).
        let client_currencies =
            self.seed_clients(counts.clients, &pool, &mut report)?;
        self.seed_catalog_items(counts.catalog_items, &pool, &mut report)?;
        let tax_ids = self.seed_taxes(counts.taxes, &mut report)?;
        let finalized = self.seed_invoices(
            counts.invoices,
            &client_currencies,
            &tax_ids,
            &mut report,
        )?;
        self.seed_payments(&finalized, &mut report)?;
        // Emails depend on finalized invoices (they get marked Sent).
        // Run after payments so paid + sent combinations exist in the data.
        self.seed_emails(&finalized, &mut report)?;
        self.seed_bookmarks(counts.bookmarks, &mut report)?;
        let client_ids: Vec<ClientId> =
            client_currencies.iter().map(|(id, _)| *id).collect();
        self.seed_journal_entries(
            counts.journal_entries_per_client,
            &client_ids,
            &mut report,
        )?;

        Ok(report)
    }

    fn seed_clients(
        &self,
        n: u32,
        currency_pool: &[Currency],
        report: &mut SeedReport,
    ) -> Result<Vec<(ClientId, Currency)>, AppError> {
        let mut rng = rand::thread_rng();
        let mut ids: Vec<(ClientId, Currency)> = Vec::with_capacity(n as usize);
        let pronouns_pool = ["she/her", "he/him", "they/them"];
        let sex_pool = ["female", "male"];
        let gender_pool = ["femme", "homme", "non-binaire"];
        let language_pool = ["fr", "en", "nl"];
        let occupation_pool = [
            "Architecte",
            "Médecin",
            "Avocate",
            "Enseignant",
            "Photographe",
            "Consultante",
            "Designer",
            "Ingénieur logiciel",
            "Comptable",
            "Cuisinière",
        ];

        for _ in 0..n {
            let phone_value: String = PhoneNumber().fake();
            let default_currency = *currency_pool.choose(&mut rng).unwrap();
            let address = NewClientAddress {
                id: None,
                label: Some("Principal".into()),
                street: format!(
                    "{} {}",
                    rng.gen_range(1..200),
                    StreetName().fake::<String>(),
                ),
                apt_suite: None,
                city: CityName().fake::<String>(),
                state_province: None,
                postal_code: ZipCode().fake::<String>(),
                country: "FR".into(),
                is_billing: true,
                is_shipping: true,
            };

            // ~30% Companies, ~70% Individuals.
            let is_company = rng.gen_bool(0.3);
            let new_client = if is_company {
                let company_name: String = CompanyName().fake();
                let contact_first: String = FirstName().fake();
                let contact_last: String = LastName().fake();
                let billing_email: String = FreeEmail().fake();
                NewClient {
                    kind: ClientKind::Company,
                    name: company_name,
                    contact_name: Some(format!("{contact_first} {contact_last}")),
                    tax_id: Some(format!(
                        "FR{:011}",
                        rng.gen_range::<u64, _>(10_000_000_000..99_999_999_999),
                    )),
                    registration_number: Some(format!(
                        "{} {} {}",
                        rng.gen_range(100..1000),
                        rng.gen_range(100..1000),
                        rng.gen_range(100..1000),
                    )),
                    emails: vec![NewContactEntry {
                        id: None,
                        value: billing_email,
                        label: Some("Facturation".into()),
                        is_default: true,
                    }],
                    phones: vec![NewContactEntry {
                        id: None,
                        value: phone_value,
                        label: Some("Standard".into()),
                        is_default: true,
                    }],
                    addresses: vec![address],
                    notes: None,
                    referred_by: None,
                    // Person-only fields stay None for Companies.
                    date_of_birth: None,
                    sex: None,
                    gender: None,
                    pronouns: None,
                    occupation: None,
                    language: Some(language_pool.choose(&mut rng).unwrap().to_string()),
                    default_currency,
                }
            } else {
                let first: String = FirstName().fake();
                let last: String = LastName().fake();
                let email_value: String = FreeEmail().fake();
                NewClient {
                    kind: ClientKind::Individual,
                    name: format!("{first} {last}"),
                    contact_name: None,
                    tax_id: None,
                    registration_number: None,
                    emails: vec![NewContactEntry {
                        id: None,
                        value: email_value,
                        label: Some("Personnel".into()),
                        is_default: true,
                    }],
                    phones: vec![NewContactEntry {
                        id: None,
                        value: phone_value,
                        label: Some("Mobile".into()),
                        is_default: true,
                    }],
                    addresses: vec![address],
                    notes: None,
                    referred_by: None,
                    date_of_birth: Some(random_dob(&mut rng)),
                    sex: Some(sex_pool.choose(&mut rng).unwrap().to_string()),
                    gender: Some(gender_pool.choose(&mut rng).unwrap().to_string()),
                    pronouns: Some(pronouns_pool.choose(&mut rng).unwrap().to_string()),
                    occupation: Some(occupation_pool.choose(&mut rng).unwrap().to_string()),
                    language: Some(language_pool.choose(&mut rng).unwrap().to_string()),
                    default_currency,
                }
            };

            let client = self.create_client.execute(new_client)?;
            ids.push((client.id, default_currency));
        }
        report.clients_added = n;
        Ok(ids)
    }

    fn seed_catalog_items(
        &self,
        n: u32,
        currency_pool: &[Currency],
        report: &mut SeedReport,
    ) -> Result<(), AppError> {
        let mut rng = rand::thread_rng();
        let products = [
            ("Livre", "piece", 2500_i64),
            ("Carnet", "piece", 1500),
            ("Stylo plume", "piece", 4500),
            ("Marqueur", "piece", 350),
            ("Lampe de bureau", "piece", 8900),
        ];
        let services = [
            ("Consultation", "heure", 15000_i64),
            ("Coaching", "session", 20000),
            ("Formation", "jour", 80000),
            ("Audit", "forfait", 120000),
            ("Suivi mensuel", "mois", 50000),
        ];

        for i in 0..n {
            let pick_service = i % 3 != 0; // ~67% services, ~33% products
            let (name, unit, price) = if pick_service {
                services.choose(&mut rng).unwrap()
            } else {
                products.choose(&mut rng).unwrap()
            };
            let kind = if pick_service {
                CatalogItemKind::Service
            } else {
                CatalogItemKind::Product
            };
            let suffix: u32 = rng.gen_range(100..1000);
            // Pricing variety:
            //  - ~10% of items have no prices (exercises the manual-entry
            //    path in the invoice editor's catalog picker).
            //  - The rest get 1..=min(3, pool_size) prices in distinct
            //    currencies drawn from the pool. The base item template's
            //    `price` is used for the first currency; additional ones
            //    nudge ±25% so values look plausible per-currency without
            //    being a real conversion.
            let prices: Vec<Money> = if rng.gen_bool(0.1) || currency_pool.is_empty() {
                Vec::new()
            } else {
                let mut shuffled = currency_pool.to_vec();
                shuffled.shuffle(&mut rng);
                let count = rng.gen_range(1..=shuffled.len().min(3));
                shuffled
                    .into_iter()
                    .take(count)
                    .enumerate()
                    .map(|(idx, c)| {
                        let nudged = if idx == 0 {
                            *price
                        } else {
                            let factor: f64 = rng.gen_range(0.75..=1.25);
                            ((*price as f64) * factor).round() as i64
                        };
                        Money::new(nudged, c)
                    })
                    .collect()
            };
            self.create_catalog_item.execute(NewCatalogItem {
                name: format!("{name} {suffix}"),
                kind,
                prices,
                unit: Some((*unit).into()),
                reference: Some(format!("REF-{suffix}")),
            })?;
        }
        report.catalog_items_added = n;
        Ok(())
    }

    fn seed_taxes(
        &self,
        n: u32,
        report: &mut SeedReport,
    ) -> Result<Vec<TaxId>, AppError> {
        // Reasonable picks. Indexing avoids dup names if n>list len.
        let candidates = [
            ("Tax 1 21%", dec!(21.0)),
            ("Tax 2 6%", dec!(6.0)),
        ];
        let mut ids = Vec::with_capacity(n as usize);
        for i in 0..n {
            let (name, pct) = candidates[(i as usize) % candidates.len()];
            let tax = self.create_tax.execute(NewTaxDefinition {
                name: format!("{name} (seed-{i})"),
                percentage: pct,
                tax_id_number: None,
            })?;
            ids.push(tax.id);
        }
        report.taxes_added = n;
        Ok(ids)
    }

    /// Returns finalized invoices as `(client_id, invoice_id, total)` so
    /// `seed_payments` can size allocations to fit the actual invoice
    /// totals (and use the right `client_id`, since payments are
    /// per-client-not-per-invoice in the domain model).
    fn seed_invoices(
        &self,
        n: u32,
        client_currencies: &[(ClientId, Currency)],
        tax_ids: &[TaxId],
        report: &mut SeedReport,
    ) -> Result<Vec<(ClientId, InvoiceId, Money)>, AppError> {
        if client_currencies.is_empty() {
            return Ok(Vec::new());
        }
        let mut rng = rand::thread_rng();
        let mut finalized: Vec<(ClientId, InvoiceId, Money)> = Vec::new();
        let today = Utc::now().date_naive();

        for _ in 0..n {
            let &(client_id, currency) = client_currencies.choose(&mut rng).unwrap();
            // Issue date: random within the last 6 months.
            let days_ago: i64 = rng.gen_range(0..180);
            let date = today - Duration::days(days_ago);
            let due_date = date + Duration::days(rng.gen_range(15..45));

            let line_count = rng.gen_range(1..5);
            let line_items: Vec<NewLineItem> = (0..line_count)
                .map(|_| NewLineItem {
                    id: None,
                    catalog_item_id: None,
                    description: Sentence(3..8).fake(),
                    quantity: Decimal::from(rng.gen_range(1..6)),
                    unit_price: Money::new(rng.gen_range(2500..50_000), currency),
                })
                .collect();

            // Apply 0–2 random taxes.
            let tax_count = if tax_ids.is_empty() {
                0
            } else {
                rng.gen_range(0..=tax_ids.len().min(2))
            };
            let mut applied_taxes = tax_ids.to_vec();
            applied_taxes.shuffle(&mut rng);
            applied_taxes.truncate(tax_count);

            let notes = if rng.gen_bool(0.4) {
                Some::<String>(Sentence(5..12).fake())
            } else {
                None
            };

            let draft = self.create_draft_invoice.execute(NewInvoice {
                client_id,
                template_id: None,
                date,
                due_date: Some(due_date),
                line_items,
                tax_ids: applied_taxes,
                notes,
                currency,
            })?;
            report.invoices_drafted += 1;

            // Status mix: ~25% finalize, ~5% finalize-then-cancel, rest stay draft.
            let roll: f64 = rng.gen();
            if roll < 0.25 {
                self.finalize_invoice.execute(draft.id)?;
                report.invoices_finalized += 1;
                finalized.push((client_id, draft.id, draft.total));
            } else if roll < 0.30 {
                self.finalize_invoice.execute(draft.id)?;
                self.cancel_invoice.execute(draft.id)?;
                report.invoices_finalized += 1;
                report.invoices_cancelled += 1;
            }
        }
        Ok(finalized)
    }

    /// Records payments against ~60% of the finalized invoices we just
    /// created. Roughly half full, half partial. Sizes are derived from
    /// each invoice's actual `total` so allocations never exceed the
    /// remaining balance (the domain rejects overallocation with
    /// `InvoiceError::OverAllocated`).
    fn seed_payments(
        &self,
        finalized: &[(ClientId, InvoiceId, Money)],
        report: &mut SeedReport,
    ) -> Result<(), AppError> {
        if finalized.is_empty() {
            return Ok(());
        }
        let mut rng = rand::thread_rng();
        let methods = [
            PaymentMethod::BankTransfer,
            PaymentMethod::Cash,
            PaymentMethod::Check,
            PaymentMethod::Card,
        ];
        let today = Utc::now().date_naive();

        for (client_id, invoice_id, total) in finalized {
            if !rng.gen_bool(0.6) {
                continue;
            }
            // Pay full ~50% of the time, partial (30%–90%) the rest.
            let amount_cents = if rng.gen_bool(0.5) {
                total.minor_units()
            } else {
                let factor: f64 = rng.gen_range(0.3..=0.9);
                let computed = ((total.minor_units() as f64) * factor) as i64;
                computed.max(1).min(total.minor_units())
            };
            // Strict-silos: payment currency must equal the invoice's.
            let amount = Money::new(amount_cents, total.currency());

            self.record_payment.execute(NewPayment {
                client_id: *client_id,
                date: today - Duration::days(rng.gen_range(0..30)),
                amount,
                method: methods.choose(&mut rng).unwrap().clone(),
                reference: None,
                allocations: vec![NewPaymentAllocation {
                    invoice_id: *invoice_id,
                    amount,
                }],
                notes: None,
            })?;
            report.payments_added += 1;
        }
        Ok(())
    }

    /// "Sends" emails for ~70% of finalized invoices. Skips the SMTP
    /// transport (we can't and shouldn't talk to a mail server during a
    /// seed run); instead, mirrors what `SendInvoice` does *after* a
    /// successful send: flips the invoice to `Sent` and writes an
    /// `email_logs` row. ~30% of those also get a follow-up reminder log
    /// so the per-invoice history shows both InitialContact and FollowUp.
    fn seed_emails(
        &self,
        finalized: &[(ClientId, InvoiceId, Money)],
        report: &mut SeedReport,
    ) -> Result<(), AppError> {
        if finalized.is_empty() {
            return Ok(());
        }
        let mut rng = rand::thread_rng();
        let now = Utc::now();

        for (client_id, invoice_id, _) in finalized {
            if !rng.gen_bool(0.7) {
                continue;
            }
            // Look up the client's default email and the invoice itself.
            // Both must exist (they were just seeded above) — anything else
            // is a programming error worth surfacing.
            let Some(client) = self.clients.get(*client_id)? else {
                continue;
            };
            let Some(to_address) = client.default_email().map(str::to_owned) else {
                continue;
            };
            let Some(mut invoice) = self.invoices.get(*invoice_id)? else {
                continue;
            };
            // Cancelled invoices won't satisfy `mark_sent`; skip them.
            if invoice.mark_sent(now).is_err() {
                continue;
            }
            self.invoices.update(&invoice)?;
            report.invoices_sent += 1;

            // Initial send — backdated to within a few days of issue date.
            let initial_sent_at = now - Duration::days(rng.gen_range(0..30));
            let initial = EmailLog::record(NewEmailLog {
                client_id: *client_id,
                invoice_id: Some(*invoice_id),
                template_type: Some(EmailTemplateType::InitialContact),
                template_name: Some("Default".into()),
                to_address: to_address.clone(),
                subject: format!(
                    "Invoice {} from {}",
                    invoice.number.map(|n| n.0.to_string()).unwrap_or_default(),
                    client.name,
                ),
                sent_at: initial_sent_at,
            })?;
            self.email_logs.insert(&initial)?;
            report.emails_added += 1;

            // ~30% chance of a follow-up reminder, dated 7-21 days later.
            if rng.gen_bool(0.3) {
                let follow_up_sent_at =
                    initial_sent_at + Duration::days(rng.gen_range(7..21));
                let follow_up = EmailLog::record(NewEmailLog {
                    client_id: *client_id,
                    invoice_id: Some(*invoice_id),
                    template_type: Some(EmailTemplateType::FollowUp),
                    template_name: Some("Default reminder".into()),
                    to_address,
                    subject: format!(
                        "Reminder: Invoice {}",
                        invoice.number.map(|n| n.0.to_string()).unwrap_or_default(),
                    ),
                    sent_at: follow_up_sent_at,
                })?;
                self.email_logs.insert(&follow_up)?;
                report.emails_added += 1;
            }
        }
        Ok(())
    }

    fn seed_bookmarks(&self, n: u32, report: &mut SeedReport) -> Result<(), AppError> {
        let candidates = [
            ("Google", "https://google.com"),
            ("Wikipedia", "https://wikipedia.org"),
            ("GitHub", "https://github.com"),
            ("Stack Overflow", "https://stackoverflow.com"),
        ];
        let actual = (n as usize).min(candidates.len());
        for (label, url) in candidates.iter().take(actual) {
            self.create_bookmark.execute(CreateBookmarkInput {
                label: (*label).into(),
                url: (*url).into(),
            })?;
        }
        report.bookmarks_added = actual as u32;
        Ok(())
    }

    fn seed_journal_entries(
        &self,
        per_client: u32,
        client_ids: &[ClientId],
        report: &mut SeedReport,
    ) -> Result<(), AppError> {
        let mut rng = rand::thread_rng();
        let today = Utc::now().date_naive();
        for client_id in client_ids {
            for _ in 0..per_client {
                let days_ago: i64 = rng.gen_range(0..120);
                self.create_journal_entry.execute(NewJournalEntry {
                    client_id: *client_id,
                    entry_date: today - Duration::days(days_ago),
                    content: Sentence(8..20).fake(),
                })?;
                report.journal_entries_added += 1;
            }
        }
        Ok(())
    }
}

/// Pick a plausible adult date of birth (between ~22 and ~75 years ago).
fn random_dob<R: Rng>(rng: &mut R) -> NaiveDate {
    let years_ago: i32 = rng.gen_range(22..75);
    let month: u32 = rng.gen_range(1..=12);
    let day: u32 = rng.gen_range(1..=28);
    let target_year = Utc::now().year() - years_ago;
    NaiveDate::from_ymd_opt(target_year, month, day).unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(target_year, 1, 1).unwrap()
    })
}

/// Avoid an unused-import warning on releases that don't compile this file.
#[allow(dead_code)]
fn _suppress_unused() {
    let _ = CompanyName().fake::<String>();
    let _ = Name().fake::<String>();
}
