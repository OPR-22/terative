use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::application::ports::SettingsRepository;
use crate::application::AppError;
use crate::domain::settings::{AppPreferences, CurrencyConfig, SellerProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSnapshot {
    pub seller: SellerProfile,
    pub currency: CurrencyConfig,
    pub preferences: AppPreferences,
}

pub struct GetSettings {
    repo: Arc<dyn SettingsRepository>,
}

impl GetSettings {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self) -> Result<SettingsSnapshot, AppError> {
        Ok(SettingsSnapshot {
            seller: self.repo.get_seller_profile()?,
            currency: self.repo.get_currency_config()?,
            preferences: self.repo.get_app_preferences()?,
        })
    }
}

pub struct UpdateSellerProfile {
    repo: Arc<dyn SettingsRepository>,
}

impl UpdateSellerProfile {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, profile: SellerProfile) -> Result<SellerProfile, AppError> {
        self.repo.set_seller_profile(&profile)?;
        Ok(profile)
    }
}

pub struct UpdateCurrency {
    repo: Arc<dyn SettingsRepository>,
}

impl UpdateCurrency {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, currency: CurrencyConfig) -> Result<CurrencyConfig, AppError> {
        currency.validate()?;
        self.repo.set_currency_config(&currency)?;
        Ok(currency)
    }
}

pub struct UpdateAppPreferences {
    repo: Arc<dyn SettingsRepository>,
}

impl UpdateAppPreferences {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, prefs: AppPreferences) -> Result<AppPreferences, AppError> {
        self.repo.set_app_preferences(&prefs)?;
        Ok(prefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::settings::{Language, Theme};
    use parking_lot::Mutex;

    #[derive(Default)]
    struct InMemorySettingsRepo {
        seller: Mutex<SellerProfile>,
        currency: Mutex<CurrencyConfig>,
        prefs: Mutex<AppPreferences>,
    }

    impl SettingsRepository for InMemorySettingsRepo {
        fn get_seller_profile(&self) -> Result<SellerProfile, RepoError> {
            Ok(self.seller.lock().clone())
        }
        fn set_seller_profile(&self, profile: &SellerProfile) -> Result<(), RepoError> {
            *self.seller.lock() = profile.clone();
            Ok(())
        }
        fn get_currency_config(&self) -> Result<CurrencyConfig, RepoError> {
            Ok(self.currency.lock().clone())
        }
        fn set_currency_config(&self, currency: &CurrencyConfig) -> Result<(), RepoError> {
            *self.currency.lock() = currency.clone();
            Ok(())
        }
        fn get_app_preferences(&self) -> Result<AppPreferences, RepoError> {
            Ok(self.prefs.lock().clone())
        }
        fn set_app_preferences(&self, prefs: &AppPreferences) -> Result<(), RepoError> {
            *self.prefs.lock() = prefs.clone();
            Ok(())
        }
    }

    fn repo() -> Arc<InMemorySettingsRepo> {
        Arc::new(InMemorySettingsRepo::default())
    }

    #[test]
    fn get_settings_returns_defaults_from_repo() {
        let r = repo();
        let s = GetSettings::new(r).execute().unwrap();
        assert_eq!(s.currency.code, "EUR");
        assert_eq!(s.preferences.theme, Theme::Light);
        assert_eq!(s.preferences.language, Language::Fr);
    }

    #[test]
    fn update_seller_profile_persists() {
        let r = repo();
        let profile = SellerProfile {
            name: "Alice".into(),
            email: Some("alice@example.com".into()),
            ..Default::default()
        };
        UpdateSellerProfile::new(r.clone())
            .execute(profile.clone())
            .unwrap();
        let loaded = r.get_seller_profile().unwrap();
        assert_eq!(loaded.name, "Alice");
        assert_eq!(loaded.email.as_deref(), Some("alice@example.com"));
    }

    #[test]
    fn update_currency_rejects_empty_code() {
        let r = repo();
        let c = CurrencyConfig {
            code: "".into(),
            ..CurrencyConfig::default()
        };
        let err = UpdateCurrency::new(r).execute(c).unwrap_err();
        assert!(matches!(err, AppError::Currency(_)));
    }

    #[test]
    fn update_currency_persists_valid() {
        let r = repo();
        let c = CurrencyConfig {
            code: "USD".into(),
            symbol: "$".into(),
            symbol_before: true,
            main_unit_name: "dollars".into(),
            sub_unit_name: "cents".into(),
        };
        UpdateCurrency::new(r.clone())
            .execute(c.clone())
            .unwrap();
        assert_eq!(r.get_currency_config().unwrap().code, "USD");
    }

    #[test]
    fn update_app_preferences_persists() {
        let r = repo();
        let prefs = AppPreferences {
            theme: Theme::Dark,
            language: Language::En,
            pdf_output_dir: "/tmp/pdfs".into(),
        };
        UpdateAppPreferences::new(r.clone())
            .execute(prefs.clone())
            .unwrap();
        let loaded = r.get_app_preferences().unwrap();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.language, Language::En);
        assert_eq!(loaded.pdf_output_dir, "/tmp/pdfs");
    }
}
