use std::sync::Arc;

use crate::application::ports::BookmarkRepository;
use crate::application::AppError;
use crate::domain::bookmark::{Bookmark, BookmarkId};

pub struct CreateBookmarkInput {
    pub label: String,
    pub url: String,
}

#[derive(Clone)]
pub struct CreateBookmark {
    repo: Arc<dyn BookmarkRepository>,
}

impl CreateBookmark {
    pub fn new(repo: Arc<dyn BookmarkRepository>) -> Self {
        Self { repo }
    }
    /// Append a new bookmark to the end of the list (sort_order = max + 1).
    pub fn execute(&self, input: CreateBookmarkInput) -> Result<Bookmark, AppError> {
        let next = self.repo.max_sort_order()? + 1;
        let bookmark = Bookmark::create(input.label, input.url, next)?;
        self.repo.insert(&bookmark)?;
        Ok(bookmark)
    }
}

pub struct UpdateBookmarkInput {
    pub id: BookmarkId,
    pub label: String,
    pub url: String,
}

pub struct UpdateBookmark {
    repo: Arc<dyn BookmarkRepository>,
}

impl UpdateBookmark {
    pub fn new(repo: Arc<dyn BookmarkRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: UpdateBookmarkInput) -> Result<Bookmark, AppError> {
        let mut bookmark = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        bookmark.rename(input.label)?;
        bookmark.relocate(input.url)?;
        self.repo.update(&bookmark)?;
        Ok(bookmark)
    }
}

pub struct DeleteBookmark {
    repo: Arc<dyn BookmarkRepository>,
}

impl DeleteBookmark {
    pub fn new(repo: Arc<dyn BookmarkRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: BookmarkId) -> Result<(), AppError> {
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct ListBookmarks {
    repo: Arc<dyn BookmarkRepository>,
}

impl ListBookmarks {
    pub fn new(repo: Arc<dyn BookmarkRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self) -> Result<Vec<Bookmark>, AppError> {
        Ok(self.repo.list()?)
    }
}

pub struct ReorderBookmarks {
    repo: Arc<dyn BookmarkRepository>,
}

impl ReorderBookmarks {
    pub fn new(repo: Arc<dyn BookmarkRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, ordered_ids: Vec<BookmarkId>) -> Result<(), AppError> {
        self.repo.reorder(&ordered_ids)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::bookmark::BookmarkError;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryBookmarkRepo {
        inner: Mutex<HashMap<BookmarkId, Bookmark>>,
    }

    impl BookmarkRepository for InMemoryBookmarkRepo {
        fn insert(&self, b: &Bookmark) -> Result<(), RepoError> {
            self.inner.lock().insert(b.id, b.clone());
            Ok(())
        }
        fn update(&self, b: &Bookmark) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&b.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(b.id, b.clone());
            Ok(())
        }
        fn get(&self, id: BookmarkId) -> Result<Option<Bookmark>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self) -> Result<Vec<Bookmark>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Bookmark> = g.values().cloned().collect();
            v.sort_by(|a, b| {
                a.sort_order
                    .cmp(&b.sort_order)
                    .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
            });
            Ok(v)
        }
        fn delete(&self, id: BookmarkId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
        fn reorder(&self, ordered_ids: &[BookmarkId]) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            for (idx, id) in ordered_ids.iter().enumerate() {
                if let Some(b) = g.get_mut(id) {
                    b.sort_order = idx as i32;
                }
            }
            Ok(())
        }
        fn max_sort_order(&self) -> Result<i32, RepoError> {
            let g = self.inner.lock();
            Ok(g.values().map(|b| b.sort_order).max().unwrap_or(-1))
        }
    }

    fn repo() -> Arc<InMemoryBookmarkRepo> {
        Arc::new(InMemoryBookmarkRepo::default())
    }

    #[test]
    fn create_assigns_increasing_sort_order() {
        let r = repo();
        let uc = CreateBookmark::new(r.clone());
        let a = uc
            .execute(CreateBookmarkInput {
                label: "A".into(),
                url: "https://a.com".into(),
            })
            .unwrap();
        let b = uc
            .execute(CreateBookmarkInput {
                label: "B".into(),
                url: "https://b.com".into(),
            })
            .unwrap();
        assert_eq!(a.sort_order, 0);
        assert_eq!(b.sort_order, 1);
    }

    #[test]
    fn create_rejects_invalid_url() {
        let r = repo();
        let err = CreateBookmark::new(r)
            .execute(CreateBookmarkInput {
                label: "X".into(),
                url: "ftp://oops".into(),
            })
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Bookmark(BookmarkError::UnsupportedScheme)
        ));
    }

    #[test]
    fn update_changes_label_and_url() {
        let r = repo();
        let b = CreateBookmark::new(r.clone())
            .execute(CreateBookmarkInput {
                label: "Old".into(),
                url: "https://old.com".into(),
            })
            .unwrap();
        let updated = UpdateBookmark::new(r)
            .execute(UpdateBookmarkInput {
                id: b.id,
                label: "New".into(),
                url: "https://new.com".into(),
            })
            .unwrap();
        assert_eq!(updated.label, "New");
        assert_eq!(updated.url, "https://new.com");
    }

    #[test]
    fn update_missing_returns_not_found() {
        let r = repo();
        let err = UpdateBookmark::new(r)
            .execute(UpdateBookmarkInput {
                id: BookmarkId::new(),
                label: "X".into(),
                url: "https://x.com".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn update_propagates_validation_errors() {
        let r = repo();
        let b = CreateBookmark::new(r.clone())
            .execute(CreateBookmarkInput {
                label: "OK".into(),
                url: "https://ok.com".into(),
            })
            .unwrap();
        let err = UpdateBookmark::new(r)
            .execute(UpdateBookmarkInput {
                id: b.id,
                label: "  ".into(),
                url: "https://ok.com".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Bookmark(BookmarkError::EmptyLabel)));
    }

    #[test]
    fn delete_removes_entity() {
        let r = repo();
        let b = CreateBookmark::new(r.clone())
            .execute(CreateBookmarkInput {
                label: "X".into(),
                url: "https://x.com".into(),
            })
            .unwrap();
        DeleteBookmark::new(r.clone()).execute(b.id).unwrap();
        assert!(r.inner.lock().is_empty());
    }

    #[test]
    fn delete_is_idempotent_on_missing() {
        let r = repo();
        // Deleting a never-existent id should not error (repo returns Ok).
        DeleteBookmark::new(r)
            .execute(BookmarkId::new())
            .unwrap();
    }

    #[test]
    fn list_returns_in_sort_order() {
        let r = repo();
        let uc = CreateBookmark::new(r.clone());
        let a = uc
            .execute(CreateBookmarkInput {
                label: "A".into(),
                url: "https://a.com".into(),
            })
            .unwrap();
        let b = uc
            .execute(CreateBookmarkInput {
                label: "B".into(),
                url: "https://b.com".into(),
            })
            .unwrap();
        let c = uc
            .execute(CreateBookmarkInput {
                label: "C".into(),
                url: "https://c.com".into(),
            })
            .unwrap();
        ReorderBookmarks::new(r.clone())
            .execute(vec![c.id, b.id, a.id])
            .unwrap();
        let list = ListBookmarks::new(r).execute().unwrap();
        assert_eq!(
            list.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(),
            vec!["C", "B", "A"]
        );
    }
}
