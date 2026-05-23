use crate::application::RepoError;
use crate::domain::notebook::{NotebookSection, NotebookSectionId};

pub trait NotebookSectionRepository: Send + Sync {
    fn insert(&self, section: &NotebookSection) -> Result<(), RepoError>;
    fn update(&self, section: &NotebookSection) -> Result<(), RepoError>;
    fn get(&self, id: NotebookSectionId) -> Result<Option<NotebookSection>, RepoError>;
    /// Returns all sections ordered by `sort_order` ASC, ties broken by name.
    fn list(&self) -> Result<Vec<NotebookSection>, RepoError>;
    fn delete(&self, id: NotebookSectionId) -> Result<(), RepoError>;
    /// Number of anamnesis entries that reference this section (for delete warnings).
    fn count_entries(&self, id: NotebookSectionId) -> Result<u64, RepoError>;
    /// Reassign `sort_order` from the provided ordering, in a single transaction.
    /// Any id not listed keeps its old sort_order (no-op).
    fn reorder(&self, ordered_ids: &[NotebookSectionId]) -> Result<(), RepoError>;
    /// Max existing sort_order, used to append new sections at the end.
    fn max_sort_order(&self) -> Result<i32, RepoError>;
}
