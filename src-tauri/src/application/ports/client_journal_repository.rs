use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::notebook::{ClientJournalEntry, JournalEntryId};

pub trait ClientJournalRepository: Send + Sync {
    fn insert(&self, entry: &ClientJournalEntry) -> Result<(), RepoError>;
    fn update(&self, entry: &ClientJournalEntry) -> Result<(), RepoError>;
    fn get(&self, id: JournalEntryId) -> Result<Option<ClientJournalEntry>, RepoError>;
    /// Entries for a client ordered by `entry_date` DESC, ties by `created_at` DESC.
    fn list_for_client(&self, id: ClientId) -> Result<Vec<ClientJournalEntry>, RepoError>;
    fn delete(&self, id: JournalEntryId) -> Result<(), RepoError>;
}
