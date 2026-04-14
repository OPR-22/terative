use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::application::ports::{
    ClientRepository, PdfGenerator, PdfRenderInput, SettingsRepository, TemplateRepository,
};
use crate::application::AppError;
use crate::domain::client::{Client, ClientId, NewClient};
use crate::domain::invoice::{AppliedTax, Invoice, InvoiceId, InvoiceNumber, InvoiceStatus};
use crate::domain::line_item::{LineItem, LineItemId};
use crate::domain::money::{Currency, Money};
use crate::domain::template::{
    FontChoice, InvoiceTemplate, NewInvoiceTemplate, TemplateId, TemplateLayout,
};
use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;

pub struct CreateTemplate {
    repo: Arc<dyn TemplateRepository>,
}

impl CreateTemplate {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: NewInvoiceTemplate) -> Result<InvoiceTemplate, AppError> {
        let template = InvoiceTemplate::create(input)?;
        self.repo.insert(&template)?;
        Ok(template)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateInput {
    pub id: TemplateId,
    pub name: String,
    pub base_layout: TemplateLayout,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoice,
    pub show_seller_phone: bool,
    pub show_seller_email: bool,
    pub show_registration_id: bool,
    pub show_tax_id_numbers: bool,
    pub show_signature: bool,
    pub show_due_date: bool,
    pub show_total_in_words: bool,
    pub header_text: Option<String>,
    pub footer_text: Option<String>,
}

pub struct UpdateTemplate {
    repo: Arc<dyn TemplateRepository>,
}

impl UpdateTemplate {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: UpdateTemplateInput) -> Result<InvoiceTemplate, AppError> {
        let existing = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        let rebuilt = InvoiceTemplate::create(NewInvoiceTemplate {
            name: input.name,
            base_layout: input.base_layout,
            logo_image: input.logo_image,
            accent_color: input.accent_color,
            font_family: input.font_family,
            show_seller_phone: input.show_seller_phone,
            show_seller_email: input.show_seller_email,
            show_registration_id: input.show_registration_id,
            show_tax_id_numbers: input.show_tax_id_numbers,
            show_signature: input.show_signature,
            show_due_date: input.show_due_date,
            show_total_in_words: input.show_total_in_words,
            header_text: input.header_text,
            footer_text: input.footer_text,
        })?;
        let updated = InvoiceTemplate {
            id: existing.id,
            is_default: existing.is_default,
            ..rebuilt
        };
        self.repo.update(&updated)?;
        Ok(updated)
    }
}

pub struct DeleteTemplate {
    repo: Arc<dyn TemplateRepository>,
}

impl DeleteTemplate {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: TemplateId) -> Result<(), AppError> {
        if self.repo.is_used_by_invoice(id)? {
            return Err(AppError::TemplateInUse);
        }
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct DuplicateTemplate {
    repo: Arc<dyn TemplateRepository>,
}

impl DuplicateTemplate {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: TemplateId) -> Result<InvoiceTemplate, AppError> {
        let source = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        let copy = InvoiceTemplate {
            id: TemplateId::new(),
            name: format!("{} (copy)", source.name),
            is_default: false,
            ..source
        };
        self.repo.insert(&copy)?;
        Ok(copy)
    }
}

pub struct SetDefaultTemplate {
    repo: Arc<dyn TemplateRepository>,
}

impl SetDefaultTemplate {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: TemplateId) -> Result<(), AppError> {
        if self.repo.get(id)?.is_none() {
            return Err(AppError::NotFound);
        }
        self.repo.set_default(id)?;
        Ok(())
    }
}

pub struct ListTemplates {
    repo: Arc<dyn TemplateRepository>,
}

impl ListTemplates {
    pub fn new(repo: Arc<dyn TemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self) -> Result<Vec<InvoiceTemplate>, AppError> {
        Ok(self.repo.list()?)
    }
}

pub struct PreviewTemplate {
    templates: Arc<dyn TemplateRepository>,
    settings: Arc<dyn SettingsRepository>,
    clients: Arc<dyn ClientRepository>,
    pdf: Arc<dyn PdfGenerator>,
}

impl PreviewTemplate {
    pub fn new(
        templates: Arc<dyn TemplateRepository>,
        settings: Arc<dyn SettingsRepository>,
        clients: Arc<dyn ClientRepository>,
        pdf: Arc<dyn PdfGenerator>,
    ) -> Self {
        Self {
            templates,
            settings,
            clients,
            pdf,
        }
    }

    pub fn execute(&self, input: PreviewTemplateInput) -> Result<Vec<u8>, AppError> {
        let template = match input.template_id {
            Some(id) => self.templates.get(id)?.ok_or(AppError::NotFound)?,
            None => self
                .templates
                .get_default()?
                .unwrap_or_else(sample_template),
        };
        let template_to_preview = if let Some(override_) = input.overrides {
            apply_override(template, override_)?
        } else {
            template
        };
        let seller = self.settings.get_seller_profile()?;
        let currency = self.settings.get_currency_config()?;
        let prefs = self.settings.get_app_preferences()?;
        let sample_currency = Currency::new(&currency.code).unwrap_or_else(|_| Currency::new("EUR").unwrap());

        let sample_client = sample_client();
        let _ = self.clients.list(Default::default());
        let sample_invoice = sample_invoice(sample_currency, sample_client.id, template_to_preview.id);

        let watermark = preview_watermark(prefs.language);
        let bytes = self.pdf.render(PdfRenderInput {
            invoice: &sample_invoice,
            template: &template_to_preview,
            seller: &seller,
            client: &sample_client,
            currency: &currency,
            language: prefs.language,
            is_preview: true,
            watermark: Some(watermark),
        })?;
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreviewTemplateInput {
    pub template_id: Option<TemplateId>,
    pub overrides: Option<TemplateOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOverride {
    pub base_layout: TemplateLayout,
    pub accent_color: Option<String>,
    pub font_family: FontChoice,
    pub logo_image: Option<Vec<u8>>,
    pub show_seller_phone: bool,
    pub show_seller_email: bool,
    pub show_registration_id: bool,
    pub show_tax_id_numbers: bool,
    pub show_signature: bool,
    pub show_due_date: bool,
    pub show_total_in_words: bool,
    pub header_text: Option<String>,
    pub footer_text: Option<String>,
}

fn apply_override(
    base: InvoiceTemplate,
    ov: TemplateOverride,
) -> Result<InvoiceTemplate, AppError> {
    let rebuilt = InvoiceTemplate::create(NewInvoiceTemplate {
        name: base.name.clone(),
        base_layout: ov.base_layout,
        logo_image: ov.logo_image.or(base.logo_image),
        accent_color: ov.accent_color,
        font_family: ov.font_family,
        show_seller_phone: ov.show_seller_phone,
        show_seller_email: ov.show_seller_email,
        show_registration_id: ov.show_registration_id,
        show_tax_id_numbers: ov.show_tax_id_numbers,
        show_signature: ov.show_signature,
        show_due_date: ov.show_due_date,
        show_total_in_words: ov.show_total_in_words,
        header_text: ov.header_text,
        footer_text: ov.footer_text,
    })?;
    Ok(InvoiceTemplate {
        id: base.id,
        is_default: base.is_default,
        ..rebuilt
    })
}

fn preview_watermark(lang: crate::domain::settings::Language) -> &'static str {
    match lang {
        crate::domain::settings::Language::Fr => "APERÇU",
        crate::domain::settings::Language::En => "PREVIEW",
    }
}

fn sample_template() -> InvoiceTemplate {
    InvoiceTemplate::create(NewInvoiceTemplate {
        name: "Sample".into(),
        ..Default::default()
    })
    .expect("sample template is valid")
}

fn sample_client() -> Client {
    Client::create(
        NewClient {
            name: "Sample Client".into(),
            email: Some("client@example.com".into()),
            address: Some("123 Example St\n1000 City".into()),
            phone: Some("+32 1 234 5678".into()),
            notes: None,
        },
        Utc::now(),
    )
    .expect("sample client valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryTemplateRepo {
        inner: Mutex<HashMap<TemplateId, InvoiceTemplate>>,
        used_by_invoice: Mutex<std::collections::HashSet<TemplateId>>,
    }

    impl TemplateRepository for InMemoryTemplateRepo {
        fn insert(&self, t: &InvoiceTemplate) -> Result<(), RepoError> {
            self.inner.lock().insert(t.id, t.clone());
            Ok(())
        }
        fn update(&self, t: &InvoiceTemplate) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&t.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(t.id, t.clone());
            Ok(())
        }
        fn get(&self, id: TemplateId) -> Result<Option<InvoiceTemplate>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self) -> Result<Vec<InvoiceTemplate>, RepoError> {
            let mut v: Vec<InvoiceTemplate> = self.inner.lock().values().cloned().collect();
            v.sort_by(|a, b| {
                b.is_default
                    .cmp(&a.is_default)
                    .then(a.name.cmp(&b.name))
            });
            Ok(v)
        }
        fn get_default(&self) -> Result<Option<InvoiceTemplate>, RepoError> {
            Ok(self.inner.lock().values().find(|t| t.is_default).cloned())
        }
        fn set_default(&self, id: TemplateId) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&id) {
                return Err(RepoError::NotFound);
            }
            for t in g.values_mut() {
                t.is_default = t.id == id;
            }
            Ok(())
        }
        fn is_used_by_invoice(&self, id: TemplateId) -> Result<bool, RepoError> {
            Ok(self.used_by_invoice.lock().contains(&id))
        }
        fn delete(&self, id: TemplateId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    impl InMemoryTemplateRepo {
        fn mark_used(&self, id: TemplateId) {
            self.used_by_invoice.lock().insert(id);
        }
    }

    fn repo() -> Arc<InMemoryTemplateRepo> {
        Arc::new(InMemoryTemplateRepo::default())
    }

    fn new_template(name: &str) -> NewInvoiceTemplate {
        NewInvoiceTemplate {
            name: name.into(),
            ..Default::default()
        }
    }

    #[test]
    fn create_template_persists() {
        let r = repo();
        let t = CreateTemplate::new(r.clone())
            .execute(new_template("Classic"))
            .unwrap();
        assert_eq!(t.name, "Classic");
        assert!(!t.is_default);
        assert_eq!(r.inner.lock().len(), 1);
    }

    #[test]
    fn update_preserves_is_default_flag() {
        let r = repo();
        let t = CreateTemplate::new(r.clone())
            .execute(new_template("Classic"))
            .unwrap();
        SetDefaultTemplate::new(r.clone()).execute(t.id).unwrap();
        let updated = UpdateTemplate::new(r.clone())
            .execute(UpdateTemplateInput {
                id: t.id,
                name: "Classic Renamed".into(),
                base_layout: TemplateLayout::Modern,
                logo_image: None,
                accent_color: Some("#123456".into()),
                font_family: FontChoice::Serif,
                show_seller_phone: true,
                show_seller_email: true,
                show_registration_id: true,
                show_tax_id_numbers: true,
                show_signature: true,
                show_due_date: true,
                show_total_in_words: true,
                header_text: None,
                footer_text: None,
            })
            .unwrap();
        assert_eq!(updated.name, "Classic Renamed");
        assert_eq!(updated.base_layout, TemplateLayout::Modern);
        assert!(updated.is_default, "update must not clear is_default");
    }

    #[test]
    fn delete_template_blocks_when_used_by_invoice() {
        let r = repo();
        let t = CreateTemplate::new(r.clone())
            .execute(new_template("Classic"))
            .unwrap();
        r.mark_used(t.id);
        let err = DeleteTemplate::new(r.clone()).execute(t.id).unwrap_err();
        assert!(matches!(err, AppError::TemplateInUse));
        assert!(r.inner.lock().contains_key(&t.id));
    }

    #[test]
    fn delete_template_succeeds_when_unused() {
        let r = repo();
        let t = CreateTemplate::new(r.clone())
            .execute(new_template("Classic"))
            .unwrap();
        DeleteTemplate::new(r.clone()).execute(t.id).unwrap();
        assert!(r.inner.lock().is_empty());
    }

    #[test]
    fn duplicate_copies_fields_with_new_id_and_suffix() {
        let r = repo();
        let t = CreateTemplate::new(r.clone())
            .execute(new_template("Classic"))
            .unwrap();
        SetDefaultTemplate::new(r.clone()).execute(t.id).unwrap();
        let copy = DuplicateTemplate::new(r.clone()).execute(t.id).unwrap();
        assert_ne!(copy.id, t.id);
        assert_eq!(copy.name, "Classic (copy)");
        assert!(!copy.is_default, "copy must not inherit default flag");
    }

    #[test]
    fn set_default_is_exclusive() {
        let r = repo();
        let a = CreateTemplate::new(r.clone())
            .execute(new_template("A"))
            .unwrap();
        let b = CreateTemplate::new(r.clone())
            .execute(new_template("B"))
            .unwrap();
        SetDefaultTemplate::new(r.clone()).execute(a.id).unwrap();
        SetDefaultTemplate::new(r.clone()).execute(b.id).unwrap();
        let g = r.inner.lock();
        assert!(g.get(&b.id).unwrap().is_default);
        assert!(!g.get(&a.id).unwrap().is_default);
    }

    #[test]
    fn set_default_rejects_missing_id() {
        let r = repo();
        let err = SetDefaultTemplate::new(r)
            .execute(TemplateId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn list_sorts_default_first_then_alphabetical() {
        let r = repo();
        let uc = CreateTemplate::new(r.clone());
        uc.execute(new_template("Zed")).unwrap();
        let mid = uc.execute(new_template("Middle")).unwrap();
        uc.execute(new_template("Alpha")).unwrap();
        SetDefaultTemplate::new(r.clone()).execute(mid.id).unwrap();
        let list = ListTemplates::new(r).execute().unwrap();
        assert_eq!(list[0].name, "Middle");
        assert_eq!(list[1].name, "Alpha");
        assert_eq!(list[2].name, "Zed");
    }
}

fn sample_invoice(currency: Currency, client_id: ClientId, template_id: TemplateId) -> Invoice {
    let line_items = vec![
        LineItem {
            id: LineItemId::new(),
            description: "Consulting — sample service".into(),
            quantity: dec!(10),
            unit_price: Money::new(10000, currency),
            total: Money::new(100_000, currency),
        },
        LineItem {
            id: LineItemId::new(),
            description: "Implementation".into(),
            quantity: dec!(1),
            unit_price: Money::new(50_000, currency),
            total: Money::new(50_000, currency),
        },
    ];
    let subtotal = Money::new(150_000, currency);
    let taxes_applied = vec![AppliedTax {
        tax_definition_id: None,
        tax_name: "TVA".into(),
        percentage: dec!(21),
        tax_id_number: Some("BE0123456789".into()),
        computed_amount: Money::new(31_500, currency),
    }];
    let tax_total = Money::new(31_500, currency);
    let total = Money::new(181_500, currency);
    Invoice {
        id: InvoiceId::new(),
        number: Some(InvoiceNumber(1001)),
        client_id,
        template_id: Some(template_id),
        date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
        line_items,
        taxes_applied,
        subtotal,
        tax_total,
        total,
        currency,
        status: InvoiceStatus::Finalized,
        pdf_path: None,
        notes: Some("Merci pour votre confiance.".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
