use crate::application::RepoError;

pub trait PdfStorage: Send + Sync {
    fn store(&self, file_name: &str, bytes: &[u8]) -> Result<String, RepoError>;
}
