use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::notebook::ClientNotebook;

pub trait ClientNotebookRepository: Send + Sync {
    /// Upsert every entry inside a single transaction. Entries absent from
    /// the aggregate are left untouched in the database (clearing a section
    /// means passing an entry with empty content, not omitting it).
    fn save(&self, notebook: &ClientNotebook) -> Result<(), RepoError>;

    /// Load the aggregate for a client. Returns a notebook with only the
    /// stored rows; the use case layer merges this with the global section
    /// list to produce a sparse view.
    fn load(&self, client_id: ClientId) -> Result<ClientNotebook, RepoError>;
}
