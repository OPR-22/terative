use crate::application::RepoError;

pub trait PdfStorage: Send + Sync {
    fn store(&self, file_name: &str, bytes: &[u8]) -> Result<String, RepoError>;
    /// Reads a PDF previously written by `store`. The path comes from
    /// `Invoice::pdf_path` so callers don't need to know the storage layout.
    /// Returns `RepoError::NotFound` if the file is missing (e.g. user moved
    /// the output directory after finalize) so the UI can render an empty
    /// state instead of crashing.
    fn read(&self, path: &str) -> Result<Vec<u8>, RepoError>;
}
