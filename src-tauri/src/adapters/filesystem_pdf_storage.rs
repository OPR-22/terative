use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::application::ports::{PdfStorage, SettingsRepository};
use crate::application::RepoError;

pub struct FilesystemPdfStorage {
    settings: Arc<dyn SettingsRepository>,
    default_dir: PathBuf,
}

impl FilesystemPdfStorage {
    pub fn new(settings: Arc<dyn SettingsRepository>, default_dir: PathBuf) -> Self {
        Self {
            settings,
            default_dir,
        }
    }

    fn resolve_dir(&self) -> Result<PathBuf, RepoError> {
        let prefs = self.settings.get_app_preferences()?;
        let configured = prefs.pdf_output_dir.trim();
        let dir = if configured.is_empty() {
            self.default_dir.clone()
        } else {
            PathBuf::from(configured)
        };
        Ok(dir)
    }
}

impl PdfStorage for FilesystemPdfStorage {
    fn store(&self, file_name: &str, bytes: &[u8]) -> Result<String, RepoError> {
        let dir = self.resolve_dir()?;
        fs::create_dir_all(&dir).map_err(|e| RepoError::Storage(e.to_string()))?;
        let path = dir.join(file_name);
        fs::write(&path, bytes).map_err(|e| RepoError::Storage(e.to_string()))?;
        Ok(path.to_string_lossy().to_string())
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, RepoError> {
        match fs::read(path) {
            Ok(bytes) => Ok(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(RepoError::NotFound)
            }
            Err(e) => Err(RepoError::Storage(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::{connection::open_memory, SqliteSettingsRepository};
    use crate::domain::settings::AppPreferences;

    fn settings_repo() -> Arc<SqliteSettingsRepository> {
        Arc::new(SqliteSettingsRepository::new(open_memory()))
    }

    #[test]
    fn store_uses_configured_output_dir_from_settings() {
        let tmp = tempfile::tempdir().unwrap();
        let configured = tmp.path().join("invoices");
        let settings = settings_repo();
        settings
            .set_app_preferences(&AppPreferences {
                pdf_output_dir: configured.to_string_lossy().to_string(),
                ..Default::default()
            })
            .unwrap();
        let fallback = tmp.path().join("fallback");
        let storage = FilesystemPdfStorage::new(settings, fallback.clone());

        let path = storage.store("invoice-1.pdf", b"hello").unwrap();

        assert!(path.starts_with(configured.to_string_lossy().as_ref()));
        assert!(!fallback.exists(), "fallback dir must not be touched");
        let written = fs::read(&path).unwrap();
        assert_eq!(written, b"hello");
    }

    #[test]
    fn store_falls_back_when_setting_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let fallback = tmp.path().join("fallback");
        let settings = settings_repo(); // default pdf_output_dir = ""
        let storage = FilesystemPdfStorage::new(settings, fallback.clone());

        let path = storage.store("invoice-2.pdf", b"bytes").unwrap();

        assert!(path.starts_with(fallback.to_string_lossy().as_ref()));
        assert!(fallback.exists());
    }

    #[test]
    fn store_creates_missing_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b").join("c");
        let settings = settings_repo();
        settings
            .set_app_preferences(&AppPreferences {
                pdf_output_dir: nested.to_string_lossy().to_string(),
                ..Default::default()
            })
            .unwrap();
        let storage = FilesystemPdfStorage::new(settings, tmp.path().to_path_buf());
        let path = storage.store("deep.pdf", b"x").unwrap();
        assert!(PathBuf::from(&path).exists());
    }

    #[test]
    fn store_rereads_setting_on_each_call() {
        let tmp = tempfile::tempdir().unwrap();
        let first = tmp.path().join("first");
        let second = tmp.path().join("second");
        let settings = settings_repo();
        settings
            .set_app_preferences(&AppPreferences {
                pdf_output_dir: first.to_string_lossy().to_string(),
                ..Default::default()
            })
            .unwrap();
        let storage = FilesystemPdfStorage::new(settings.clone(), tmp.path().to_path_buf());
        let p1 = storage.store("a.pdf", b"a").unwrap();
        settings
            .set_app_preferences(&AppPreferences {
                pdf_output_dir: second.to_string_lossy().to_string(),
                ..Default::default()
            })
            .unwrap();
        let p2 = storage.store("b.pdf", b"b").unwrap();
        assert!(p1.starts_with(first.to_string_lossy().as_ref()));
        assert!(p2.starts_with(second.to_string_lossy().as_ref()));
    }
}
