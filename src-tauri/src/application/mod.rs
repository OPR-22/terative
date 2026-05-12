pub mod accounting_usecases;
pub mod bookmark_usecases;
pub mod catalog_item_usecases;
pub mod client_usecases;
pub mod dto;
pub mod email_log_usecases;
pub mod email_template_usecases;
pub mod email_usecases;
pub mod errors;
pub mod invoice_usecases;
pub mod notebook_usecases;
pub mod org_registry;
pub mod payment_usecases;
pub mod ports;
#[cfg(debug_assertions)]
pub mod seed_usecases;
pub mod settings_usecases;
pub mod tax_usecases;
pub mod template_usecases;

pub use errors::{AppError, ErrorCode, RepoError};
