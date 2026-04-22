pub mod accounting_commands;
pub mod bookmark_commands;
pub mod catalog_item_commands;
pub mod client_commands;
pub mod data_commands;
pub mod email_commands;
pub mod email_template_commands;
pub mod invoice_commands;
pub mod notebook_commands;
pub mod payment_commands;
pub mod settings_commands;
pub mod tax_commands;
pub mod template_commands;

use std::sync::Arc;

use crate::adapters::sqlite::{
    SqliteAccountingRepository, SqliteCatalogItemRepository, SqliteClientJournalRepository,
    SqliteClientNotebookRepository, SqliteClientRepository, SqliteEmailTemplateRepository,
    SqliteInvoiceNumberGenerator, SqliteInvoiceRepository, SqliteNotebookSectionRepository,
    SqlitePaymentRepository, SqliteSettingsRepository, SqliteTaxRepository,
    SqliteTemplateRepository,
};
use crate::adapters::{
    FilesystemDataManagement, FilesystemPdfStorage, KeyringCredentialStore, LettreEmailSender,
    TypstPdfGenerator,
};
use crate::application::ports::DataManagement;
use crate::application::accounting_usecases::AccountingService;
use crate::application::client_usecases::{
    ArchiveClient, CreateClient, GetClientDetail, ListClients, UnarchiveClient, UpdateClient,
};
use crate::application::email_template_usecases::{
    CreateEmailTemplate, DeleteEmailTemplate, ListEmailTemplates, SetDefaultEmailTemplate,
    UpdateEmailTemplate,
};
use crate::application::email_usecases::{
    SendInvoice, TestEmailConnection, UpdateEmailConfig, UpdateEmailPassword,
};
use crate::application::invoice_usecases::{
    CancelInvoice, CreateDraftInvoice, DuplicateInvoice, FinalizeInvoice, GetInvoice,
    ListInvoices, UpdateDraftInvoice,
};
use crate::application::notebook_usecases::{
    CountSectionEntries, CreateJournalEntry, CreateNotebookSection, DeleteJournalEntry,
    DeleteNotebookSection, GetClientNotebook, GetJournalEntry, ListClientJournal,
    ListNotebookSections, RenameNotebookSection, ReorderNotebookSections, SaveClientNotebook,
    UpdateJournalEntry,
};
use crate::application::payment_usecases::{
    DeletePayment, GetPayment, ListPayments, RecordPayment, UpdatePayment,
};
use crate::application::catalog_item_usecases::{
    ArchiveCatalogItem, CreateCatalogItem, ListCatalogItems, UnarchiveCatalogItem,
    UpdateCatalogItem,
};
use crate::application::settings_usecases::{
    GetSettings, UpdateAppPreferences, UpdateCurrency, UpdateSellerProfile,
};
use crate::application::tax_usecases::{ArchiveTax, CreateTax, ListTaxes, UnarchiveTax, UpdateTax};
use crate::application::template_usecases::{
    CreateTemplate, DeleteTemplate, DuplicateTemplate, ListTemplates, PreviewTemplate,
    SetDefaultTemplate, UpdateTemplate,
};

pub struct AppState {
    pub create_client: CreateClient,
    pub update_client: UpdateClient,
    pub archive_client: ArchiveClient,
    pub unarchive_client: UnarchiveClient,
    pub list_clients: ListClients,
    pub get_client_detail: GetClientDetail,

    pub create_catalog_item: CreateCatalogItem,
    pub update_catalog_item: UpdateCatalogItem,
    pub archive_catalog_item: ArchiveCatalogItem,
    pub unarchive_catalog_item: UnarchiveCatalogItem,
    pub list_catalog_items: ListCatalogItems,

    pub get_settings: GetSettings,
    pub update_seller_profile: UpdateSellerProfile,
    pub update_currency: UpdateCurrency,
    pub update_app_preferences: UpdateAppPreferences,

    pub create_tax: CreateTax,
    pub update_tax: UpdateTax,
    pub archive_tax: ArchiveTax,
    pub unarchive_tax: UnarchiveTax,
    pub list_taxes: ListTaxes,

    pub create_template: CreateTemplate,
    pub update_template: UpdateTemplate,
    pub delete_template: DeleteTemplate,
    pub duplicate_template: DuplicateTemplate,
    pub set_default_template: SetDefaultTemplate,
    pub list_templates: ListTemplates,
    pub preview_template: PreviewTemplate,

    pub create_draft_invoice: CreateDraftInvoice,
    pub update_draft_invoice: UpdateDraftInvoice,
    pub finalize_invoice: FinalizeInvoice,
    pub duplicate_invoice: DuplicateInvoice,
    pub cancel_invoice: CancelInvoice,
    pub list_invoices: ListInvoices,
    pub get_invoice: GetInvoice,

    pub update_email_config: UpdateEmailConfig,
    pub update_email_password: UpdateEmailPassword,
    pub test_email_connection: TestEmailConnection,
    pub send_invoice: SendInvoice,

    pub create_email_template: CreateEmailTemplate,
    pub update_email_template: UpdateEmailTemplate,
    pub delete_email_template: DeleteEmailTemplate,
    pub set_default_email_template: SetDefaultEmailTemplate,
    pub list_email_templates: ListEmailTemplates,

    pub record_payment: RecordPayment,
    pub update_payment: UpdatePayment,
    pub delete_payment: DeletePayment,
    pub list_payments: ListPayments,
    pub get_payment: GetPayment,

    pub accounting: AccountingService,

    pub data_management: Arc<dyn DataManagement>,
    pub user_backup_dir: std::path::PathBuf,
    pub system_backup_dir: std::path::PathBuf,

    pub create_notebook_section: CreateNotebookSection,
    pub rename_notebook_section: RenameNotebookSection,
    pub delete_notebook_section: DeleteNotebookSection,
    pub count_section_entries: CountSectionEntries,
    pub reorder_notebook_sections: ReorderNotebookSections,
    pub list_notebook_sections: ListNotebookSections,

    pub get_client_notebook: GetClientNotebook,
    pub save_client_notebook: SaveClientNotebook,

    pub create_journal_entry: CreateJournalEntry,
    pub update_journal_entry: UpdateJournalEntry,
    pub delete_journal_entry: DeleteJournalEntry,
    pub list_client_journal: ListClientJournal,
    pub get_journal_entry: GetJournalEntry,
}

impl AppState {
    pub fn new(
        db: crate::adapters::sqlite::Db,
        db_path: std::path::PathBuf,
        default_pdf_dir: std::path::PathBuf,
        user_backup_dir: std::path::PathBuf,
        system_backup_dir: std::path::PathBuf,
    ) -> Self {
        let client_repo = Arc::new(SqliteClientRepository::new(db.clone()));
        let catalog_item_repo = Arc::new(SqliteCatalogItemRepository::new(db.clone()));
        let settings_repo = Arc::new(SqliteSettingsRepository::new(db.clone()));
        let tax_repo = Arc::new(SqliteTaxRepository::new(db.clone()));
        let template_repo = Arc::new(SqliteTemplateRepository::new(db.clone()));
        let invoice_repo = Arc::new(SqliteInvoiceRepository::new(db.clone()));
        let payment_repo = Arc::new(SqlitePaymentRepository::new(db.clone()));
        let accounting_repo = Arc::new(SqliteAccountingRepository::new(db.clone()));
        let notebook_section_repo = Arc::new(SqliteNotebookSectionRepository::new(db.clone()));
        let client_notebook_repo = Arc::new(SqliteClientNotebookRepository::new(db.clone()));
        let client_journal_repo = Arc::new(SqliteClientJournalRepository::new(db.clone()));
        let email_template_repo = Arc::new(SqliteEmailTemplateRepository::new(db.clone()));
        let number_gen = Arc::new(SqliteInvoiceNumberGenerator::new(db.clone()));
        let pdf = Arc::new(TypstPdfGenerator::new());
        let pdf_storage = Arc::new(FilesystemPdfStorage::new(
            settings_repo.clone(),
            default_pdf_dir,
        ));
        let credentials = Arc::new(KeyringCredentialStore::new("terative", "smtp"));
        let email_sender = Arc::new(LettreEmailSender::new());
        let data_management: Arc<dyn DataManagement> = Arc::new(FilesystemDataManagement::new(
            db.clone(),
            db_path,
            settings_repo.clone(),
            user_backup_dir.clone(),
            system_backup_dir.clone(),
        ));

        Self {
            create_client: CreateClient::new(client_repo.clone()),
            update_client: UpdateClient::new(client_repo.clone()),
            archive_client: ArchiveClient::new(client_repo.clone()),
            unarchive_client: UnarchiveClient::new(client_repo.clone()),
            list_clients: ListClients::new(client_repo.clone()),
            get_client_detail: GetClientDetail::new(client_repo.clone()),

            create_catalog_item: CreateCatalogItem::new(catalog_item_repo.clone()),
            update_catalog_item: UpdateCatalogItem::new(catalog_item_repo.clone()),
            archive_catalog_item: ArchiveCatalogItem::new(catalog_item_repo.clone()),
            unarchive_catalog_item: UnarchiveCatalogItem::new(catalog_item_repo.clone()),
            list_catalog_items: ListCatalogItems::new(catalog_item_repo),

            get_settings: GetSettings::new(settings_repo.clone(), credentials.clone()),
            update_seller_profile: UpdateSellerProfile::new(settings_repo.clone()),
            update_currency: UpdateCurrency::new(settings_repo.clone()),
            update_app_preferences: UpdateAppPreferences::new(settings_repo.clone()),

            create_tax: CreateTax::new(tax_repo.clone()),
            update_tax: UpdateTax::new(tax_repo.clone()),
            archive_tax: ArchiveTax::new(tax_repo.clone()),
            unarchive_tax: UnarchiveTax::new(tax_repo.clone()),
            list_taxes: ListTaxes::new(tax_repo.clone()),

            create_template: CreateTemplate::new(template_repo.clone()),
            update_template: UpdateTemplate::new(template_repo.clone()),
            delete_template: DeleteTemplate::new(template_repo.clone()),
            duplicate_template: DuplicateTemplate::new(template_repo.clone()),
            set_default_template: SetDefaultTemplate::new(template_repo.clone()),
            list_templates: ListTemplates::new(template_repo.clone()),
            preview_template: PreviewTemplate::new(
                template_repo.clone(),
                settings_repo.clone(),
                client_repo.clone(),
                pdf.clone(),
            ),

            create_draft_invoice: CreateDraftInvoice::new(invoice_repo.clone(), tax_repo.clone()),
            update_draft_invoice: UpdateDraftInvoice::new(invoice_repo.clone(), tax_repo.clone()),
            finalize_invoice: FinalizeInvoice::new(
                invoice_repo.clone(),
                number_gen,
                template_repo.clone(),
                settings_repo.clone(),
                client_repo.clone(),
                pdf.clone(),
                pdf_storage.clone(),
            ),
            duplicate_invoice: DuplicateInvoice::new(invoice_repo.clone()),
            cancel_invoice: CancelInvoice::new(
                invoice_repo.clone(),
                client_repo.clone(),
                template_repo,
                settings_repo.clone(),
                pdf,
                pdf_storage,
            ),
            list_invoices: ListInvoices::new(invoice_repo.clone(), payment_repo.clone()),
            get_invoice: GetInvoice::new(invoice_repo.clone(), payment_repo.clone()),

            update_email_config: UpdateEmailConfig::new(settings_repo.clone()),
            update_email_password: UpdateEmailPassword::new(credentials.clone()),
            test_email_connection: TestEmailConnection::new(
                settings_repo.clone(),
                credentials.clone(),
                email_sender.clone(),
            ),
            send_invoice: SendInvoice::new(
                invoice_repo.clone(),
                client_repo,
                settings_repo,
                credentials,
                email_sender,
                email_template_repo.clone(),
            ),

            create_email_template: CreateEmailTemplate::new(email_template_repo.clone()),
            update_email_template: UpdateEmailTemplate::new(email_template_repo.clone()),
            delete_email_template: DeleteEmailTemplate::new(email_template_repo.clone()),
            set_default_email_template: SetDefaultEmailTemplate::new(email_template_repo.clone()),
            list_email_templates: ListEmailTemplates::new(email_template_repo),

            record_payment: RecordPayment::new(payment_repo.clone(), invoice_repo.clone()),
            update_payment: UpdatePayment::new(payment_repo.clone(), invoice_repo.clone()),
            delete_payment: DeletePayment::new(payment_repo.clone()),
            list_payments: ListPayments::new(payment_repo.clone()),
            get_payment: GetPayment::new(payment_repo),

            accounting: AccountingService::new(accounting_repo),

            data_management,
            user_backup_dir,
            system_backup_dir,

            create_notebook_section: CreateNotebookSection::new(notebook_section_repo.clone()),
            rename_notebook_section: RenameNotebookSection::new(notebook_section_repo.clone()),
            delete_notebook_section: DeleteNotebookSection::new(notebook_section_repo.clone()),
            count_section_entries: CountSectionEntries::new(notebook_section_repo.clone()),
            reorder_notebook_sections: ReorderNotebookSections::new(notebook_section_repo.clone()),
            list_notebook_sections: ListNotebookSections::new(notebook_section_repo.clone()),

            get_client_notebook: GetClientNotebook::new(
                notebook_section_repo,
                client_notebook_repo.clone(),
            ),
            save_client_notebook: SaveClientNotebook::new(client_notebook_repo),

            create_journal_entry: CreateJournalEntry::new(client_journal_repo.clone()),
            update_journal_entry: UpdateJournalEntry::new(client_journal_repo.clone()),
            delete_journal_entry: DeleteJournalEntry::new(client_journal_repo.clone()),
            list_client_journal: ListClientJournal::new(client_journal_repo.clone()),
            get_journal_entry: GetJournalEntry::new(client_journal_repo),
        }
    }
}

pub fn to_ipc_err(e: crate::application::AppError) -> String {
    e.to_string()
}
