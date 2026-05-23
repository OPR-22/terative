use crate::application::RepoError;
use crate::domain::invoice::InvoiceNumber;

pub trait InvoiceNumberGenerator: Send + Sync {
    /// Hands out the next number and advances the sequence. Called once per
    /// invoice finalize.
    fn next(&self) -> Result<InvoiceNumber, RepoError>;

    /// The number the next finalized invoice will receive, without consuming
    /// it. Used to surface the current starting point in settings.
    fn peek(&self) -> Result<InvoiceNumber, RepoError>;

    /// Overrides the number the next finalized invoice will receive. Used to
    /// configure the sequence's starting point before the first invoice is
    /// finalized; callers are responsible for enforcing that precondition.
    fn set_next(&self, next: InvoiceNumber) -> Result<(), RepoError>;
}
