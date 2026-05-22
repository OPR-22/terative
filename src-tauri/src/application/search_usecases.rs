use std::sync::Arc;

use crate::application::ports::{SearchHit, SearchRepository};
use crate::application::AppError;

/// Hard cap on results returned by a single search. The palette UI shows a
/// flat, grouped list — more than this is noise.
pub const MAX_SEARCH_RESULTS: u32 = 30;

/// Cross-entity global search (T1.07).
///
/// The application-layer entry point the `global_search` command calls. It
/// is deliberately thin: deciding *how* to search — full-text for names,
/// references and invoice numbers, substring for client emails and phone
/// numbers — belongs to the `SearchRepository` adapter, which owns that
/// SQLite-specific knowledge. This use case just applies the result cap.
#[derive(Clone)]
pub struct GlobalSearch {
    repo: Arc<dyn SearchRepository>,
}

impl GlobalSearch {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, query: &str) -> Result<Vec<SearchHit>, AppError> {
        Ok(self.repo.search(query, MAX_SEARCH_RESULTS)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::SearchEntityKind;
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use uuid::Uuid;

    /// Records the query it was handed and replays a canned result set.
    #[derive(Default)]
    struct FakeRepo {
        last_call: Mutex<Option<(String, u32)>>,
        hits: Mutex<Vec<SearchHit>>,
    }

    impl SearchRepository for FakeRepo {
        fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
            *self.last_call.lock() = Some((query.to_string(), limit));
            Ok(self.hits.lock().clone())
        }
    }

    fn hit(title: &str) -> SearchHit {
        SearchHit {
            kind: SearchEntityKind::Client,
            entity_id: Uuid::new_v4(),
            title: title.into(),
            snippet: String::new(),
        }
    }

    #[test]
    fn execute_passes_the_raw_query_and_result_cap_to_the_repo() {
        let repo = Arc::new(FakeRepo::default());
        GlobalSearch::new(repo.clone())
            .execute("john@acme.com")
            .unwrap();
        assert_eq!(
            *repo.last_call.lock(),
            Some(("john@acme.com".to_string(), MAX_SEARCH_RESULTS))
        );
    }

    #[test]
    fn execute_returns_the_repository_hits() {
        let repo = Arc::new(FakeRepo::default());
        *repo.hits.lock() = vec![hit("Acme"), hit("Globex")];
        let out = GlobalSearch::new(repo).execute("co").unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].title, "Acme");
        assert_eq!(out[1].title, "Globex");
    }
}
