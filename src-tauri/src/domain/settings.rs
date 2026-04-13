use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SellerProfile {
    pub name: String,
    pub title: Option<String>,
    pub registration_id: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyConfig {
    pub code: String,
    pub symbol: String,
    pub symbol_before: bool,
    pub main_unit_name: String,
    pub sub_unit_name: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CurrencyConfigError {
    #[error("currency code must not be empty")]
    EmptyCode,
    #[error("currency symbol must not be empty")]
    EmptySymbol,
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self {
            code: "EUR".into(),
            symbol: "€".into(),
            symbol_before: false,
            main_unit_name: "euros".into(),
            sub_unit_name: "centimes".into(),
        }
    }
}

impl CurrencyConfig {
    pub fn validate(&self) -> Result<(), CurrencyConfigError> {
        if self.code.trim().is_empty() {
            return Err(CurrencyConfigError::EmptyCode);
        }
        if self.symbol.trim().is_empty() {
            return Err(CurrencyConfigError::EmptySymbol);
        }
        Ok(())
    }

    pub fn format(&self, amount_cents: i64) -> String {
        let sign = if amount_cents < 0 { "-" } else { "" };
        let abs = amount_cents.unsigned_abs();
        let whole = abs / 100;
        let frac = abs % 100;
        let number = format!("{sign}{whole}.{frac:02}");
        if self.symbol_before {
            format!("{}{}", self.symbol, number)
        } else {
            format!("{} {}", number, self.symbol)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
        assert_eq!(c.code, "EUR");
        assert_eq!(c.symbol, "€");
        assert!(!c.symbol_before);
    }

    #[test]
    fn currency_config_validate_accepts_defaults() {
        assert!(CurrencyConfig::default().validate().is_ok());
    }

    #[test]
    fn currency_config_validate_rejects_empty_code() {
        let c = CurrencyConfig {
            code: "".into(),
            ..CurrencyConfig::default()
        };
        assert_eq!(c.validate(), Err(CurrencyConfigError::EmptyCode));
    }

    #[test]
    fn currency_config_validate_rejects_empty_symbol() {
        let c = CurrencyConfig {
            symbol: "".into(),
            ..CurrencyConfig::default()
        };
        assert_eq!(c.validate(), Err(CurrencyConfigError::EmptySymbol));
    }

    #[test]
    fn currency_format_symbol_after() {
        let c = CurrencyConfig::default();
        assert_eq!(c.format(12345), "123.45 €");
        assert_eq!(c.format(0), "0.00 €");
        assert_eq!(c.format(5), "0.05 €");
    }

    #[test]
    fn currency_format_symbol_before() {
        let c = CurrencyConfig {
            code: "USD".into(),
            symbol: "$".into(),
            symbol_before: true,
            main_unit_name: "dollars".into(),
            sub_unit_name: "cents".into(),
        };
        assert_eq!(c.format(12345), "$123.45");
    }

    #[test]
    fn currency_format_negative() {
        let c = CurrencyConfig::default();
        assert_eq!(c.format(-12345), "-123.45 €");
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
}
