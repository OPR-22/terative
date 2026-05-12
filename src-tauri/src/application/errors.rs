use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::bookmark::BookmarkError;
use crate::domain::catalog_item::CatalogItemError;
use crate::domain::client::ClientError;
use crate::domain::email_log::EmailLogError;
use crate::domain::email_template::EmailTemplateError;
use crate::domain::invoice::InvoiceError;
use crate::domain::line_item::LineItemError;
use crate::domain::money::MoneyError;
use crate::domain::notebook::{JournalEntryError, NotebookError, NotebookSectionError};
use crate::domain::payment::PaymentError;
use crate::domain::settings::{CurrencyConfigError, EmailConfigError};
use crate::domain::tax::TaxError;
use crate::domain::template::TemplateError;

use super::ports::{EmailError, PdfError};

/// Stable, typed identifier for every code-bearing error.
///
/// These codes accompany an `AppError::InvalidArgument` / `NotFound` /
/// `AlreadyExists` / `FailedPrecondition`. They are the i18n keys used by
/// the frontend's `errorCatalog`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // ── Generic ──
    ResourceNotFound,

    // ── Org lifecycle ──
    NoActiveOrg,
    OrgNotFound,
    OrgCodeAlreadyExists,
    InvalidOrgCode,

    // ── Bookmark ──
    BookmarkEmptyLabel,
    BookmarkEmptyUrl,
    BookmarkInvalidUrl,
    BookmarkUnsupportedScheme,

    // ── Catalog item ──
    CatalogItemEmptyName,
    CatalogItemNegativePrice,
    CatalogItemDuplicateCurrency,

    // ── Client ──
    ClientEmptyName,
    ClientEmptyContactValue,
    ClientEmptyAddressStreet,
    ClientEmptyAddressCity,
    ClientEmptyAddressPostalCode,
    ClientEmptyAddressCountry,
    ClientDuplicateBillingAddress,
    ClientDuplicateShippingAddress,
    ClientSelfReferral,
    ClientFutureDateOfBirth,
    ClientHasInvoices,

    // ── Currency ──
    CurrencyUnsupported,

    // ── DTO conversion ──
    DtoConvert,

    // ── Email config ──
    EmailConfigEmptyHost,
    EmailConfigInvalidPort,
    EmailConfigEmptySender,
    EmailConfigInvalidSender,

    // ── Email log ──
    EmailLogEmptyRecipient,
    EmailLogEmptySubject,

    // ── Email template ──
    EmailTemplateEmptyName,
    EmailTemplateEmptySubject,
    EmailTemplateEmptyBody,
    EmailTemplateNoDefault,
    EmailTemplateIsDefault,

    // ── Invoice ──
    InvoiceNoLineItems,
    InvoiceNotDraft,
    InvoiceCannotCancelDraft,
    InvoiceAlreadyCancelled,
    InvoiceNotFinalized,
    InvoiceNotSendable,
    InvoiceOverAllocated,
    InvoiceAllocationCurrencyMismatch,
    InvoiceNotAllocatable,
    InvoiceMissingPdf,

    // ── Journal entry ──
    JournalEntryEmptyContent,

    // ── Line item ──
    LineItemEmptyDescription,
    LineItemNonPositiveQuantity,
    LineItemNegativeUnitPrice,

    // ── Money ──
    MoneyUnsupportedCurrency,
    MoneyCurrencyMismatch,
    MoneyOverflow,

    // ── Notebook ──
    NotebookDuplicateSection,
    NotebookSectionEmptyName,

    // ── Payment ──
    PaymentNonPositiveAmount,
    PaymentNonPositiveAllocation,
    PaymentAllocationsExceedPayment,
    PaymentCurrencyMismatch,
    PaymentInvoiceCurrencyMismatch,
    PaymentDuplicateAllocation,

    // ── SMTP ──
    SmtpPasswordMissing,

    // ── Tax ──
    TaxEmptyName,
    TaxNegativePercentage,

    // ── Template ──
    TemplateEmptyName,
    TemplateInvalidAccentColor,
    TemplateInUse,
}

/// Wire-format error returned by every Tauri command.
///
/// Modelled on gRPC's status codes — a small, fixed set of categories that
/// determines how the frontend should react (highlight a form field, toast,
/// redirect, etc.). Code-bearing variants carry a stable `ErrorCode` for
/// i18n lookup; `Internal` and `Unknown` carry a free-form detail string
/// because their content is implementation-specific.
#[derive(Debug, Clone, Serialize, specta::Type, thiserror::Error)]
#[serde(tag = "status")]
pub enum AppError {
    /// Client supplied invalid input. Frontend renders next to the form
    /// field. Maps to gRPC `INVALID_ARGUMENT`.
    #[error("invalid argument: {code:?}")]
    InvalidArgument {
        code: ErrorCode,
        params: Option<HashMap<String, String>>,
    },

    /// Requested entity does not exist. Maps to gRPC `NOT_FOUND`.
    #[error("not found: {code:?}")]
    NotFound {
        code: ErrorCode,
        params: Option<HashMap<String, String>>,
    },

    /// Resource already exists / unique-constraint violation.
    /// Maps to gRPC `ALREADY_EXISTS`.
    #[error("already exists: {code:?}")]
    AlreadyExists {
        code: ErrorCode,
        params: Option<HashMap<String, String>>,
    },

    /// System is not in the state required for this op — covers domain
    /// state-machine violations (invoice not draft, over-allocated payment)
    /// and control-flow signals (`NoActiveOrg`). Maps to gRPC
    /// `FAILED_PRECONDITION`.
    #[error("failed precondition: {code:?}")]
    FailedPrecondition {
        code: ErrorCode,
        params: Option<HashMap<String, String>>,
    },

    /// Authentication missing or invalid. Reserved for T03 (wrong org
    /// password). Maps to gRPC `UNAUTHENTICATED`.
    #[error("unauthenticated")]
    Unauthenticated,

    /// Internal infrastructure failure (I/O, db, pdf, smtp, etc.). Not
    /// translated — frontend toasts the detail. Maps to gRPC `INTERNAL`.
    #[error("internal: {detail}")]
    Internal { detail: String },

    /// Unmapped foreign error or a truly unexpected condition. Last-resort
    /// catch-all. Maps to gRPC `UNKNOWN`.
    #[error("unknown: {detail}")]
    Unknown { detail: String },
}

impl AppError {
    // ── Constructors ──

    pub fn invalid_argument(code: ErrorCode) -> Self {
        AppError::InvalidArgument { code, params: None }
    }

    pub fn not_found(code: ErrorCode) -> Self {
        AppError::NotFound { code, params: None }
    }

    pub fn already_exists(code: ErrorCode) -> Self {
        AppError::AlreadyExists { code, params: None }
    }

    pub fn failed_precondition(code: ErrorCode) -> Self {
        AppError::FailedPrecondition { code, params: None }
    }

    pub fn with_params(self, params: HashMap<String, String>) -> Self {
        match self {
            AppError::InvalidArgument { code, .. } => AppError::InvalidArgument {
                code,
                params: Some(params),
            },
            AppError::NotFound { code, .. } => AppError::NotFound {
                code,
                params: Some(params),
            },
            AppError::AlreadyExists { code, .. } => AppError::AlreadyExists {
                code,
                params: Some(params),
            },
            AppError::FailedPrecondition { code, .. } => AppError::FailedPrecondition {
                code,
                params: Some(params),
            },
            other => other,
        }
    }

    pub fn no_active_org() -> Self {
        AppError::failed_precondition(ErrorCode::NoActiveOrg)
    }

    pub fn resource_not_found() -> Self {
        AppError::not_found(ErrorCode::ResourceNotFound)
    }

    pub fn org_not_found(code: impl Into<String>) -> Self {
        let mut p = HashMap::new();
        p.insert("code".into(), code.into());
        AppError::not_found(ErrorCode::OrgNotFound).with_params(p)
    }

    pub fn org_code_already_exists(code: impl Into<String>) -> Self {
        let mut p = HashMap::new();
        p.insert("code".into(), code.into());
        AppError::already_exists(ErrorCode::OrgCodeAlreadyExists).with_params(p)
    }

    pub fn invalid_org_code(reason: impl Into<String>) -> Self {
        let mut p = HashMap::new();
        p.insert("reason".into(), reason.into());
        AppError::invalid_argument(ErrorCode::InvalidOrgCode).with_params(p)
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        AppError::Internal {
            detail: detail.into(),
        }
    }

    pub fn unknown(detail: impl Into<String>) -> Self {
        AppError::Unknown {
            detail: detail.into(),
        }
    }

    // ── Predicates ──

    /// True when the error has the given code, regardless of category.
    pub fn is(&self, expected: ErrorCode) -> bool {
        matches!(
            self,
            AppError::InvalidArgument { code, .. }
            | AppError::NotFound { code, .. }
            | AppError::AlreadyExists { code, .. }
            | AppError::FailedPrecondition { code, .. }
            if *code == expected
        )
    }
}

// ─────────────────────────────────────────────────────────────────────
//  Repository error
// ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("entity not found")]
    NotFound,
    #[error("constraint violation: {0}")]
    Conflict(String),
    #[error("storage error: {0}")]
    Storage(String),
}

impl From<RepoError> for AppError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::NotFound => AppError::resource_not_found(),
            RepoError::Conflict(detail) => AppError::Internal { detail },
            RepoError::Storage(detail) => AppError::Internal { detail },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
//  Domain → AppError mappings
// ─────────────────────────────────────────────────────────────────────

impl From<BookmarkError> for AppError {
    fn from(e: BookmarkError) -> Self {
        let code = match e {
            BookmarkError::EmptyLabel => ErrorCode::BookmarkEmptyLabel,
            BookmarkError::EmptyUrl => ErrorCode::BookmarkEmptyUrl,
            BookmarkError::InvalidUrl => ErrorCode::BookmarkInvalidUrl,
            BookmarkError::UnsupportedScheme => ErrorCode::BookmarkUnsupportedScheme,
        };
        AppError::invalid_argument(code)
    }
}

impl From<CatalogItemError> for AppError {
    fn from(e: CatalogItemError) -> Self {
        match e {
            CatalogItemError::EmptyName => {
                AppError::invalid_argument(ErrorCode::CatalogItemEmptyName)
            }
            CatalogItemError::NegativePrice => {
                AppError::invalid_argument(ErrorCode::CatalogItemNegativePrice)
            }
            CatalogItemError::DuplicateCurrency => {
                AppError::invalid_argument(ErrorCode::CatalogItemDuplicateCurrency)
            }
            CatalogItemError::Money(inner) => inner.into(),
        }
    }
}

impl From<ClientError> for AppError {
    fn from(e: ClientError) -> Self {
        let code = match e {
            ClientError::EmptyName => ErrorCode::ClientEmptyName,
            ClientError::EmptyContactValue => ErrorCode::ClientEmptyContactValue,
            ClientError::EmptyAddressStreet => ErrorCode::ClientEmptyAddressStreet,
            ClientError::EmptyAddressCity => ErrorCode::ClientEmptyAddressCity,
            ClientError::EmptyAddressPostalCode => ErrorCode::ClientEmptyAddressPostalCode,
            ClientError::EmptyAddressCountry => ErrorCode::ClientEmptyAddressCountry,
            ClientError::DuplicateBillingAddress => ErrorCode::ClientDuplicateBillingAddress,
            ClientError::DuplicateShippingAddress => ErrorCode::ClientDuplicateShippingAddress,
            ClientError::SelfReferral => ErrorCode::ClientSelfReferral,
            ClientError::FutureDateOfBirth => ErrorCode::ClientFutureDateOfBirth,
        };
        AppError::invalid_argument(code)
    }
}

impl From<EmailLogError> for AppError {
    fn from(e: EmailLogError) -> Self {
        let code = match e {
            EmailLogError::EmptyRecipient => ErrorCode::EmailLogEmptyRecipient,
            EmailLogError::EmptySubject => ErrorCode::EmailLogEmptySubject,
        };
        AppError::invalid_argument(code)
    }
}

impl From<EmailTemplateError> for AppError {
    fn from(e: EmailTemplateError) -> Self {
        let code = match e {
            EmailTemplateError::EmptyName => ErrorCode::EmailTemplateEmptyName,
            EmailTemplateError::EmptySubject => ErrorCode::EmailTemplateEmptySubject,
            EmailTemplateError::EmptyBody => ErrorCode::EmailTemplateEmptyBody,
        };
        AppError::invalid_argument(code)
    }
}

impl From<InvoiceError> for AppError {
    fn from(e: InvoiceError) -> Self {
        match e {
            InvoiceError::NoLineItems => AppError::failed_precondition(ErrorCode::InvoiceNoLineItems),
            InvoiceError::NotDraft => AppError::failed_precondition(ErrorCode::InvoiceNotDraft),
            InvoiceError::CannotCancelDraft => {
                AppError::failed_precondition(ErrorCode::InvoiceCannotCancelDraft)
            }
            InvoiceError::AlreadyCancelled => {
                AppError::failed_precondition(ErrorCode::InvoiceAlreadyCancelled)
            }
            InvoiceError::NotFinalized => {
                AppError::failed_precondition(ErrorCode::InvoiceNotFinalized)
            }
            InvoiceError::NotSendable => {
                AppError::failed_precondition(ErrorCode::InvoiceNotSendable)
            }
            InvoiceError::OverAllocated => {
                AppError::failed_precondition(ErrorCode::InvoiceOverAllocated)
            }
            InvoiceError::AllocationCurrencyMismatch => {
                AppError::failed_precondition(ErrorCode::InvoiceAllocationCurrencyMismatch)
            }
            InvoiceError::NotAllocatable(status) => {
                let mut params = HashMap::new();
                params.insert("status".into(), format!("{status:?}"));
                AppError::failed_precondition(ErrorCode::InvoiceNotAllocatable).with_params(params)
            }
            InvoiceError::LineItem(inner) => inner.into(),
            InvoiceError::Money(inner) => inner.into(),
        }
    }
}

impl From<LineItemError> for AppError {
    fn from(e: LineItemError) -> Self {
        match e {
            LineItemError::EmptyDescription => {
                AppError::invalid_argument(ErrorCode::LineItemEmptyDescription)
            }
            LineItemError::NonPositiveQuantity => {
                AppError::invalid_argument(ErrorCode::LineItemNonPositiveQuantity)
            }
            LineItemError::NegativeUnitPrice => {
                AppError::invalid_argument(ErrorCode::LineItemNegativeUnitPrice)
            }
            LineItemError::Money(inner) => inner.into(),
        }
    }
}

impl From<MoneyError> for AppError {
    fn from(e: MoneyError) -> Self {
        match e {
            MoneyError::UnsupportedCurrency(code) => {
                let mut params = HashMap::new();
                params.insert("currency".into(), code);
                AppError::failed_precondition(ErrorCode::MoneyUnsupportedCurrency)
                    .with_params(params)
            }
            MoneyError::CurrencyMismatch { left, right } => {
                let mut params = HashMap::new();
                params.insert("left".into(), left);
                params.insert("right".into(), right);
                AppError::failed_precondition(ErrorCode::MoneyCurrencyMismatch).with_params(params)
            }
            MoneyError::Overflow => AppError::failed_precondition(ErrorCode::MoneyOverflow),
        }
    }
}

impl From<CurrencyConfigError> for AppError {
    fn from(_e: CurrencyConfigError) -> Self {
        AppError::invalid_argument(ErrorCode::CurrencyUnsupported)
    }
}

impl From<EmailConfigError> for AppError {
    fn from(e: EmailConfigError) -> Self {
        let code = match e {
            EmailConfigError::EmptyHost => ErrorCode::EmailConfigEmptyHost,
            EmailConfigError::InvalidPort => ErrorCode::EmailConfigInvalidPort,
            EmailConfigError::EmptySender => ErrorCode::EmailConfigEmptySender,
            EmailConfigError::InvalidSender => ErrorCode::EmailConfigInvalidSender,
        };
        AppError::invalid_argument(code)
    }
}

impl From<NotebookSectionError> for AppError {
    fn from(e: NotebookSectionError) -> Self {
        let code = match e {
            NotebookSectionError::EmptyName => ErrorCode::NotebookSectionEmptyName,
        };
        AppError::invalid_argument(code)
    }
}

impl From<NotebookError> for AppError {
    fn from(e: NotebookError) -> Self {
        let code = match e {
            NotebookError::DuplicateSection => ErrorCode::NotebookDuplicateSection,
        };
        AppError::failed_precondition(code)
    }
}

impl From<JournalEntryError> for AppError {
    fn from(e: JournalEntryError) -> Self {
        let code = match e {
            JournalEntryError::EmptyContent => ErrorCode::JournalEntryEmptyContent,
        };
        AppError::invalid_argument(code)
    }
}

impl From<PaymentError> for AppError {
    fn from(e: PaymentError) -> Self {
        match e {
            PaymentError::NonPositiveAmount => {
                AppError::invalid_argument(ErrorCode::PaymentNonPositiveAmount)
            }
            PaymentError::NonPositiveAllocation => {
                AppError::invalid_argument(ErrorCode::PaymentNonPositiveAllocation)
            }
            PaymentError::AllocationsExceedPayment => {
                AppError::failed_precondition(ErrorCode::PaymentAllocationsExceedPayment)
            }
            PaymentError::CurrencyMismatch => {
                AppError::failed_precondition(ErrorCode::PaymentCurrencyMismatch)
            }
            PaymentError::DuplicateAllocation => {
                AppError::failed_precondition(ErrorCode::PaymentDuplicateAllocation)
            }
            PaymentError::Money(inner) => inner.into(),
        }
    }
}

impl From<TaxError> for AppError {
    fn from(e: TaxError) -> Self {
        let code = match e {
            TaxError::EmptyName => ErrorCode::TaxEmptyName,
            TaxError::NegativePercentage => ErrorCode::TaxNegativePercentage,
        };
        AppError::invalid_argument(code)
    }
}

impl From<TemplateError> for AppError {
    fn from(e: TemplateError) -> Self {
        let code = match e {
            TemplateError::EmptyName => ErrorCode::TemplateEmptyName,
            TemplateError::InvalidAccentColor => ErrorCode::TemplateInvalidAccentColor,
        };
        AppError::invalid_argument(code)
    }
}

// ── Infrastructure ports ───────────────────────────────────────────

impl From<EmailError> for AppError {
    fn from(e: EmailError) -> Self {
        AppError::Internal {
            detail: e.to_string(),
        }
    }
}

impl From<PdfError> for AppError {
    fn from(e: PdfError) -> Self {
        AppError::Internal {
            detail: e.to_string(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal {
            detail: e.to_string(),
        }
    }
}

/// Convenience: `?` on `Result<_, String>` from foreign APIs (notably tauri
/// window/webview methods) maps to `AppError::Unknown`.
impl From<String> for AppError {
    fn from(detail: String) -> Self {
        AppError::Unknown { detail }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invalid_argument_serializes_with_status_and_code() {
        let v = serde_json::to_value(AppError::invalid_argument(ErrorCode::ClientEmptyName))
            .unwrap();
        assert_eq!(
            v,
            json!({
                "status": "InvalidArgument",
                "code": "client_empty_name",
                "params": null
            })
        );
    }

    #[test]
    fn not_found_with_params_serializes_correctly() {
        let v = serde_json::to_value(AppError::org_not_found("acme")).unwrap();
        assert_eq!(
            v,
            json!({
                "status": "NotFound",
                "code": "org_not_found",
                "params": { "code": "acme" }
            })
        );
    }

    #[test]
    fn no_active_org_is_failed_precondition() {
        let v = serde_json::to_value(AppError::no_active_org()).unwrap();
        assert_eq!(
            v,
            json!({
                "status": "FailedPrecondition",
                "code": "no_active_org",
                "params": null
            })
        );
    }

    #[test]
    fn internal_serializes_with_detail() {
        let v = serde_json::to_value(AppError::internal("oops")).unwrap();
        assert_eq!(
            v,
            json!({ "status": "Internal", "detail": "oops" })
        );
    }

    #[test]
    fn unauthenticated_is_unit_shape() {
        let v = serde_json::to_value(AppError::Unauthenticated).unwrap();
        assert_eq!(v, json!({ "status": "Unauthenticated" }));
    }

    #[test]
    fn bookmark_error_maps_to_invalid_argument() {
        let app: AppError = BookmarkError::EmptyLabel.into();
        assert!(app.is(ErrorCode::BookmarkEmptyLabel));
        assert!(matches!(app, AppError::InvalidArgument { .. }));
    }

    #[test]
    fn invoice_error_maps_to_failed_precondition() {
        let app: AppError = InvoiceError::OverAllocated.into();
        assert!(app.is(ErrorCode::InvoiceOverAllocated));
        assert!(matches!(app, AppError::FailedPrecondition { .. }));
    }
}
