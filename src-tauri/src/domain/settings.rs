use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub sender_address: String,
    pub subject_template: String,
    pub body_template: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmailConfigError {
    #[error("smtp host cannot be empty")]
    EmptyHost,
    #[error("smtp port must be non-zero")]
    InvalidPort,
    #[error("sender address cannot be empty")]
    EmptySender,
    #[error("sender address must contain '@'")]
    InvalidSender,
}

impl EmailConfig {
    pub fn validate(&self) -> Result<(), EmailConfigError> {
        if self.smtp_host.trim().is_empty() {
            return Err(EmailConfigError::EmptyHost);
        }
        if self.smtp_port == 0 {
            return Err(EmailConfigError::InvalidPort);
        }
        let sender = self.sender_address.trim();
        if sender.is_empty() {
            return Err(EmailConfigError::EmptySender);
        }
        if !sender.contains('@') {
            return Err(EmailConfigError::InvalidSender);
        }
        Ok(())
    }

    pub fn render_subject(&self, vars: &HashMap<&str, String>) -> String {
        render_placeholders(&self.subject_template, vars)
    }

    pub fn render_body(&self, vars: &HashMap<&str, String>) -> String {
        render_placeholders(&self.body_template, vars)
    }
}

/// Replace `{{key}}` occurrences with the corresponding value.
/// Unknown keys are left untouched so the user can see which ones didn't resolve.
pub fn render_placeholders(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end_rel) = find_close(&bytes[i + 2..]) {
                let key_start = i + 2;
                let key_end = i + 2 + end_rel;
                let key = template[key_start..key_end].trim();
                if let Some(value) = vars.get(key) {
                    out.push_str(value);
                    i = key_end + 2;
                    continue;
                }
            }
        }
        out.push(template[i..].chars().next().unwrap_or('\0'));
        i += template[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    out
}

fn find_close(bytes: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SellerProfile {
    pub name: String,
    pub title: Option<String>,
    pub registration_id: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub signature_image: Option<Vec<u8>>,
}

/// The app's display-currency setting. A thin newtype around
/// [`crate::domain::money::Currency`]; all formatting metadata (symbol,
/// fraction digits, unit names) is derived from the enum, not stored here.
///
/// The user picks one value from the closed set of [`Currency::all`]. There's
/// no free-form editing of symbol/position/etc. — the settings UI is a
/// dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyConfig {
    currency: crate::domain::money::Currency,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CurrencyConfigError {
    #[error("unsupported currency code: {0:?}")]
    UnsupportedCurrency(String),
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self {
            currency: crate::domain::money::Currency::Eur,
        }
    }
}

impl CurrencyConfig {
    pub const fn new(currency: crate::domain::money::Currency) -> Self {
        Self { currency }
    }

    pub fn from_code(code: &str) -> Result<Self, CurrencyConfigError> {
        crate::domain::money::Currency::parse(code)
            .map(Self::new)
            .ok_or_else(|| CurrencyConfigError::UnsupportedCurrency(code.to_string()))
    }

    pub const fn currency(&self) -> crate::domain::money::Currency {
        self.currency
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Light" => Some(Theme::Light),
            "Dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Fr,
    En,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Fr => "fr",
            Language::En => "en",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "fr" => Some(Language::Fr),
            "en" => Some(Language::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppPreferences {
    pub theme: Theme,
    pub language: Language,
    pub pdf_output_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_currency_config_is_euro() {
        let c = CurrencyConfig::default();
        assert_eq!(c.currency(), crate::domain::money::Currency::Eur);
    }

    #[test]
    fn from_code_accepts_supported_codes() {
        assert_eq!(
            CurrencyConfig::from_code("USD").unwrap().currency(),
            crate::domain::money::Currency::Usd,
        );
        assert_eq!(
            CurrencyConfig::from_code("JPY").unwrap().currency(),
            crate::domain::money::Currency::Jpy,
        );
    }

    #[test]
    fn from_code_rejects_unsupported() {
        let err = CurrencyConfig::from_code("XXX").unwrap_err();
        assert!(matches!(err, CurrencyConfigError::UnsupportedCurrency(_)));
    }

    #[test]
    fn theme_round_trips() {
        assert_eq!(Theme::parse(Theme::Light.as_str()), Some(Theme::Light));
        assert_eq!(Theme::parse(Theme::Dark.as_str()), Some(Theme::Dark));
        assert_eq!(Theme::parse("Neon"), None);
    }

    #[test]
    fn language_round_trips() {
        assert_eq!(Language::parse(Language::Fr.as_str()), Some(Language::Fr));
        assert_eq!(Language::parse(Language::En.as_str()), Some(Language::En));
        assert_eq!(Language::parse("de"), None);
    }

    #[test]
    fn seller_profile_default_is_empty() {
        let s = SellerProfile::default();
        assert_eq!(s.name, "");
        assert!(s.email.is_none());
    }

    fn valid_email_config() -> EmailConfig {
        EmailConfig {
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            sender_address: "me@example.com".into(),
            subject_template: "Invoice {{number}}".into(),
            body_template: "Hi {{client_name}}, total: {{total}}".into(),
        }
    }

    #[test]
    fn email_config_validate_accepts_valid() {
        assert!(valid_email_config().validate().is_ok());
    }

    #[test]
    fn email_config_validate_rejects_empty_host() {
        let c = EmailConfig {
            smtp_host: "  ".into(),
            ..valid_email_config()
        };
        assert_eq!(c.validate(), Err(EmailConfigError::EmptyHost));
    }

    #[test]
    fn email_config_validate_rejects_zero_port() {
        let c = EmailConfig {
            smtp_port: 0,
            ..valid_email_config()
        };
        assert_eq!(c.validate(), Err(EmailConfigError::InvalidPort));
    }

    #[test]
    fn email_config_validate_rejects_sender_without_at() {
        let c = EmailConfig {
            sender_address: "me".into(),
            ..valid_email_config()
        };
        assert_eq!(c.validate(), Err(EmailConfigError::InvalidSender));
    }

    #[test]
    fn render_placeholders_replaces_known_keys() {
        let mut vars = HashMap::new();
        vars.insert("number", "42".to_string());
        vars.insert("client_name", "Acme".to_string());
        let out = render_placeholders("Invoice {{number}} for {{client_name}}", &vars);
        assert_eq!(out, "Invoice 42 for Acme");
    }

    #[test]
    fn render_placeholders_leaves_unknown_keys_untouched() {
        let vars: HashMap<&str, String> = HashMap::new();
        let out = render_placeholders("Hello {{ghost}}", &vars);
        assert_eq!(out, "Hello {{ghost}}");
    }

    #[test]
    fn render_placeholders_allows_spaces_inside_braces() {
        let mut vars = HashMap::new();
        vars.insert("number", "7".to_string());
        let out = render_placeholders("N={{ number }}", &vars);
        assert_eq!(out, "N=7");
    }

    #[test]
    fn render_placeholders_handles_repeated_keys() {
        let mut vars = HashMap::new();
        vars.insert("x", "A".to_string());
        let out = render_placeholders("{{x}}-{{x}}-{{x}}", &vars);
        assert_eq!(out, "A-A-A");
    }

    #[test]
    fn render_placeholders_preserves_literal_braces_without_close() {
        let vars: HashMap<&str, String> = HashMap::new();
        let out = render_placeholders("{{ open with no end", &vars);
        assert_eq!(out, "{{ open with no end");
    }

    #[test]
    fn render_placeholders_handles_unicode_content() {
        let mut vars = HashMap::new();
        vars.insert("name", "Élise".to_string());
        let out = render_placeholders("Bonjour {{name}} — merci", &vars);
        assert_eq!(out, "Bonjour Élise — merci");
    }

    #[test]
    fn email_config_render_subject_and_body_use_same_engine() {
        let cfg = valid_email_config();
        let mut vars = HashMap::new();
        vars.insert("number", "1001".to_string());
        vars.insert("client_name", "Acme".to_string());
        vars.insert("total", "181.50 €".to_string());
        assert_eq!(cfg.render_subject(&vars), "Invoice 1001");
        assert_eq!(cfg.render_body(&vars), "Hi Acme, total: 181.50 €");
    }
}
