use serde::Serialize;
use typst::foundations::{Dict, IntoValue};
use typst::layout::PagedDocument;
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_as_lib::TypstEngine;

use crate::application::ports::{PdfError, PdfGenerator, PdfRenderInput};
use crate::domain::invoice::Invoice;
use crate::domain::settings::{CurrencyConfig, Language};

const MAIN_TEMPLATE: &str = include_str!("../../templates/main.typ");
const CLASSIC_TEMPLATE: &str = include_str!("../../templates/classic.typ");
const MODERN_TEMPLATE: &str = include_str!("../../templates/modern.typ");
const MINIMAL_TEMPLATE: &str = include_str!("../../templates/minimal.typ");

pub struct TypstPdfGenerator;

impl TypstPdfGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypstPdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfGenerator for TypstPdfGenerator {
    fn render(&self, input: PdfRenderInput<'_>) -> Result<Vec<u8>, PdfError> {
        let data = build_template_data(&input);
        let json_data = serde_json::to_string(&data)
            .map_err(|e| PdfError::Render(format!("serialize data: {e}")))?;

        let engine = TypstEngine::builder()
            .main_file(MAIN_TEMPLATE)
            .with_static_source_file_resolver([
                ("classic.typ", CLASSIC_TEMPLATE),
                ("modern.typ", MODERN_TEMPLATE),
                ("minimal.typ", MINIMAL_TEMPLATE),
            ])
            .search_fonts_with(TypstKitFontOptions::default().include_system_fonts(true))
            .build();

        let mut inputs = Dict::new();
        inputs.insert("data".into(), json_data.into_value());

        let compiled = engine
            .compile_with_input::<_, PagedDocument>(inputs)
            .output
            .map_err(|e| PdfError::Render(format!("typst compile: {e:?}")))?;

        typst_pdf::pdf(&compiled, &typst_pdf::PdfOptions::default())
            .map_err(|e| PdfError::Render(format!("pdf encode: {e:?}")))
    }
}

#[derive(Serialize)]
struct TemplateData<'a> {
    layout: &'a str,
    font_family: &'a str,
    accent_color: &'a str,
    logo_base64: Option<String>,
    toggles: Toggles,
    header_text: Option<&'a str>,
    footer_text: Option<&'a str>,
    watermark: Option<&'a str>,
    is_preview: bool,
    labels: Labels,
    seller: SellerView<'a>,
    client: ClientView<'a>,
    invoice: InvoiceView,
}

#[derive(Serialize)]
struct Labels {
    invoice: &'static str,
    bill_to: &'static str,
    due: &'static str,
    description: &'static str,
    quantity: &'static str,
    unit_price: &'static str,
    total: &'static str,
    subtotal: &'static str,
    tel: &'static str,
    reg: &'static str,
    signature: &'static str,
}

impl Labels {
    fn for_language(lang: Language) -> Self {
        match lang {
            Language::Fr => Labels {
                invoice: "FACTURE",
                bill_to: "Facturée à :",
                due: "Échéance :",
                description: "Description",
                quantity: "Qté",
                unit_price: "P.U.",
                total: "Total",
                subtotal: "Sous-total",
                tel: "Tél. : ",
                reg: "N° : ",
                signature: "Signature :",
            },
            Language::En => Labels {
                invoice: "INVOICE",
                bill_to: "Bill to:",
                due: "Due:",
                description: "Description",
                quantity: "Qty",
                unit_price: "Unit",
                total: "Total",
                subtotal: "Subtotal",
                tel: "Tel: ",
                reg: "Reg: ",
                signature: "Signature:",
            },
        }
    }
}

#[derive(Serialize)]
struct Toggles {
    show_seller_phone: bool,
    show_seller_email: bool,
    show_registration_id: bool,
    show_tax_id_numbers: bool,
    show_signature: bool,
    show_due_date: bool,
    show_total_in_words: bool,
}

#[derive(Serialize)]
struct SellerView<'a> {
    name: &'a str,
    title: Option<&'a str>,
    registration_id: Option<&'a str>,
    address: Option<&'a str>,
    phone: Option<&'a str>,
    email: Option<&'a str>,
    signature_base64: Option<String>,
}

#[derive(Serialize)]
struct ClientView<'a> {
    name: &'a str,
    email: Option<&'a str>,
    address: Option<&'a str>,
    phone: Option<&'a str>,
}

#[derive(Serialize)]
struct InvoiceView {
    number: String,
    status: String,
    date: String,
    due_date: Option<String>,
    currency_symbol: String,
    currency_code: String,
    line_items: Vec<LineItemView>,
    taxes: Vec<TaxView>,
    subtotal: String,
    tax_total: String,
    total: String,
    total_in_words: String,
    notes: Option<String>,
}

#[derive(Serialize)]
struct LineItemView {
    description: String,
    quantity: String,
    unit_price: String,
    total: String,
}

#[derive(Serialize)]
struct TaxView {
    name: String,
    percentage: String,
    tax_id_number: Option<String>,
    amount: String,
}

fn build_template_data<'a>(input: &'a PdfRenderInput<'a>) -> TemplateData<'a> {
    let template = input.template;
    let seller = input.seller;
    let client = input.client;
    let currency = input.currency;

    TemplateData {
        layout: template.base_layout.as_str(),
        font_family: template.font_family.as_str(),
        accent_color: template.accent_color.as_deref().unwrap_or("#111827"),
        logo_base64: template.logo_image.as_ref().map(base64_encode),
        toggles: Toggles {
            show_seller_phone: template.show_seller_phone,
            show_seller_email: template.show_seller_email,
            show_registration_id: template.show_registration_id,
            show_tax_id_numbers: template.show_tax_id_numbers,
            show_signature: template.show_signature,
            show_due_date: template.show_due_date,
            show_total_in_words: template.show_total_in_words,
        },
        header_text: template.header_text.as_deref(),
        footer_text: template.footer_text.as_deref(),
        watermark: input.watermark,
        is_preview: input.is_preview,
        labels: Labels::for_language(input.language),
        seller: SellerView {
            name: &seller.name,
            title: seller.title.as_deref(),
            registration_id: seller.registration_id.as_deref(),
            address: seller.address.as_deref(),
            phone: seller.phone.as_deref(),
            email: seller.email.as_deref(),
            signature_base64: seller.signature_image.as_ref().map(base64_encode),
        },
        client: ClientView {
            name: &client.name,
            email: client.default_email(),
            address: client.address.as_deref(),
            phone: client.default_phone(),
        },
        invoice: build_invoice_view(input.invoice, currency),
    }
}

fn build_invoice_view(invoice: &Invoice, currency: &CurrencyConfig) -> InvoiceView {
    let line_items = invoice
        .line_items
        .iter()
        .map(|li| LineItemView {
            description: li.description.clone(),
            quantity: format_quantity(li.quantity),
            unit_price: li.unit_price.format(),
            total: li.total.format(),
        })
        .collect();
    let taxes = invoice
        .taxes_applied
        .iter()
        .map(|t| TaxView {
            name: t.tax_name.clone(),
            percentage: format_percent(t.percentage),
            tax_id_number: t.tax_id_number.clone(),
            amount: t.computed_amount.format(),
        })
        .collect();
    InvoiceView {
        number: invoice
            .number
            .map(|n| n.to_string())
            .unwrap_or_else(|| "DRAFT".into()),
        status: invoice.status.as_str().to_string(),
        date: invoice.date.format("%Y-%m-%d").to_string(),
        due_date: invoice.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
        currency_symbol: currency.currency().symbol().to_string(),
        currency_code: currency.currency().code().to_string(),
        line_items,
        taxes,
        subtotal: invoice.subtotal.format(),
        tax_total: invoice.tax_total.format(),
        total: invoice.total.format(),
        total_in_words: amount_in_words(invoice.total),
        notes: invoice.notes.clone(),
    }
}

fn format_quantity(q: rust_decimal::Decimal) -> String {
    let normalized = q.normalize();
    normalized.to_string()
}

fn format_percent(p: rust_decimal::Decimal) -> String {
    let normalized = p.normalize();
    format!("{normalized}%")
}

fn amount_in_words(amount: crate::domain::money::Money) -> String {
    let meta = amount.currency().meta();
    let scale = amount.currency().minor_unit_scale();
    let minor = amount.minor_units();
    let whole = minor / scale;
    if meta.fraction_digits == 0 {
        return format!("{} {}", whole, meta.main_unit_name);
    }
    let frac = (minor % scale).unsigned_abs();
    let sub = meta.sub_unit_name.unwrap_or("");
    format!(
        "{whole} {main} {frac:0width$} {sub}",
        main = meta.main_unit_name,
        width = meta.fraction_digits as usize,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::client::{Client, NewClient};
    use crate::domain::invoice::{AppliedTax, InvoiceId, InvoiceNumber, InvoiceStatus};
    use crate::domain::line_item::{LineItem, LineItemId};
    use crate::domain::money::{Currency, Money};
    use crate::domain::settings::{CurrencyConfig, SellerProfile};
    use crate::domain::template::{
        FontChoice, InvoiceTemplate, NewInvoiceTemplate, TemplateLayout,
    };
    use chrono::{NaiveDate, Utc};
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn sample_template(layout: TemplateLayout) -> InvoiceTemplate {
        let mut t = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Classic".into(),
            base_layout: layout,
            accent_color: Some("#2563EB".into()),
            font_family: FontChoice::SansSerif,
            ..Default::default()
        })
        .unwrap();
        t.is_default = true;
        t
    }

    fn sample_invoice() -> Invoice {
        let currency = eur();
        Invoice {
            id: InvoiceId::new(),
            number: Some(InvoiceNumber(42)),
            client_id: crate::domain::client::ClientId::new(),
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
            line_items: vec![
                LineItem {
                    id: LineItemId::new(),
                    description: "Consulting".into(),
                    quantity: dec!(2),
                    unit_price: Money::new(15000, currency),
                    total: Money::new(30000, currency),
                },
                LineItem {
                    id: LineItemId::new(),
                    description: "Implementation".into(),
                    quantity: dec!(1),
                    unit_price: Money::new(50000, currency),
                    total: Money::new(50000, currency),
                },
            ],
            taxes_applied: vec![AppliedTax {
                tax_definition_id: None,
                tax_name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: Some("BE0123456789".into()),
                computed_amount: Money::new(16800, currency),
            }],
            subtotal: Money::new(80000, currency),
            tax_total: Money::new(16800, currency),
            total: Money::new(96800, currency),
            currency,
            status: InvoiceStatus::Finalized,
            pdf_path: None,
            notes: Some("Thanks for your business.".into()),
            email_sends: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_client() -> Client {
        use crate::domain::client::NewContactEntry;
        Client::create(
            NewClient {
                name: "Acme Corp".into(),
                emails: vec![NewContactEntry {
                    value: "billing@acme.example".into(),
                    label: None,
                    is_default: true,
                }],
                phones: vec![NewContactEntry {
                    value: "+32 1 234 5678".into(),
                    label: None,
                    is_default: true,
                }],
                address: Some("123 Main St\n1000 City".into()),
                notes: None,
                referred_by: None,
            },
            Utc::now(),
        )
        .unwrap()
    }

    fn sample_seller() -> SellerProfile {
        SellerProfile {
            name: "Terative SA".into(),
            title: Some("Freelance consultant".into()),
            registration_id: Some("BE0987654321".into()),
            address: Some("42 Example Ave\n1050 Brussels".into()),
            phone: Some("+32 2 000 0000".into()),
            email: Some("hello@terative.example".into()),
            signature_image: None,
        }
    }

    fn render(layout: TemplateLayout) -> Vec<u8> {
        let template = sample_template(layout);
        let invoice = sample_invoice();
        let client = sample_client();
        let seller = sample_seller();
        let currency = CurrencyConfig::default();
        TypstPdfGenerator::new()
            .render(PdfRenderInput {
                invoice: &invoice,
                template: &template,
                seller: &seller,
                client: &client,
                currency: &currency,
                language: Language::Fr,
                is_preview: false,
                watermark: None,
            })
            .unwrap_or_else(|e| panic!("render {layout:?} failed: {e}"))
    }

    #[test]
    fn renders_classic_layout_to_pdf_bytes() {
        let bytes = render(TemplateLayout::Classic);
        assert!(bytes.starts_with(b"%PDF"), "output must be a valid PDF");
        assert!(bytes.len() > 1000, "pdf should be non-trivial size");
    }

    #[test]
    fn renders_modern_layout_to_pdf_bytes() {
        let bytes = render(TemplateLayout::Modern);
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn renders_minimal_layout_to_pdf_bytes() {
        let bytes = render(TemplateLayout::Minimal);
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn renders_preview_with_watermark() {
        let template = sample_template(TemplateLayout::Classic);
        let invoice = sample_invoice();
        let client = sample_client();
        let seller = sample_seller();
        let currency = CurrencyConfig::default();
        let bytes = TypstPdfGenerator::new()
            .render(PdfRenderInput {
                invoice: &invoice,
                template: &template,
                seller: &seller,
                client: &client,
                currency: &currency,
                language: Language::Fr,
                is_preview: true,
                watermark: Some("APERÇU"),
            })
            .unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }
}

fn base64_encode(bytes: &Vec<u8>) -> String {
    const CHARSET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let chunks = bytes.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARSET[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARSET[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

