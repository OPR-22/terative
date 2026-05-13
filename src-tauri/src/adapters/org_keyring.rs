//! Per-org database key storage backed by the OS keyring.
//!
//! Each org's SQLCipher passphrase lives at
//! `service="terative"` / `account="dbkey:<code>"`. Presence of an entry
//! means the user has opted into auto-unlock for that org; absence means
//! the next `org_open` must prompt.

use keyring::Entry;

use crate::application::ports::OrgKeyStore;
use crate::application::RepoError;

const SERVICE: &str = "terative";

pub struct KeyringOrgKeyStore;

impl KeyringOrgKeyStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(code: &str) -> Result<Entry, keyring::Error> {
        Entry::new(SERVICE, &format!("dbkey:{code}"))
    }
}

impl Default for KeyringOrgKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

fn map_err(e: keyring::Error, op: &str) -> RepoError {
    RepoError::Storage(format!("keyring {op}: {e}"))
}

impl OrgKeyStore for KeyringOrgKeyStore {
    fn get(&self, code: &str) -> Result<Option<String>, RepoError> {
        match Self::entry(code).map(|e| e.get_password()) {
            Ok(Ok(p)) => Ok(Some(p)),
            Ok(Err(keyring::Error::NoEntry)) => Ok(None),
            Ok(Err(e)) => Err(map_err(e, "read")),
            Err(e) => Err(map_err(e, "read")),
        }
    }

    fn set(&self, code: &str, password: &str) -> Result<(), RepoError> {
        Self::entry(code)
            .map_err(|e| map_err(e, "write"))?
            .set_password(password)
            .map_err(|e| map_err(e, "write"))
    }

    fn delete(&self, code: &str) -> Result<(), RepoError> {
        match Self::entry(code).map(|e| e.delete_credential()) {
            Ok(Ok(())) | Ok(Err(keyring::Error::NoEntry)) => Ok(()),
            Ok(Err(e)) => Err(map_err(e, "delete")),
            Err(e) => Err(map_err(e, "delete")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_code() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("test-{nanos}")
    }

    /// Hits the real OS keystore; ignored so batch runs don't pollute the
    /// user's keychain. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn org_keyring_round_trip_against_real_keystore() {
        let store = KeyringOrgKeyStore::new();
        let code = unique_code();

        assert!(store.get(&code).unwrap().is_none());
        store.set(&code, "hunter2").unwrap();
        assert_eq!(store.get(&code).unwrap().as_deref(), Some("hunter2"));

        store.set(&code, "hunter3").unwrap();
        assert_eq!(store.get(&code).unwrap().as_deref(), Some("hunter3"));

        store.delete(&code).unwrap();
        assert!(store.get(&code).unwrap().is_none());
        store.delete(&code).unwrap();
    }
}
