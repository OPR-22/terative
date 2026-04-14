use crate::application::RepoError;
use crate::domain::settings::{AppPreferences, CurrencyConfig, EmailConfig, SellerProfile};

pub trait SettingsRepository: Send + Sync {
    fn get_seller_profile(&self) -> Result<SellerProfile, RepoError>;
    fn set_seller_profile(&self, profile: &SellerProfile) -> Result<(), RepoError>;

    fn get_currency_config(&self) -> Result<CurrencyConfig, RepoError>;
    fn set_currency_config(&self, currency: &CurrencyConfig) -> Result<(), RepoError>;

    fn get_app_preferences(&self) -> Result<AppPreferences, RepoError>;
    fn set_app_preferences(&self, prefs: &AppPreferences) -> Result<(), RepoError>;

    fn get_email_config(&self) -> Result<EmailConfig, RepoError>;
    fn set_email_config(&self, config: &EmailConfig) -> Result<(), RepoError>;
}
