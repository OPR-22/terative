use crate::application::RepoError;
use crate::domain::bookmark::{Bookmark, BookmarkId};

pub trait BookmarkRepository: Send + Sync {
    fn insert(&self, bookmark: &Bookmark) -> Result<(), RepoError>;
    fn update(&self, bookmark: &Bookmark) -> Result<(), RepoError>;
    fn get(&self, id: BookmarkId) -> Result<Option<Bookmark>, RepoError>;
    /// Returns all bookmarks ordered by `sort_order` ASC, ties broken by label.
    fn list(&self) -> Result<Vec<Bookmark>, RepoError>;
    fn delete(&self, id: BookmarkId) -> Result<(), RepoError>;
    /// Reassign `sort_order` from the provided ordering, in a single transaction.
    /// Any id not listed keeps its old sort_order (no-op).
    fn reorder(&self, ordered_ids: &[BookmarkId]) -> Result<(), RepoError>;
    /// Max existing sort_order, used to append new bookmarks at the end.
    /// Returns -1 when no bookmarks exist.
    fn max_sort_order(&self) -> Result<i32, RepoError>;
}
