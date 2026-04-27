pub mod pagination;
pub mod accounting_queries;
pub mod bookmark_repository;
pub mod catalog_item_repository;
pub mod client_journal_repository;
pub mod client_notebook_repository;
pub mod client_repository;
pub mod credential_store;
pub mod data_management;
pub mod email_sender;
pub mod email_template_repository;
pub mod invoice_number_generator;
pub mod invoice_repository;
pub mod notebook_section_repository;
pub mod payment_repository;
pub mod pdf_generator;
pub mod pdf_storage;
pub mod settings_repository;
pub mod tax_repository;
pub mod template_repository;

pub use accounting_queries::{
    AccountingQueries, AgingBucket, AgingRow, ClientBalance, DashboardSummary,
    DerivedPaymentStatus, InvoicePaymentRow, RevenueBucket, RevenueByClient, RevenueGrouping,
};
pub use client_journal_repository::ClientJournalRepository;
pub use client_notebook_repository::ClientNotebookRepository;
pub use client_repository::{ClientRepository, ListClientsQuery};
pub use notebook_section_repository::NotebookSectionRepository;
pub use credential_store::CredentialStore;
pub use data_management::{BackupKind, BackupMetadata, BackupScope, DataManagement};
pub use email_sender::{EmailAttachment, EmailError, EmailSender, OutboundEmail};
pub use email_template_repository::EmailTemplateRepository;
pub use invoice_number_generator::InvoiceNumberGenerator;
pub use invoice_repository::{InvoiceRepository, ListInvoicesQuery};
pub use payment_repository::{ListPaymentsQuery, PaymentRepository};
pub use pdf_generator::{PdfError, PdfGenerator, PdfRenderInput};
pub use pdf_storage::PdfStorage;
pub use bookmark_repository::BookmarkRepository;
pub use catalog_item_repository::CatalogItemRepository;
pub use settings_repository::SettingsRepository;
pub use tax_repository::TaxRepository;
pub use template_repository::TemplateRepository;
pub use pagination::{Page, PaginationParams};
