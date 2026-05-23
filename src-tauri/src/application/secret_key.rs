//! Newtype wrapping a SQLCipher passphrase.
//!
//! Backed by `zeroize::Zeroizing<String>`, so the bytes are wiped from the
//! heap when the value is dropped. Cloneable so the same key can flow from
//! the command layer into both the open path and the cached
//! `FilesystemDataManagement.key` field without exposing the raw `String`.
//!
//! `Debug` is redacted; callers must use [`SecretKey::expose`] to obtain a
//! `&str` for the (single) point where it is fed to SQLCipher.

use std::fmt;

use zeroize::Zeroizing;

#[derive(Clone)]
pub struct SecretKey(Zeroizing<String>);

impl SecretKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    /// Borrow the raw passphrase. Do not clone the returned `&str` into a
    /// plain `String` — that copy would not be zeroized.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_returns_underlying_value() {
        let k = SecretKey::new("hunter2");
        assert_eq!(k.expose(), "hunter2");
    }

    #[test]
    fn debug_is_redacted() {
        let k = SecretKey::new("hunter2");
        assert_eq!(format!("{k:?}"), "SecretKey(***)");
        assert!(!format!("{k:?}").contains("hunter2"));
    }

    #[test]
    fn clone_preserves_value() {
        let k = SecretKey::new("hunter2");
        let c = k.clone();
        assert_eq!(c.expose(), "hunter2");
        drop(k);
        assert_eq!(c.expose(), "hunter2");
    }
}
