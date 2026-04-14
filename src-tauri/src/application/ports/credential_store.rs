use crate::application::RepoError;

/// Stores secrets outside the main app database (OS keystore in production).
///
/// All methods must be idempotent: `set` overwrites, `delete` succeeds even if
/// the entry is missing, `has` returns false rather than erroring.
pub trait CredentialStore: Send + Sync {
    fn set_smtp_password(&self, password: &str) -> Result<(), RepoError>;
    fn get_smtp_password(&self) -> Result<Option<String>, RepoError>;
    fn has_smtp_password(&self) -> Result<bool, RepoError>;
    fn delete_smtp_password(&self) -> Result<(), RepoError>;
}
