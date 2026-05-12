//! Org identity types.
//!
//! `OrgCode` is the immutable identifier the user picks at creation. The
//! folder `<app_data_dir>/orgs/<code>/` and the database file
//! `<code>.sqlite` both use it. The code is also the string shown in the
//! picker — no derivation, no un-mangling: what the user typed is what
//! appears on disk and in the UI.
//!
//! Validation rules: ASCII letters (any case), digits, `_`, `-`. No spaces,
//! no dots, no other characters. 1–50 chars. Cannot collide with Windows
//! reserved device names (case-insensitive).

use crate::application::AppError;
#[cfg(test)]
use crate::application::ErrorCode;

const MAX_CODE_LEN: usize = 50;

/// Filesystem-safe org identifier supplied by the user at creation time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgCode(String);

impl OrgCode {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(AppError::invalid_org_code("empty"));
        }
        if trimmed.chars().count() > MAX_CODE_LEN {
            return Err(AppError::invalid_org_code("too_long"));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            return Err(AppError::invalid_org_code("invalid_chars"));
        }
        if is_windows_reserved(&trimmed.to_ascii_lowercase()) {
            return Err(AppError::invalid_org_code("reserved"));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OrgCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_windows_reserved(s: &str) -> bool {
    matches!(
        s,
        "con" | "prn" | "aux" | "nul"
        | "com1" | "com2" | "com3" | "com4" | "com5"
        | "com6" | "com7" | "com8" | "com9"
        | "lpt1" | "lpt2" | "lpt3" | "lpt4" | "lpt5"
        | "lpt6" | "lpt7" | "lpt8" | "lpt9"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_lowercase_ascii() {
        let c = OrgCode::parse("acme").unwrap();
        assert_eq!(c.as_str(), "acme");
    }

    #[test]
    fn accepts_underscores_dashes_digits() {
        let c = OrgCode::parse("acme_corp-2025").unwrap();
        assert_eq!(c.as_str(), "acme_corp-2025");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let c = OrgCode::parse("  acme  ").unwrap();
        assert_eq!(c.as_str(), "acme");
    }

    #[test]
    fn rejects_empty() {
        let err = OrgCode::parse("").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn rejects_whitespace_only() {
        let err = OrgCode::parse("   ").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn accepts_mixed_case() {
        let c = OrgCode::parse("AcmeCorp_2025").unwrap();
        assert_eq!(c.as_str(), "AcmeCorp_2025");
    }

    #[test]
    fn rejects_spaces() {
        let err = OrgCode::parse("acme corp").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn rejects_dots() {
        let err = OrgCode::parse("acme.corp").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn rejects_slashes() {
        let err = OrgCode::parse("acme/corp").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn rejects_non_ascii() {
        let err = OrgCode::parse("société").unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn rejects_too_long() {
        let s = "a".repeat(51);
        let err = OrgCode::parse(&s).unwrap_err();
        assert!(err.is(ErrorCode::InvalidOrgCode));
    }

    #[test]
    fn accepts_max_length() {
        let s = "a".repeat(50);
        let c = OrgCode::parse(&s).unwrap();
        assert_eq!(c.as_str().len(), 50);
    }

    #[test]
    fn rejects_windows_reserved_names_case_insensitive() {
        for name in ["con", "CON", "Con", "prn", "AUX", "nul", "com1", "LPT9"] {
            let err = OrgCode::parse(name).unwrap_err();
            assert!(err.is(ErrorCode::InvalidOrgCode), "should reject '{name}'");
        }
    }
}
