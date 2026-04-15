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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CurrencyConfigDto {
    pub code: String,
    pub symbol: String,
    pub symbol_before: bool,
    pub main_unit_name: String,
    pub sub_unit_name: String,
}

impl From<&CurrencyConfig> for CurrencyConfigDto {
    fn from(c: &CurrencyConfig) -> Self {
        Self {
            code: c.code.clone(),
            symbol: c.symbol.clone(),
            symbol_before: c.symbol_before,
            main_unit_name: c.main_unit_name.clone(),
            sub_unit_name: c.sub_unit_name.clone(),
        }
    }
}

impl From<CurrencyConfigDto> for CurrencyConfig {
    fn from(dto: CurrencyConfigDto) -> Self {
        CurrencyConfig {
            code: dto.code,
            symbol: dto.symbol,
            symbol_before: dto.symbol_before,
            main_unit_name: dto.main_unit_name,
            sub_unit_name: dto.sub_unit_name,
        }
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
    fn currency_config_round_trip() {
        let domain = CurrencyConfig::default();
        let dto: CurrencyConfigDto = (&domain).into();
        let back: CurrencyConfig = dto.into();
        assert_eq!(back, domain);
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
