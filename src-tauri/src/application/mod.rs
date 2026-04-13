pub mod client_usecases;
pub mod ports;
pub mod service_usecases;
pub mod settings_usecases;

use crate::domain::client::ClientError;
use crate::domain::money::MoneyError;
use crate::domain::service::ServiceError;
use crate::domain::settings::CurrencyConfigError;

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
    Client(#[from] ClientError),
    #[error(transparent)]
    Service(#[from] ServiceError),
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error(transparent)]
    Currency(#[from] CurrencyConfigError),
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error("cannot delete client with existing invoices")]
    ClientHasInvoices,
    #[error("entity not found")]
    NotFound,
}
