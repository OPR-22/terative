use rusqlite::params;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::SettingsRepository;
use crate::application::RepoError;
use crate::domain::settings::{
    AppPreferences, CurrencyConfig, EmailConfig, Language, SellerProfile, Theme,
};

pub struct SqliteSettingsRepository {
    db: Db,
}

impl SqliteSettingsRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn map_err(e: rusqlite::Error) -> RepoError {
    RepoError::Storage(e.to_string())
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get_seller_profile(&self) -> Result<SellerProfile, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT name, title, registration_id, address, phone, email, signature_image
             FROM seller_profile WHERE id = 1",
            [],
            |row| {
                Ok(SellerProfile {
                    name: row.get(0)?,
                    title: row.get(1)?,
                    registration_id: row.get(2)?,
                    address: row.get(3)?,
                    phone: row.get(4)?,
                    email: row.get(5)?,
                    signature_image: row.get(6)?,
                })
            },
        )
        .map_err(map_err)
    }

    fn set_seller_profile(&self, p: &SellerProfile) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE seller_profile
             SET name = ?1, title = ?2, registration_id = ?3, address = ?4,
                 phone = ?5, email = ?6, signature_image = ?7
             WHERE id = 1",
            params![
                p.name,
                p.title,
                p.registration_id,
                p.address,
                p.phone,
                p.email,
                p.signature_image,
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn get_currency_config(&self) -> Result<CurrencyConfig, RepoError> {
        let conn = self.db.lock();
        let code: String = conn
            .query_row(
                "SELECT code FROM currency_config WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .map_err(map_err)?;
        // If the stored code isn't in the supported set (e.g. migrating from
        // an older schema), fall back to the default.
        Ok(CurrencyConfig::from_code(&code).unwrap_or_default())
    }

    fn set_currency_config(&self, c: &CurrencyConfig) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE currency_config SET code = ?1 WHERE id = 1",
            params![c.currency().code()],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn get_app_preferences(&self) -> Result<AppPreferences, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT theme, language, pdf_output_dir FROM app_preferences WHERE id = 1",
            [],
            |row| {
                let theme_s: String = row.get(0)?;
                let lang_s: String = row.get(1)?;
                Ok(AppPreferences {
                    theme: Theme::parse(&theme_s).unwrap_or_default(),
                    language: Language::parse(&lang_s).unwrap_or_default(),
                    pdf_output_dir: row.get(2)?,
                })
            },
        )
        .map_err(map_err)
    }

    fn set_app_preferences(&self, p: &AppPreferences) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE app_preferences
             SET theme = ?1, language = ?2, pdf_output_dir = ?3
             WHERE id = 1",
            params![p.theme.as_str(), p.language.as_str(), p.pdf_output_dir],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn get_email_config(&self) -> Result<EmailConfig, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT smtp_host, smtp_port, sender_address
             FROM email_config WHERE id = 1",
            [],
            |row| {
                let port: i64 = row.get(1)?;
                Ok(EmailConfig {
                    smtp_host: row.get(0)?,
                    smtp_port: port as u16,
                    sender_address: row.get(2)?,
                })
            },
        )
        .map_err(map_err)
    }

    fn set_email_config(&self, c: &EmailConfig) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE email_config
             SET smtp_host = ?1, smtp_port = ?2, sender_address = ?3
             WHERE id = 1",
            params![c.smtp_host, c.smtp_port as i64, c.sender_address],
        )
        .map_err(map_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    #[test]
    fn seller_profile_round_trip() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let p = SellerProfile {
            name: "Alice".into(),
            title: Some("Consultant".into()),
            registration_id: Some("REG-123".into()),
            address: Some("1 Main St".into()),
            phone: Some("555-0100".into()),
            email: Some("alice@example.com".into()),
            signature_image: None,
        };
        repo.set_seller_profile(&p).unwrap();
        let loaded = repo.get_seller_profile().unwrap();
        assert_eq!(loaded, p);
    }

    #[test]
    fn seller_profile_default_is_empty_string_name() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let loaded = repo.get_seller_profile().unwrap();
        assert_eq!(loaded.name, "");
    }

    #[test]
    fn currency_config_round_trip() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let c = CurrencyConfig::new(crate::domain::money::Currency::Usd);
        repo.set_currency_config(&c).unwrap();
        let loaded = repo.get_currency_config().unwrap();
        assert_eq!(loaded, c);
    }

    #[test]
    fn currency_config_default_is_eur() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let loaded = repo.get_currency_config().unwrap();
        assert_eq!(loaded.currency(), crate::domain::money::Currency::Eur);
    }

    #[test]
    fn app_preferences_round_trip() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let p = AppPreferences {
            theme: Theme::Dark,
            language: Language::En,
            pdf_output_dir: "/tmp/pdfs".into(),
        };
        repo.set_app_preferences(&p).unwrap();
        let loaded = repo.get_app_preferences().unwrap();
        assert_eq!(loaded, p);
    }

    #[test]
    fn app_preferences_default_is_light_fr() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let loaded = repo.get_app_preferences().unwrap();
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.language, Language::Fr);
    }

    #[test]
    fn email_config_default_has_port_587() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let loaded = repo.get_email_config().unwrap();
        assert_eq!(loaded.smtp_port, 587);
        assert_eq!(loaded.smtp_host, "");
    }

    #[test]
    fn email_config_round_trip() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let c = EmailConfig {
            smtp_host: "smtp.example.com".into(),
            smtp_port: 465,
            sender_address: "me@example.com".into(),
        };
        repo.set_email_config(&c).unwrap();
        let loaded = repo.get_email_config().unwrap();
        assert_eq!(loaded, c);
    }
}
