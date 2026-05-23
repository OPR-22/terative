use crate::application::RepoError;
use crate::domain::client::Client;
use crate::domain::invoice::Invoice;
use crate::domain::settings::{CurrencyConfig, Language, SellerProfile};
use crate::domain::template::InvoiceTemplate;

pub struct PdfRenderInput<'a> {
    pub invoice: &'a Invoice,
    pub template: &'a InvoiceTemplate,
    pub seller: &'a SellerProfile,
    pub client: &'a Client,
    pub currency: &'a CurrencyConfig,
    pub language: Language,
    pub is_preview: bool,
    pub watermark: Option<&'a str>,
}

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("pdf generation failed: {0}")]
    Render(String),
    #[error(transparent)]
    Repo(#[from] RepoError),
}

pub trait PdfGenerator: Send + Sync {
    fn render(&self, input: PdfRenderInput<'_>) -> Result<Vec<u8>, PdfError>;
}
