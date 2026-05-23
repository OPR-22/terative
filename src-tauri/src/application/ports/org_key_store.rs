use crate::application::RepoError;

/// Per-org SQLCipher passphrase storage (typically backed by the OS keyring).
///
/// Presence of an entry for `code` means the user opted into auto-unlock for
/// that org; absence means the next `org_open` must prompt. Implementations
/// must be idempotent: `set` overwrites, `delete` succeeds even when no
/// entry exists.
pub trait OrgKeyStore: Send + Sync {
    fn get(&self, code: &str) -> Result<Option<String>, RepoError>;
    fn set(&self, code: &str, password: &str) -> Result<(), RepoError>;
    fn delete(&self, code: &str) -> Result<(), RepoError>;
}
