use keyring::Entry;

use crate::application::ports::CredentialStore;
use crate::application::RepoError;

pub struct KeyringCredentialStore {
    service: String,
    smtp_user: String,
}

impl KeyringCredentialStore {
    pub fn new(service: impl Into<String>, smtp_user: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            smtp_user: smtp_user.into(),
        }
    }

    fn entry(&self) -> Result<Entry, RepoError> {
        Entry::new(&self.service, &self.smtp_user)
            .map_err(|e| RepoError::Storage(format!("keyring: {e}")))
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn set_smtp_password(&self, password: &str) -> Result<(), RepoError> {
        self.entry()?
            .set_password(password)
            .map_err(|e| RepoError::Storage(format!("keyring set: {e}")))
    }

    fn get_smtp_password(&self) -> Result<Option<String>, RepoError> {
        match self.entry()?.get_password() {
            Ok(p) => Ok(Some(p)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(RepoError::Storage(format!("keyring get: {e}"))),
        }
    }

    fn has_smtp_password(&self) -> Result<bool, RepoError> {
        Ok(self.get_smtp_password()?.is_some())
    }

    fn delete_smtp_password(&self) -> Result<(), RepoError> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(RepoError::Storage(format!("keyring delete: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_user() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("terative-test-{nanos}")
    }

    /// Hits the real OS keystore; ignored by default so CI/local batch runs
    /// don't pollute the user's keychain. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn round_trip_against_real_keystore() {
        let store = KeyringCredentialStore::new("terative-test", unique_user());
        assert!(!store.has_smtp_password().unwrap());
        store.set_smtp_password("hunter2").unwrap();
        assert!(store.has_smtp_password().unwrap());
        assert_eq!(
            store.get_smtp_password().unwrap().as_deref(),
            Some("hunter2")
        );
        store.delete_smtp_password().unwrap();
        assert!(!store.has_smtp_password().unwrap());
        // Deleting a missing entry must not error.
        store.delete_smtp_password().unwrap();
    }
}
