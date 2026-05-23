pub mod accounting_commands;
pub mod audit_commands;
pub mod bookmark_commands;
pub mod catalog_item_commands;
pub mod client_commands;
pub mod data_commands;
pub mod email_commands;
pub mod email_template_commands;
pub mod invoice_commands;
pub mod notebook_commands;
pub mod org_commands;
pub mod payment_commands;
pub mod search_commands;
#[cfg(debug_assertions)]
pub mod seed_commands;
pub mod settings_commands;
pub mod tax_commands;
pub mod template_commands;

use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::adapters::sqlite::{
    Db, SqliteAccountingRepository, SqliteAuditRepository, SqliteBookmarkRepository,
    SqliteCatalogItemRepository, SqliteClientJournalRepository, SqliteClientNotebookRepository,
    SqliteClientRepository, SqliteEmailLogRepository, SqliteEmailTemplateRepository,
    SqliteInvoiceNumberGenerator, SqliteInvoiceRepository, SqliteNotebookSectionRepository,
    SqlitePaymentRepository, SqliteSearchRepository, SqliteSettingsRepository,
    SqliteTaxRepository, SqliteTemplateRepository,
};
use crate::adapters::{
    FilesystemDataManagement, FilesystemPdfStorage, InProcessEventBus, KeyringCredentialStore,
    LettreEmailSender, TypstPdfGenerator,
};
use crate::application::accounting_usecases::AccountingService;
use crate::application::audit_handlers::{register_all, AuditHandlerContext};
use crate::application::audit_usecases::{
    CleanupAudits, PaginateAuditForClient, PaginateAuditForInvoice, PaginateRecentAudit,
};
use crate::application::bookmark_usecases::{
    CreateBookmark, DeleteBookmark, ListBookmarks, ReorderBookmarks, UpdateBookmark,
};
use crate::application::catalog_item_usecases::{
    ArchiveCatalogItem, CreateCatalogItem, ListCatalogItems, UnarchiveCatalogItem,
    UpdateCatalogItem,
};
use crate::application::client_usecases::{
    ArchiveClient, CreateClient, GetClientDetail, ListClientAttributeValues, ListClients,
    UnarchiveClient, UpdateClient,
};
use crate::application::email_log_usecases::ListEmailLogsForClient;
use crate::application::email_template_usecases::{
    CreateEmailTemplate, DeleteEmailTemplate, ListEmailTemplates, SetDefaultEmailTemplate,
    UpdateEmailTemplate,
};
use crate::application::email_usecases::{
    SendInvoice, TestEmailConnection, UpdateEmailConfig, UpdateEmailPassword,
};
use crate::application::invoice_usecases::{
    CancelInvoice, CreateDraftInvoice, DuplicateInvoice, FinalizeInvoice, GetInvoice,
    GetInvoiceNumbering, GetInvoicePdf, ListInvoices, OpenInvoiceExternally, PrintInvoice,
    SetStartingInvoiceNumber, UpdateDraftInvoice,
};
use crate::application::notebook_usecases::{
    CountSectionEntries, CreateJournalEntry, CreateNotebookSection, DeleteJournalEntry,
    DeleteNotebookSection, GetClientNotebook, GetJournalEntry, ListClientJournal,
    ListNotebookSections, RenameNotebookSection, ReorderNotebookSections, SaveClientNotebook,
    UpdateJournalEntry,
};
use crate::application::data_usecases::{AutoBackupIfDue, CreateBackup};
use crate::application::org_registry::OrgRegistry;
use crate::application::payment_usecases::{
    DeletePayment, GetPayment, ListPayments, RecordPayment, UpdatePayment,
};
use crate::application::ports::{AuditRepository, DataManagement, EventBus, OrgKeyStore};
use crate::application::search_usecases::GlobalSearch;
#[cfg(debug_assertions)]
use crate::application::seed_usecases::SeedDatabase;
use crate::application::settings_usecases::{
    GetSettings, UpdateAppPreferences, UpdateCurrency, UpdateSellerProfile,
};
use crate::application::tax_usecases::{ArchiveTax, CreateTax, ListTaxes, UnarchiveTax, UpdateTax};
use crate::application::template_usecases::{
    CreateTemplate, DeleteTemplate, DuplicateTemplate, ListTemplates, PreviewTemplate,
    SetDefaultTemplate, UpdateTemplate,
};
use crate::application::{AppError, SecretKey};
use crate::domain::org::OrgCode;

/// Per-org service bundle: the use cases and Arc-shared adapters that live
/// inside an open organisation. Constructed by `org_open` and held inside
/// `AppState` via an `ArcSwap`. Dropping the `Arc<OrgServices>` releases the
/// underlying SQLite connection.
pub struct OrgServices {
    pub code: OrgCode,
    pub user_backup_dir: PathBuf,
    pub system_backup_dir: PathBuf,

    pub create_client: CreateClient,
    pub update_client: UpdateClient,
    pub archive_client: ArchiveClient,
    pub unarchive_client: UnarchiveClient,
    pub list_clients: ListClients,
    pub get_client_detail: GetClientDetail,
    pub list_client_attribute_values: ListClientAttributeValues,

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

    pub create_bookmark: CreateBookmark,
    pub update_bookmark: UpdateBookmark,
    pub delete_bookmark: DeleteBookmark,
    pub list_bookmarks: ListBookmarks,
    pub reorder_bookmarks: ReorderBookmarks,

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
    pub get_invoice_pdf: GetInvoicePdf,
    pub print_invoice: PrintInvoice,
    pub open_invoice_externally: OpenInvoiceExternally,
    pub get_invoice_numbering: GetInvoiceNumbering,
    pub set_starting_invoice_number: SetStartingInvoiceNumber,

    pub update_email_config: UpdateEmailConfig,
    pub update_email_password: UpdateEmailPassword,
    pub test_email_connection: TestEmailConnection,
    pub send_invoice: SendInvoice,

    pub list_email_logs_for_client: ListEmailLogsForClient,

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

    pub global_search: GlobalSearch,

    pub data_management: Arc<dyn DataManagement>,
    pub create_backup: CreateBackup,
    pub auto_backup_if_due: AutoBackupIfDue,

    pub paginate_recent_audit: PaginateRecentAudit,
    pub paginate_audit_for_client: PaginateAuditForClient,
    pub paginate_audit_for_invoice: PaginateAuditForInvoice,
    pub cleanup_audits: CleanupAudits,

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

    #[cfg(debug_assertions)]
    pub seed_database: SeedDatabase,
}

impl OrgServices {
    pub fn new(
        code: OrgCode,
        db: Db,
        db_path: PathBuf,
        default_pdf_dir: PathBuf,
        user_backup_dir: PathBuf,
        system_backup_dir: PathBuf,
        key: Option<SecretKey>,
    ) -> Self {
        let client_repo = Arc::new(SqliteClientRepository::new(db.clone()));
        let catalog_item_repo = Arc::new(SqliteCatalogItemRepository::new(db.clone()));
        let settings_repo = Arc::new(SqliteSettingsRepository::new(db.clone()));
        let tax_repo = Arc::new(SqliteTaxRepository::new(db.clone()));
        let bookmark_repo = Arc::new(SqliteBookmarkRepository::new(db.clone()));
        let template_repo = Arc::new(SqliteTemplateRepository::new(db.clone()));
        let invoice_repo = Arc::new(SqliteInvoiceRepository::new(db.clone()));
        let payment_repo = Arc::new(SqlitePaymentRepository::new(db.clone()));
        let accounting_repo = Arc::new(SqliteAccountingRepository::new(db.clone()));
        let search_repo = Arc::new(SqliteSearchRepository::new(db.clone()));
        let notebook_section_repo = Arc::new(SqliteNotebookSectionRepository::new(db.clone()));
        let client_notebook_repo = Arc::new(SqliteClientNotebookRepository::new(db.clone()));
        let client_journal_repo = Arc::new(SqliteClientJournalRepository::new(db.clone()));
        let email_template_repo = Arc::new(SqliteEmailTemplateRepository::new(db.clone()));
        let email_log_repo = Arc::new(SqliteEmailLogRepository::new(db.clone()));
        let audit_repo: Arc<dyn AuditRepository> =
            Arc::new(SqliteAuditRepository::new(db.clone()));
        let number_gen = Arc::new(SqliteInvoiceNumberGenerator::new(db.clone()));
        let pdf = Arc::new(TypstPdfGenerator::new());
        let pdf_storage = Arc::new(FilesystemPdfStorage::new(
            settings_repo.clone(),
            default_pdf_dir,
        ));
        // Per-org SMTP credential entry. The keyring "user" includes the org
        // code so different orgs each get their own slot in the OS keychain —
        // setting an SMTP password in one org never affects another.
        let credentials = Arc::new(KeyringCredentialStore::new(
            "terative",
            format!("smtp:{}", code.as_str()),
        ));
        let email_sender = Arc::new(LettreEmailSender::new());
        let data_management: Arc<dyn DataManagement> = Arc::new(FilesystemDataManagement::new(
            db.clone(),
            db_path,
            settings_repo.clone(),
            user_backup_dir.clone(),
            system_backup_dir.clone(),
            key,
        ));

        // The audit-log event bus. `register_all` wires every domain-event
        // handler against it; mutating use cases get this bus via
        // `.with_events(..)` so their `commit()` calls land in the log.
        let mut bus = InProcessEventBus::new();
        register_all(
            &mut bus,
            AuditHandlerContext {
                audits: audit_repo.clone(),
                invoices: invoice_repo.clone(),
                clients: client_repo.clone(),
                catalog_items: catalog_item_repo.clone(),
                taxes: tax_repo.clone(),
            },
        );
        let events: Arc<dyn EventBus> = Arc::new(bus);

        #[cfg(debug_assertions)]
        let seed_database = SeedDatabase::new(
            CreateClient::new(client_repo.clone()).with_events(events.clone()),
            CreateCatalogItem::new(catalog_item_repo.clone()).with_events(events.clone()),
            CreateTax::new(tax_repo.clone()).with_events(events.clone()),
            CreateBookmark::new(bookmark_repo.clone()),
            CreateDraftInvoice::new(invoice_repo.clone(), tax_repo.clone())
                .with_events(events.clone()),
            FinalizeInvoice::new(
                invoice_repo.clone(),
                number_gen.clone(),
                template_repo.clone(),
                settings_repo.clone(),
                client_repo.clone(),
                pdf.clone(),
                pdf_storage.clone(),
            )
            .with_events(events.clone()),
            CancelInvoice::new(
                invoice_repo.clone(),
                client_repo.clone(),
                template_repo.clone(),
                settings_repo.clone(),
                pdf.clone(),
                pdf_storage.clone(),
            )
            .with_events(events.clone()),
            RecordPayment::new(payment_repo.clone(), invoice_repo.clone())
                .with_events(events.clone()),
            CreateJournalEntry::new(client_journal_repo.clone()),
            invoice_repo.clone(),
            client_repo.clone(),
            email_log_repo.clone(),
        );

        Self {
            code,
            user_backup_dir,
            system_backup_dir,

            create_client: CreateClient::new(client_repo.clone())
                .with_events(events.clone()),
            update_client: UpdateClient::new(client_repo.clone())
                .with_events(events.clone()),
            archive_client: ArchiveClient::new(client_repo.clone())
                .with_events(events.clone()),
            unarchive_client: UnarchiveClient::new(client_repo.clone())
                .with_events(events.clone()),
            list_clients: ListClients::new(client_repo.clone()),
            get_client_detail: GetClientDetail::new(client_repo.clone()),
            list_client_attribute_values: ListClientAttributeValues::new(client_repo.clone()),

            create_catalog_item: CreateCatalogItem::new(catalog_item_repo.clone())
                .with_events(events.clone()),
            update_catalog_item: UpdateCatalogItem::new(catalog_item_repo.clone())
                .with_events(events.clone()),
            archive_catalog_item: ArchiveCatalogItem::new(catalog_item_repo.clone()),
            unarchive_catalog_item: UnarchiveCatalogItem::new(catalog_item_repo.clone()),
            list_catalog_items: ListCatalogItems::new(catalog_item_repo),

            get_settings: GetSettings::new(settings_repo.clone(), credentials.clone()),
            update_seller_profile: UpdateSellerProfile::new(settings_repo.clone()),
            update_currency: UpdateCurrency::new(settings_repo.clone()),
            update_app_preferences: UpdateAppPreferences::new(settings_repo.clone()),

            create_tax: CreateTax::new(tax_repo.clone()).with_events(events.clone()),
            update_tax: UpdateTax::new(tax_repo.clone()).with_events(events.clone()),
            archive_tax: ArchiveTax::new(tax_repo.clone()),
            unarchive_tax: UnarchiveTax::new(tax_repo.clone()),
            list_taxes: ListTaxes::new(tax_repo.clone()),

            create_bookmark: CreateBookmark::new(bookmark_repo.clone()),
            update_bookmark: UpdateBookmark::new(bookmark_repo.clone()),
            delete_bookmark: DeleteBookmark::new(bookmark_repo.clone()),
            list_bookmarks: ListBookmarks::new(bookmark_repo.clone()),
            reorder_bookmarks: ReorderBookmarks::new(bookmark_repo),

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

            create_draft_invoice: CreateDraftInvoice::new(invoice_repo.clone(), tax_repo.clone())
                .with_events(events.clone()),
            update_draft_invoice: UpdateDraftInvoice::new(invoice_repo.clone(), tax_repo.clone())
                .with_events(events.clone()),
            finalize_invoice: FinalizeInvoice::new(
                invoice_repo.clone(),
                number_gen.clone(),
                template_repo.clone(),
                settings_repo.clone(),
                client_repo.clone(),
                pdf.clone(),
                pdf_storage.clone(),
            )
            .with_events(events.clone()),
            duplicate_invoice: DuplicateInvoice::new(invoice_repo.clone())
                .with_events(events.clone()),
            cancel_invoice: CancelInvoice::new(
                invoice_repo.clone(),
                client_repo.clone(),
                template_repo,
                settings_repo.clone(),
                pdf,
                pdf_storage.clone(),
            )
            .with_events(events.clone()),
            list_invoices: ListInvoices::new(
                invoice_repo.clone(),
                payment_repo.clone(),
                client_repo.clone(),
                email_log_repo.clone(),
            ),
            get_invoice: GetInvoice::new(
                invoice_repo.clone(),
                payment_repo.clone(),
                client_repo.clone(),
                email_log_repo.clone(),
            ),
            get_invoice_pdf: GetInvoicePdf::new(invoice_repo.clone(), pdf_storage),
            print_invoice: PrintInvoice::new(invoice_repo.clone()),
            open_invoice_externally: OpenInvoiceExternally::new(invoice_repo.clone()),
            get_invoice_numbering: GetInvoiceNumbering::new(
                number_gen.clone(),
                invoice_repo.clone(),
            ),
            set_starting_invoice_number: SetStartingInvoiceNumber::new(
                number_gen,
                invoice_repo.clone(),
            ),

            update_email_config: UpdateEmailConfig::new(settings_repo.clone()),
            update_email_password: UpdateEmailPassword::new(credentials.clone()),
            test_email_connection: TestEmailConnection::new(
                settings_repo.clone(),
                credentials.clone(),
                email_sender.clone(),
            ),
            send_invoice: SendInvoice::new(
                invoice_repo.clone(),
                client_repo.clone(),
                settings_repo,
                credentials,
                email_sender,
                email_template_repo.clone(),
                email_log_repo.clone(),
            )
            .with_events(events.clone()),

            list_email_logs_for_client: ListEmailLogsForClient::new(email_log_repo),

            create_email_template: CreateEmailTemplate::new(email_template_repo.clone()),
            update_email_template: UpdateEmailTemplate::new(email_template_repo.clone()),
            delete_email_template: DeleteEmailTemplate::new(email_template_repo.clone()),
            set_default_email_template: SetDefaultEmailTemplate::new(email_template_repo.clone()),
            list_email_templates: ListEmailTemplates::new(email_template_repo),

            record_payment: RecordPayment::new(payment_repo.clone(), invoice_repo.clone())
                .with_events(events.clone()),
            update_payment: UpdatePayment::new(payment_repo.clone(), invoice_repo.clone())
                .with_events(events.clone()),
            delete_payment: DeletePayment::new(payment_repo.clone())
                .with_events(events.clone()),
            list_payments: ListPayments::new(payment_repo.clone(), client_repo.clone()),
            get_payment: GetPayment::new(payment_repo, client_repo),

            accounting: AccountingService::new(accounting_repo),

            global_search: GlobalSearch::new(search_repo),

            create_backup: CreateBackup::new(data_management.clone())
                .with_events(events.clone()),
            auto_backup_if_due: AutoBackupIfDue::new(data_management.clone())
                .with_events(events.clone()),
            data_management,

            paginate_recent_audit: PaginateRecentAudit::new(audit_repo.clone()),
            paginate_audit_for_client: PaginateAuditForClient::new(audit_repo.clone()),
            paginate_audit_for_invoice: PaginateAuditForInvoice::new(audit_repo.clone()),
            cleanup_audits: CleanupAudits::new(audit_repo),

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

            #[cfg(debug_assertions)]
            seed_database,
        }
    }
}

/// Permanent app context, registered once at startup. Holds the always-
/// available `OrgRegistry`, the `OrgKeyStore` (OS keyring adapter), and a
/// swappable `OrgServices` for the active org. Commands access org-scoped
/// use cases via `state.org()?`.
pub struct AppState {
    pub org_registry: Arc<OrgRegistry>,
    pub org_key_store: Arc<dyn OrgKeyStore>,
    active: ArcSwap<Option<Arc<OrgServices>>>,
}

impl AppState {
    pub fn new(
        org_registry: Arc<OrgRegistry>,
        org_key_store: Arc<dyn OrgKeyStore>,
    ) -> Self {
        Self {
            org_registry,
            org_key_store,
            active: ArcSwap::new(Arc::new(None)),
        }
    }

    /// Returns the active org's services or `NoActiveOrg`. Cheap — clones
    /// an `Arc` from a lock-free `ArcSwap` snapshot.
    pub fn org(&self) -> Result<Arc<OrgServices>, AppError> {
        (**self.active.load())
            .clone()
            .ok_or_else(AppError::no_active_org)
    }

    /// Atomically swap in a freshly-built `OrgServices`. Drops the previous
    /// one once any in-flight command holding it finishes.
    pub fn open_org(&self, services: OrgServices) {
        self.active.store(Arc::new(Some(Arc::new(services))));
    }

    /// Drop the active org. New commands fail with `NoActiveOrg` until
    /// `open_org` is called again.
    pub fn close_org(&self) {
        self.active.store(Arc::new(None));
    }

    /// Currently active org's code, if any. Used by the auto-backup ticker
    /// and by `org_get_active`.
    pub fn active_code(&self) -> Option<OrgCode> {
        self.active
            .load()
            .as_ref()
            .as_ref()
            .map(|s| s.code.clone())
    }
}
