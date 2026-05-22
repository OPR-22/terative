//! DTO layer — all types that cross the IPC boundary live here.
//!
//! Rules:
//! - Every type is named `XxxDto`
//! - Every type derives `Serialize`, `Deserialize`, and `specta::Type`
//! - No domain types appear as field types; IDs are `Uuid`, enums are mirrored,
//!   value objects have their own DTO
//! - Each module provides `impl From<&Domain> for Dto` (infallible) and
//!   `impl TryFrom<Dto> for Domain` / helper fns for inputs (fallible when
//!   domain invariants must be enforced, e.g. ID parsing)
//! - The command layer is responsible for all DTO ↔ domain conversion.
//!   Use cases stay pure domain.

pub mod accounting;
pub mod audit;
pub mod bookmark;
pub mod catalog_item;
pub mod client;
pub mod common;
pub mod email_log;
pub mod email_template;
pub mod invoice;
pub mod notebook;
pub mod payment;
#[cfg(debug_assertions)]
pub mod seed;
pub mod settings;
pub mod tax;
pub mod template;

pub use accounting::{
    AgingBucketDto, AgingRowDto, ClientBalanceDto, DashboardSummaryDto,
    DerivedPaymentStatusDto, InvoicePaymentRowDto, RevenueBucketDto, RevenueByClientDto,
    RevenueByClientInputDto, RevenueByPeriodInputDto, RevenueGroupingDto,
};
pub use audit::AuditDto;
pub use client::{
    ClientAddressDto, ClientAttributeValuesDto, ClientDto, ClientKindDto, ContactEntryDto,
    ListClientsQueryDto, NewClientDto, UpdateClientDto,
};
pub use common::{MoneyDto, PageDto, PaginationParamsDto};
pub use email_log::EmailLogDto;
pub use email_template::{
    EmailTemplateDto, EmailTemplateTypeDto, NewEmailTemplateDto, UpdateEmailTemplateDto,
};
pub use invoice::{
    AppliedTaxDto, EmailSendDto, InvoiceDto, InvoiceNumberingDto, InvoiceStatusDto, LineItemDto,
    ListInvoicesQueryDto, NewInvoiceDto, NewLineItemDto, UpdateDraftInvoiceDto,
};
pub use notebook::{
    ClientJournalEntryDto, ClientNotebookSectionDto, ClientNotebookViewDto,
    NewJournalEntryDto, NotebookEntryDto, NotebookSectionDto, RenameNotebookSectionDto,
    SaveClientNotebookDto, UpdateJournalEntryDto,
};
pub use payment::{
    ListPaymentsQueryDto, NewPaymentAllocationDto, NewPaymentDto, PaymentAllocationDto,
    PaymentDto, PaymentMethodDto, UpdatePaymentDto,
};
#[cfg(debug_assertions)]
pub use seed::{SeedCountsDto, SeedReportDto};
pub use bookmark::{BookmarkDto, NewBookmarkDto, UpdateBookmarkDto};
pub use catalog_item::{
    CatalogItemDto, CatalogItemKindDto, NewCatalogItemDto, UpdateCatalogItemDto,
};
pub use settings::{
    AppPreferencesDto, CurrencyConfigDto, EmailConfigDto, LanguageDto, SellerProfileDto,
    SettingsSnapshotDto, ThemeDto,
};
pub use tax::{NewTaxDefinitionDto, TaxDefinitionDto, UpdateTaxDto};
pub use template::{
    FontChoiceDto, InvoiceTemplateDto, NewInvoiceTemplateDto, PreviewTemplateInputDto,
    TemplateLayoutDto, TemplateOverrideDto, UpdateTemplateDto,
};

/// Errors raised when converting a DTO into its domain counterpart. Distinct
/// from `AppError` so the DTO layer stays free of upstream error types.
#[derive(Debug, thiserror::Error)]
pub enum DtoConvertError {
    #[error("invalid uuid: {0}")]
    InvalidUuid(String),
    #[error("invalid decimal: {0}")]
    InvalidDecimal(String),
    #[error("invalid currency code: {0}")]
    InvalidCurrency(String),
    #[error("unknown enum variant: {0}")]
    UnknownVariant(String),
}

impl From<DtoConvertError> for crate::application::AppError {
    fn from(err: DtoConvertError) -> Self {
        let mut params = std::collections::HashMap::new();
        params.insert("detail".to_string(), err.to_string());
        crate::application::AppError::invalid_argument(
            crate::application::ErrorCode::DtoConvert,
        )
        .with_params(params)
    }
}
