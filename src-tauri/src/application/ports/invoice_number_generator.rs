use crate::application::RepoError;
use crate::domain::invoice::InvoiceNumber;

pub trait InvoiceNumberGenerator: Send + Sync {
    fn next(&self) -> Result<InvoiceNumber, RepoError>;
}
