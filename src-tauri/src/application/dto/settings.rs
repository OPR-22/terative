use serde::{Deserialize, Serialize};

use crate::application::settings_usecases::SettingsSnapshot;
use crate::domain::settings::{
    AppPreferences, CurrencyConfig, EmailConfig, Language, SellerProfile, Theme,
};

// ---- SellerProfileDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, Default)]
pub struct SellerProfileDto {
    pub name: String,
    pub title: Option<String>,
    pub registration_id: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub signature_image: Option<Vec<u8>>,
}

impl From<&SellerProfile> for SellerProfileDto {
    fn from(s: &SellerProfile) -> Self {
        Self {
            name: s.name.clone(),
            title: s.title.clone(),
            registration_id: s.registration_id.clone(),
            address: s.address.clone(),
            phone: s.phone.clone(),
            email: s.email.clone(),
            signature_image: s.signature_image.clone(),
        }
    }
}

impl From<SellerProfileDto> for SellerProfile {
    fn from(dto: SellerProfileDto) -> Self {
        SellerProfile {
            name: dto.name,
            title: dto.title,
            registration_id: dto.registration_id,
            address: dto.address,
            phone: dto.phone,
            email: dto.email,
            signature_image: dto.signature_image,
        }
    }
}

// ---- CurrencyConfigDto ----

/// Full metadata for a currency as seen by the frontend. The write-side
/// command only sends `code`; everything else is derived server-side from the
/// [`Currency`] enum and included on the read path so the UI can render
/// amounts without a second catalog lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CurrencyConfigDto {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub symbol_before: bool,
    pub fraction_digits: u8,
    pub main_unit_name: String,
    pub sub_unit_name: Option<String>,
}

impl From<crate::domain::money::Currency> for CurrencyConfigDto {
    fn from(currency: crate::domain::money::Currency) -> Self {
        let meta = currency.meta();
        Self {
            code: meta.code.to_string(),
            name: meta.name.to_string(),
            symbol: meta.symbol.to_string(),
            symbol_before: matches!(
                meta.symbol_position,
                crate::domain::money::SymbolPosition::Before
            ),
            fraction_digits: meta.fraction_digits,
            main_unit_name: meta.main_unit_name.to_string(),
            sub_unit_name: meta.sub_unit_name.map(|s| s.to_string()),
        }
    }
}

impl From<&CurrencyConfig> for CurrencyConfigDto {
    fn from(c: &CurrencyConfig) -> Self {
        c.currency().into()
    }
}

impl From<CurrencyConfig> for CurrencyConfigDto {
    fn from(c: CurrencyConfig) -> Self {
        (&c).into()
    }
}

// ---- ThemeDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
pub enum ThemeDto {
    #[default]
    Light,
    Dark,
}

impl From<Theme> for ThemeDto {
    fn from(t: Theme) -> Self {
        match t {
            Theme::Light => Self::Light,
            Theme::Dark => Self::Dark,
        }
    }
}

impl From<ThemeDto> for Theme {
    fn from(dto: ThemeDto) -> Self {
        match dto {
            ThemeDto::Light => Self::Light,
            ThemeDto::Dark => Self::Dark,
        }
    }
}

// ---- LanguageDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
pub enum LanguageDto {
    #[default]
    Fr,
    En,
}

impl From<Language> for LanguageDto {
    fn from(l: Language) -> Self {
        match l {
            Language::Fr => Self::Fr,
            Language::En => Self::En,
        }
    }
}

impl From<LanguageDto> for Language {
    fn from(dto: LanguageDto) -> Self {
        match dto {
            LanguageDto::Fr => Self::Fr,
            LanguageDto::En => Self::En,
        }
    }
}

// ---- AppPreferencesDto ----

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
pub struct AppPreferencesDto {
    pub theme: ThemeDto,
    pub language: LanguageDto,
    pub pdf_output_dir: String,
}

impl From<&AppPreferences> for AppPreferencesDto {
    fn from(p: &AppPreferences) -> Self {
        Self {
            theme: p.theme.into(),
            language: p.language.into(),
            pdf_output_dir: p.pdf_output_dir.clone(),
        }
    }
}

impl From<AppPreferencesDto> for AppPreferences {
    fn from(dto: AppPreferencesDto) -> Self {
        AppPreferences {
            theme: dto.theme.into(),
            language: dto.language.into(),
            pdf_output_dir: dto.pdf_output_dir,
        }
    }
}

// ---- EmailConfigDto ----

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
pub struct EmailConfigDto {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub sender_address: String,
    pub subject_template: String,
    pub body_template: String,
}

impl From<&EmailConfig> for EmailConfigDto {
    fn from(c: &EmailConfig) -> Self {
        Self {
            smtp_host: c.smtp_host.clone(),
            smtp_port: c.smtp_port,
            sender_address: c.sender_address.clone(),
            subject_template: c.subject_template.clone(),
            body_template: c.body_template.clone(),
        }
    }
}

impl From<EmailConfigDto> for EmailConfig {
    fn from(dto: EmailConfigDto) -> Self {
        EmailConfig {
            smtp_host: dto.smtp_host,
            smtp_port: dto.smtp_port,
            sender_address: dto.sender_address,
            subject_template: dto.subject_template,
            body_template: dto.body_template,
        }
    }
}

// ---- SettingsSnapshotDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SettingsSnapshotDto {
    pub seller: SellerProfileDto,
    pub currency: CurrencyConfigDto,
    pub preferences: AppPreferencesDto,
    pub email: EmailConfigDto,
    pub has_email_password: bool,
}

impl From<&SettingsSnapshot> for SettingsSnapshotDto {
    fn from(s: &SettingsSnapshot) -> Self {
        Self {
            seller: (&s.seller).into(),
            currency: (&s.currency).into(),
            preferences: (&s.preferences).into(),
            email: (&s.email).into(),
            has_email_password: s.has_email_password,
        }
    }
}

impl From<SettingsSnapshot> for SettingsSnapshotDto {
    fn from(s: SettingsSnapshot) -> Self {
        (&s).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_round_trips() {
        for theme in [Theme::Light, Theme::Dark] {
            let dto: ThemeDto = theme.into();
            let back: Theme = dto.into();
            assert_eq!(back, theme);
        }
    }

    #[test]
    fn language_round_trips() {
        for lang in [Language::Fr, Language::En] {
            let dto: LanguageDto = lang.into();
            let back: Language = dto.into();
            assert_eq!(back, lang);
        }
    }

    #[test]
    fn seller_profile_round_trip_preserves_all_fields() {
        let domain = SellerProfile {
            name: "Me".into(),
            title: Some("Consultant".into()),
            registration_id: Some("REG".into()),
            address: Some("Addr".into()),
            phone: Some("555".into()),
            email: Some("e@e".into()),
            signature_image: Some(vec![1, 2, 3]),
        };
        let dto: SellerProfileDto = (&domain).into();
        let back: SellerProfile = dto.into();
        assert_eq!(back, domain);
    }

    #[test]
    fn currency_config_dto_carries_metadata_for_read_side() {
        let dto: CurrencyConfigDto = (&CurrencyConfig::default()).into();
        assert_eq!(dto.code, "EUR");
        assert_eq!(dto.symbol, "€");
        assert!(!dto.symbol_before);
        assert_eq!(dto.fraction_digits, 2);
        assert_eq!(dto.main_unit_name, "euros");
    }

    #[test]
    fn currency_config_dto_for_jpy_has_no_sub_unit() {
        let dto: CurrencyConfigDto =
            CurrencyConfig::new(crate::domain::money::Currency::Jpy)
                .into();
        assert_eq!(dto.code, "JPY");
        assert_eq!(dto.fraction_digits, 0);
        assert_eq!(dto.sub_unit_name, None);
    }

    #[test]
    fn settings_snapshot_to_dto() {
        let snap = SettingsSnapshot {
            seller: SellerProfile::default(),
            currency: CurrencyConfig::default(),
            preferences: AppPreferences::default(),
            email: EmailConfig::default(),
            has_email_password: true,
        };
        let dto: SettingsSnapshotDto = (&snap).into();
        assert!(dto.has_email_password);
        assert_eq!(dto.currency.code, "EUR");
    }
}
