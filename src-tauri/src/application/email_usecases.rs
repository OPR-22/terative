use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;

use crate::application::ports::{
    ClientRepository, CredentialStore, EmailAttachment, EmailSender, InvoiceRepository,
    OutboundEmail, SettingsRepository,
};
use crate::application::AppError;
use crate::domain::invoice::{Invoice, InvoiceId, InvoiceStatus};
use crate::domain::settings::{CurrencyConfig, EmailConfig, SellerProfile};

pub struct UpdateEmailConfig {
    repo: Arc<dyn SettingsRepository>,
}

impl UpdateEmailConfig {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, config: EmailConfig) -> Result<EmailConfig, AppError> {
        // Allow saving an incomplete config (user may fill it in progressively),
        // but validate on send/test. We only trim here.
        let trimmed = EmailConfig {
            smtp_host: config.smtp_host.trim().to_string(),
            smtp_port: config.smtp_port,
            sender_address: config.sender_address.trim().to_string(),
            subject_template: config.subject_template,
            body_template: config.body_template,
        };
        self.repo.set_email_config(&trimmed)?;
        Ok(trimmed)
    }
}

pub struct UpdateEmailPassword {
    credentials: Arc<dyn CredentialStore>,
}

impl UpdateEmailPassword {
    pub fn new(credentials: Arc<dyn CredentialStore>) -> Self {
        Self { credentials }
    }
    pub fn execute(&self, password: &str) -> Result<(), AppError> {
        if password.is_empty() {
            self.credentials.delete_smtp_password()?;
        } else {
            self.credentials.set_smtp_password(password)?;
        }
        Ok(())
    }
}

pub struct TestEmailConnection {
    settings: Arc<dyn SettingsRepository>,
    credentials: Arc<dyn CredentialStore>,
    email: Arc<dyn EmailSender>,
}

impl TestEmailConnection {
    pub fn new(
        settings: Arc<dyn SettingsRepository>,
        credentials: Arc<dyn CredentialStore>,
        email: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            settings,
            credentials,
            email,
        }
    }
    pub fn execute(&self) -> Result<(), AppError> {
        let cfg = self.settings.get_email_config()?;
        cfg.validate()?;
        let password = self
            .credentials
            .get_smtp_password()?
            .ok_or(AppError::MissingSmtpPassword)?;
        self.email
            .test_connection(&cfg.smtp_host, cfg.smtp_port, &cfg.sender_address, &password)?;
        Ok(())
    }
}

pub struct SendInvoice {
    invoices: Arc<dyn InvoiceRepository>,
    clients: Arc<dyn ClientRepository>,
    settings: Arc<dyn SettingsRepository>,
    credentials: Arc<dyn CredentialStore>,
    email: Arc<dyn EmailSender>,
}

impl SendInvoice {
    pub fn new(
        invoices: Arc<dyn InvoiceRepository>,
        clients: Arc<dyn ClientRepository>,
        settings: Arc<dyn SettingsRepository>,
        credentials: Arc<dyn CredentialStore>,
        email: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            invoices,
            clients,
            settings,
            credentials,
            email,
        }
    }

    pub fn execute(&self, id: InvoiceId) -> Result<Invoice, AppError> {
        let mut invoice = self.invoices.get(id)?.ok_or(AppError::NotFound)?;
        if invoice.status != InvoiceStatus::Finalized {
            return Err(AppError::Invoice(
                crate::domain::invoice::InvoiceError::NotFinalized,
            ));
        }
        let pdf_path = invoice
            .pdf_path
            .clone()
            .ok_or(AppError::MissingInvoicePdf)?;
        let pdf_bytes = std::fs::read(&pdf_path)
            .map_err(|e| AppError::Repo(crate::application::RepoError::Storage(e.to_string())))?;

        let client = self
            .clients
            .get(invoice.client_id)?
            .ok_or(AppError::NotFound)?;
        let seller = self.settings.get_seller_profile()?;
        let currency = self.settings.get_currency_config()?;
        let cfg = self.settings.get_email_config()?;
        cfg.validate()?;

        let to_address = client
            .default_email()
            .map(str::to_owned)
            .ok_or_else(|| AppError::Email(crate::application::ports::EmailError::NotConfigured(
                "client has no email address".into(),
            )))?;

        let vars = build_placeholder_vars(&invoice, &client, &seller, &currency);
        let subject = cfg.render_subject(&vars);
        let body = cfg.render_body(&vars);

        let password = self
            .credentials
            .get_smtp_password()?
            .ok_or(AppError::MissingSmtpPassword)?;

        let file_name = format!(
            "invoice-{}.pdf",
            invoice
                .number
                .map(|n| n.0.to_string())
                .unwrap_or_else(|| invoice.id.to_string())
        );

        self.email.send(OutboundEmail {
            smtp_host: &cfg.smtp_host,
            smtp_port: cfg.smtp_port,
            smtp_user: &cfg.sender_address,
            smtp_password: &password,
            from_address: &cfg.sender_address,
            to_address: &to_address,
            subject: &subject,
            body: &body,
            attachment: Some(EmailAttachment {
                file_name: &file_name,
                content_type: "application/pdf",
                bytes: &pdf_bytes,
            }),
        })?;

        invoice.mark_sent(Utc::now())?;
        self.invoices.update(&invoice)?;
        Ok(invoice)
    }
}

pub(crate) fn build_placeholder_vars<'a>(
    invoice: &'a Invoice,
    client: &'a crate::domain::client::Client,
    seller: &'a SellerProfile,
    currency: &'a CurrencyConfig,
) -> HashMap<&'a str, String> {
    let mut vars = HashMap::new();
    vars.insert(
        "number",
        invoice
            .number
            .map(|n| n.0.to_string())
            .unwrap_or_else(|| "DRAFT".into()),
    );
    vars.insert("client_name", client.name.clone());
    vars.insert("date", invoice.date.format("%Y-%m-%d").to_string());
    vars.insert(
        "due_date",
        invoice
            .due_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
    );
    vars.insert("total", currency.format(invoice.total.amount_cents));
    vars.insert("subtotal", currency.format(invoice.subtotal.amount_cents));
    vars.insert("seller_name", seller.name.clone());
    vars.insert("currency_code", currency.code.clone());
    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{EmailError, ListInvoicesQuery};
    use crate::application::RepoError;
    use crate::domain::client::{Client, ClientId, NewClient};
    use crate::domain::invoice::{AppliedTax, InvoiceNumber};
    use crate::domain::line_item::{LineItem, LineItemId};
    use crate::domain::money::{Currency, Money};
    use crate::domain::settings::AppPreferences;
    use chrono::NaiveDate;
    use parking_lot::Mutex;
    use rust_decimal_macros::dec;
    use std::collections::HashMap as Map;

    // --- fakes ---

    #[derive(Default)]
    struct InMemoryInvoiceRepo(Mutex<Map<InvoiceId, Invoice>>);
    impl InvoiceRepository for InMemoryInvoiceRepo {
        fn insert(&self, i: &Invoice) -> Result<(), RepoError> {
            self.0.lock().insert(i.id, i.clone());
            Ok(())
        }
        fn update(&self, i: &Invoice) -> Result<(), RepoError> {
            self.0.lock().insert(i.id, i.clone());
            Ok(())
        }
        fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError> {
            Ok(self.0.lock().get(&id).cloned())
        }
        fn list(&self, _: ListInvoicesQuery) -> Result<Vec<Invoice>, RepoError> {
            Ok(self.0.lock().values().cloned().collect())
        }
        fn delete(&self, _: InvoiceId) -> Result<(), RepoError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemoryClientRepo(Mutex<Map<ClientId, Client>>);
    impl ClientRepository for InMemoryClientRepo {
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
            _: crate::application::ports::ListClientsQuery,
        ) -> Result<Vec<Client>, RepoError> {
            Ok(vec![])
        }
    }

    struct InMemorySettingsRepo {
        email: Mutex<EmailConfig>,
        seller: Mutex<SellerProfile>,
        currency: Mutex<CurrencyConfig>,
    }
    impl Default for InMemorySettingsRepo {
        fn default() -> Self {
            Self {
                email: Mutex::new(EmailConfig {
                    smtp_host: "smtp.example.com".into(),
                    smtp_port: 587,
                    sender_address: "me@example.com".into(),
                    subject_template: "Invoice {{number}} for {{client_name}}".into(),
                    body_template: "Hi {{client_name}},\n\nTotal: {{total}}\n— {{seller_name}}"
                        .into(),
                }),
                seller: Mutex::new(SellerProfile {
                    name: "Acme Freelance".into(),
                    ..Default::default()
                }),
                currency: Mutex::new(CurrencyConfig::default()),
            }
        }
    }
    impl SettingsRepository for InMemorySettingsRepo {
        fn get_seller_profile(&self) -> Result<SellerProfile, RepoError> {
            Ok(self.seller.lock().clone())
        }
        fn set_seller_profile(&self, p: &SellerProfile) -> Result<(), RepoError> {
            *self.seller.lock() = p.clone();
            Ok(())
        }
        fn get_currency_config(&self) -> Result<CurrencyConfig, RepoError> {
            Ok(self.currency.lock().clone())
        }
        fn set_currency_config(&self, c: &CurrencyConfig) -> Result<(), RepoError> {
            *self.currency.lock() = c.clone();
            Ok(())
        }
        fn get_app_preferences(&self) -> Result<AppPreferences, RepoError> {
            Ok(AppPreferences::default())
        }
        fn set_app_preferences(&self, _: &AppPreferences) -> Result<(), RepoError> {
            Ok(())
        }
        fn get_email_config(&self) -> Result<EmailConfig, RepoError> {
            Ok(self.email.lock().clone())
        }
        fn set_email_config(&self, c: &EmailConfig) -> Result<(), RepoError> {
            *self.email.lock() = c.clone();
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemoryCredentialStore(Mutex<Option<String>>);
    impl CredentialStore for InMemoryCredentialStore {
        fn set_smtp_password(&self, p: &str) -> Result<(), RepoError> {
            *self.0.lock() = Some(p.to_string());
            Ok(())
        }
        fn get_smtp_password(&self) -> Result<Option<String>, RepoError> {
            Ok(self.0.lock().clone())
        }
        fn has_smtp_password(&self) -> Result<bool, RepoError> {
            Ok(self.0.lock().is_some())
        }
        fn delete_smtp_password(&self) -> Result<(), RepoError> {
            *self.0.lock() = None;
            Ok(())
        }
    }

    #[derive(Default)]
    struct CapturingEmailSender {
        sent: Mutex<Vec<SentEmail>>,
        test_calls: Mutex<Vec<TestCall>>,
        fail_next_send: Mutex<bool>,
    }
    #[derive(Clone)]
    struct SentEmail {
        from: String,
        to: String,
        subject: String,
        body: String,
        attachment_name: Option<String>,
        attachment_bytes: Option<Vec<u8>>,
    }
    #[derive(Clone)]
    struct TestCall {
        host: String,
        port: u16,
    }
    impl EmailSender for CapturingEmailSender {
        fn send(&self, m: OutboundEmail<'_>) -> Result<(), EmailError> {
            if *self.fail_next_send.lock() {
                *self.fail_next_send.lock() = false;
                return Err(EmailError::Transport("forced failure".into()));
            }
            self.sent.lock().push(SentEmail {
                from: m.from_address.to_string(),
                to: m.to_address.to_string(),
                subject: m.subject.to_string(),
                body: m.body.to_string(),
                attachment_name: m.attachment.as_ref().map(|a| a.file_name.to_string()),
                attachment_bytes: m.attachment.as_ref().map(|a| a.bytes.to_vec()),
            });
            Ok(())
        }
        fn test_connection(
            &self,
            host: &str,
            port: u16,
            _user: &str,
            _password: &str,
        ) -> Result<(), EmailError> {
            self.test_calls.lock().push(TestCall {
                host: host.to_string(),
                port,
            });
            Ok(())
        }
    }

    // --- fixtures ---

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn make_finalized_invoice_with_pdf(
        client_id: ClientId,
        pdf_path: Option<String>,
    ) -> Invoice {
        let currency = eur();
        Invoice {
            id: InvoiceId::new(),
            number: Some(InvoiceNumber(1001)),
            client_id,
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
            line_items: vec![LineItem {
                id: LineItemId::new(),
                description: "Consulting".into(),
                quantity: dec!(1),
                unit_price: Money::new(100_000, currency),
                total: Money::new(100_000, currency),
            }],
            taxes_applied: vec![AppliedTax {
                tax_definition_id: None,
                tax_name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
                computed_amount: Money::new(21_000, currency),
            }],
            subtotal: Money::new(100_000, currency),
            tax_total: Money::new(21_000, currency),
            total: Money::new(121_000, currency),
            currency,
            status: InvoiceStatus::Finalized,
            pdf_path,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn seed_client_with_email(
        repo: &InMemoryClientRepo,
        email: Option<&str>,
    ) -> Client {
        use crate::domain::client::NewContactEntry;
        let emails = email
            .map(|e| {
                vec![NewContactEntry {
                    value: e.into(),
                    label: None,
                    is_default: true,
                }]
            })
            .unwrap_or_default();
        let client = Client::create(
            NewClient {
                name: "Acme Corp".into(),
                emails,
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&client).unwrap();
        client
    }

    // --- UpdateEmailConfig ---

    #[test]
    fn update_email_config_trims_host_and_sender() {
        let settings = Arc::new(InMemorySettingsRepo::default());
        let uc = UpdateEmailConfig::new(settings.clone());
        let saved = uc
            .execute(EmailConfig {
                smtp_host: "  smtp.new.com  ".into(),
                smtp_port: 465,
                sender_address: "  new@example.com  ".into(),
                subject_template: "Hi".into(),
                body_template: "Body".into(),
            })
            .unwrap();
        assert_eq!(saved.smtp_host, "smtp.new.com");
        assert_eq!(saved.sender_address, "new@example.com");
        assert_eq!(saved.smtp_port, 465);
    }

    // --- UpdateEmailPassword ---

    #[test]
    fn update_email_password_stores_and_deletes() {
        let creds = Arc::new(InMemoryCredentialStore::default());
        let uc = UpdateEmailPassword::new(creds.clone());
        uc.execute("hunter2").unwrap();
        assert_eq!(creds.get_smtp_password().unwrap().as_deref(), Some("hunter2"));
        uc.execute("").unwrap();
        assert_eq!(creds.get_smtp_password().unwrap(), None);
    }

    // --- TestEmailConnection ---

    #[test]
    fn test_email_connection_forwards_config_and_password() {
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());
        TestEmailConnection::new(settings, creds, sender.clone())
            .execute()
            .unwrap();
        let calls = sender.test_calls.lock();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].host, "smtp.example.com");
        assert_eq!(calls[0].port, 587);
    }

    #[test]
    fn test_email_connection_rejects_missing_password() {
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        let sender = Arc::new(CapturingEmailSender::default());
        let err = TestEmailConnection::new(settings, creds, sender)
            .execute()
            .unwrap_err();
        assert!(matches!(err, AppError::MissingSmtpPassword));
    }

    #[test]
    fn test_email_connection_rejects_invalid_config() {
        let settings = Arc::new(InMemorySettingsRepo::default());
        settings
            .set_email_config(&EmailConfig {
                smtp_host: "".into(),
                smtp_port: 587,
                sender_address: "me@example.com".into(),
                subject_template: "".into(),
                body_template: "".into(),
            })
            .unwrap();
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());
        let err = TestEmailConnection::new(settings, creds, sender)
            .execute()
            .unwrap_err();
        assert!(matches!(err, AppError::EmailConfig(_)));
    }

    // --- SendInvoice ---

    #[test]
    fn send_invoice_renders_placeholders_and_attaches_pdf() {
        let tmp = tempfile::tempdir().unwrap();
        let pdf_path = tmp.path().join("invoice-1001.pdf");
        std::fs::write(&pdf_path, b"%PDF-1.4 fake").unwrap();

        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let clients = Arc::new(InMemoryClientRepo::default());
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());

        let client = seed_client_with_email(&clients, Some("billing@acme.example"));
        let invoice = make_finalized_invoice_with_pdf(
            client.id,
            Some(pdf_path.to_string_lossy().to_string()),
        );
        invoices.insert(&invoice).unwrap();

        let sent = SendInvoice::new(
            invoices.clone(),
            clients,
            settings,
            creds,
            sender.clone(),
        )
        .execute(invoice.id)
        .unwrap();

        assert_eq!(sent.status, InvoiceStatus::Sent);
        let stored = sender.sent.lock();
        assert_eq!(stored.len(), 1);
        let e = &stored[0];
        assert_eq!(e.from, "me@example.com");
        assert_eq!(e.to, "billing@acme.example");
        assert_eq!(e.subject, "Invoice 1001 for Acme Corp");
        assert!(e.body.contains("Acme Corp"));
        assert!(e.body.contains("Acme Freelance"));
        assert!(e.body.contains("1210.00 €"));
        assert_eq!(e.attachment_name.as_deref(), Some("invoice-1001.pdf"));
        assert_eq!(e.attachment_bytes.as_deref(), Some(b"%PDF-1.4 fake" as &[u8]));

        // Invoice persisted in Sent state.
        let reloaded = invoices.0.lock().get(&invoice.id).cloned().unwrap();
        assert_eq!(reloaded.status, InvoiceStatus::Sent);
    }

    #[test]
    fn send_invoice_rejects_non_finalized() {
        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let clients = Arc::new(InMemoryClientRepo::default());
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());

        let client = seed_client_with_email(&clients, Some("x@y.z"));
        let mut invoice = make_finalized_invoice_with_pdf(client.id, None);
        invoice.status = InvoiceStatus::Draft;
        invoices.insert(&invoice).unwrap();

        let err = SendInvoice::new(invoices, clients, settings, creds, sender)
            .execute(invoice.id)
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Invoice(crate::domain::invoice::InvoiceError::NotFinalized)
        ));
    }

    #[test]
    fn send_invoice_rejects_missing_pdf_path() {
        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let clients = Arc::new(InMemoryClientRepo::default());
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());

        let client = seed_client_with_email(&clients, Some("x@y.z"));
        let invoice = make_finalized_invoice_with_pdf(client.id, None);
        invoices.insert(&invoice).unwrap();

        let err = SendInvoice::new(invoices, clients, settings, creds, sender)
            .execute(invoice.id)
            .unwrap_err();
        assert!(matches!(err, AppError::MissingInvoicePdf));
    }

    #[test]
    fn send_invoice_rejects_client_without_email() {
        let tmp = tempfile::tempdir().unwrap();
        let pdf_path = tmp.path().join("invoice-1001.pdf");
        std::fs::write(&pdf_path, b"%PDF").unwrap();

        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let clients = Arc::new(InMemoryClientRepo::default());
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());

        let client = seed_client_with_email(&clients, None);
        let invoice = make_finalized_invoice_with_pdf(
            client.id,
            Some(pdf_path.to_string_lossy().to_string()),
        );
        invoices.insert(&invoice).unwrap();

        let err = SendInvoice::new(invoices, clients, settings, creds, sender)
            .execute(invoice.id)
            .unwrap_err();
        assert!(matches!(err, AppError::Email(EmailError::NotConfigured(_))));
    }

    #[test]
    fn send_invoice_does_not_mark_sent_when_transport_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let pdf_path = tmp.path().join("invoice-1001.pdf");
        std::fs::write(&pdf_path, b"%PDF").unwrap();

        let invoices = Arc::new(InMemoryInvoiceRepo::default());
        let clients = Arc::new(InMemoryClientRepo::default());
        let settings = Arc::new(InMemorySettingsRepo::default());
        let creds = Arc::new(InMemoryCredentialStore::default());
        creds.set_smtp_password("pw").unwrap();
        let sender = Arc::new(CapturingEmailSender::default());
        *sender.fail_next_send.lock() = true;

        let client = seed_client_with_email(&clients, Some("x@y.z"));
        let invoice = make_finalized_invoice_with_pdf(
            client.id,
            Some(pdf_path.to_string_lossy().to_string()),
        );
        invoices.insert(&invoice).unwrap();

        let err = SendInvoice::new(
            invoices.clone(),
            clients,
            settings,
            creds,
            sender,
        )
        .execute(invoice.id)
        .unwrap_err();
        assert!(matches!(err, AppError::Email(EmailError::Transport(_))));
        let reloaded = invoices.0.lock().get(&invoice.id).cloned().unwrap();
        assert_eq!(reloaded.status, InvoiceStatus::Finalized);
    }
}
