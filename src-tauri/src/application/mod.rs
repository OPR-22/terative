pub mod accounting_usecases;
pub mod bookmark_usecases;
pub mod catalog_item_usecases;
pub mod client_usecases;
pub mod dto;
pub mod email_log_usecases;
pub mod email_template_usecases;
pub mod email_usecases;
pub mod invoice_usecases;
pub mod notebook_usecases;
pub mod payment_usecases;
pub mod ports;
#[cfg(debug_assertions)]
pub mod seed_usecases;
pub mod settings_usecases;
pub mod tax_usecases;
pub mod template_usecases;

use crate::domain::bookmark::BookmarkError;
use crate::domain::catalog_item::CatalogItemError;
use crate::domain::client::ClientError;
use crate::domain::email_log::EmailLogError;
use crate::domain::email_template::EmailTemplateError;
use crate::domain::invoice::InvoiceError;
use crate::domain::line_item::LineItemError;
use crate::domain::money::MoneyError;
use crate::domain::notebook::{
    JournalEntryError, NotebookError, NotebookSectionError,
};
use crate::domain::payment::PaymentError;
use crate::domain::settings::{CurrencyConfigError, EmailConfigError};
use crate::domain::tax::TaxError;
use crate::domain::template::TemplateError;
use crate::application::ports::{EmailError, PdfError};

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("entity not found")]
    NotFound,
    #[error("constraint violation: {0}")]
    Conflict(String),
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),
    #[error(transparent)]
    CatalogItem(#[from] CatalogItemError),
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Tax(#[from] TaxError),
    #[error(transparent)]
    Template(#[from] TemplateError),
    #[error(transparent)]
    Invoice(#[from] InvoiceError),
    #[error(transparent)]
    Payment(#[from] PaymentError),
    #[error(transparent)]
    NotebookSection(#[from] NotebookSectionError),
    #[error(transparent)]
    Notebook(#[from] NotebookError),
    #[error(transparent)]
    JournalEntry(#[from] JournalEntryError),
    #[error(transparent)]
    LineItem(#[from] LineItemError),
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error(transparent)]
    Currency(#[from] CurrencyConfigError),
    #[error(transparent)]
    EmailConfig(#[from] EmailConfigError),
    #[error(transparent)]
    EmailLog(#[from] EmailLogError),
    #[error(transparent)]
    EmailTemplate(#[from] EmailTemplateError),
    #[error(transparent)]
    Email(#[from] EmailError),
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error(transparent)]
    Pdf(#[from] PdfError),
    #[error("cannot delete client with existing invoices")]
    ClientHasInvoices,
    #[error("template is in use by one or more invoices")]
    TemplateInUse,
    #[error("smtp password not configured")]
    MissingSmtpPassword,
    #[error("invoice has no pdf to send; finalize it first")]
    MissingInvoicePdf,
    #[error("no default email template configured for this type")]
    NoDefaultEmailTemplate,
    #[error("cannot delete the default email template")]
    EmailTemplateIsDefault,
    #[error("entity not found")]
    NotFound,
}
