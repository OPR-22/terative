use rusqlite::params;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::SettingsRepository;
use crate::application::RepoError;
use crate::domain::settings::{AppPreferences, CurrencyConfig, Language, SellerProfile, Theme};

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
        conn.query_row(
            "SELECT code, symbol, symbol_before, main_unit_name, sub_unit_name
             FROM currency_config WHERE id = 1",
            [],
            |row| {
                Ok(CurrencyConfig {
                    code: row.get(0)?,
                    symbol: row.get(1)?,
                    symbol_before: row.get::<_, i64>(2)? != 0,
                    main_unit_name: row.get(3)?,
                    sub_unit_name: row.get(4)?,
                })
            },
        )
        .map_err(map_err)
    }

    fn set_currency_config(&self, c: &CurrencyConfig) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE currency_config
             SET code = ?1, symbol = ?2, symbol_before = ?3, main_unit_name = ?4, sub_unit_name = ?5
             WHERE id = 1",
            params![
                c.code,
                c.symbol,
                c.symbol_before as i64,
                c.main_unit_name,
                c.sub_unit_name,
            ],
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
        let c = CurrencyConfig {
            code: "USD".into(),
            symbol: "$".into(),
            symbol_before: true,
            main_unit_name: "dollars".into(),
            sub_unit_name: "cents".into(),
        };
        repo.set_currency_config(&c).unwrap();
        let loaded = repo.get_currency_config().unwrap();
        assert_eq!(loaded, c);
    }

    #[test]
    fn currency_config_default_is_eur() {
        let db = open_memory();
        let repo = SqliteSettingsRepository::new(db);
        let loaded = repo.get_currency_config().unwrap();
        assert_eq!(loaded.code, "EUR");
        assert_eq!(loaded.symbol, "€");
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
}
