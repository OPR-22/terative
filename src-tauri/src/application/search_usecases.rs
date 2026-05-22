use std::sync::Arc;

use crate::application::ports::{SearchHit, SearchRepository};
use crate::application::AppError;

/// Hard cap on results returned by a single search. The palette UI shows a
/// flat, grouped list — more than this is noise, and the FTS5 `LIMIT` keeps
/// the query cheap.
pub const MAX_SEARCH_RESULTS: u32 = 30;

/// Cross-entity global search (T1.07). Turns the raw text a user types into
/// a safe FTS5 query and returns ranked hits across clients, invoices and
/// catalog items.
#[derive(Clone)]
pub struct GlobalSearch {
    repo: Arc<dyn SearchRepository>,
}

impl GlobalSearch {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, raw_query: &str) -> Result<Vec<SearchHit>, AppError> {
        match build_fts_query(raw_query) {
            // Nothing searchable in the input (blank, or only punctuation):
            // return an empty result without touching the database.
            None => Ok(Vec::new()),
            Some(fts_query) => Ok(self.repo.search(&fts_query, MAX_SEARCH_RESULTS)?),
        }
    }
}

/// Build a safe FTS5 `MATCH` expression from raw user input.
///
/// Each whitespace-separated word is reduced to its alphanumeric characters,
/// wrapped in double quotes and given a `*` prefix operator. Stripping
/// non-alphanumerics both removes FTS5 syntax characters (so the user can't
/// trigger a query-syntax error or inject operators) and makes quoting safe
/// — there is no `"` left to escape.
///
/// Words are joined with spaces, which is an implicit AND in FTS5: typing
/// "acme inv" requires a row to match a prefix of both. Returns `None` when
/// no usable term remains.
fn build_fts_query(raw: &str) -> Option<String> {
    let terms: Vec<String> = raw
        .split_whitespace()
        .filter_map(|word| {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if cleaned.is_empty() {
                None
            } else {
                Some(format!("\"{cleaned}\"*"))
            }
        })
        .collect();

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{SearchEntityKind, SearchRepository};
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use uuid::Uuid;

    /// Records the last query it was handed and replays a canned result set.
    #[derive(Default)]
    struct FakeRepo {
        last_call: Mutex<Option<(String, u32)>>,
        hits: Mutex<Vec<SearchHit>>,
    }

    impl FakeRepo {
        fn with_hits(hits: Vec<SearchHit>) -> Self {
            Self {
                last_call: Mutex::new(None),
                hits: Mutex::new(hits),
            }
        }
        fn last_query(&self) -> Option<String> {
            self.last_call.lock().clone().map(|(q, _)| q)
        }
        fn last_limit(&self) -> Option<u32> {
            self.last_call.lock().clone().map(|(_, l)| l)
        }
    }

    impl SearchRepository for FakeRepo {
        fn search(&self, fts_query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
            *self.last_call.lock() = Some((fts_query.to_string(), limit));
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
    fn single_word_becomes_a_quoted_prefix_term() {
        let repo = Arc::new(FakeRepo::default());
        GlobalSearch::new(repo.clone()).execute("acme").unwrap();
        assert_eq!(repo.last_query().as_deref(), Some("\"acme\"*"));
    }

    #[test]
    fn multiple_words_are_joined_as_prefix_terms() {
        let repo = Arc::new(FakeRepo::default());
        GlobalSearch::new(repo.clone()).execute("acme corp").unwrap();
        assert_eq!(repo.last_query().as_deref(), Some("\"acme\"* \"corp\"*"));
    }

    #[test]
    fn fts_operators_in_input_are_stripped() {
        let repo = Arc::new(FakeRepo::default());
        // `*`, `"`, `(`, `-`, `OR` punctuation must not reach FTS5 as syntax.
        GlobalSearch::new(repo.clone())
            .execute("acme* OR (\"corp\")")
            .unwrap();
        assert_eq!(
            repo.last_query().as_deref(),
            Some("\"acme\"* \"OR\"* \"corp\"*")
        );
    }

    #[test]
    fn numeric_query_is_preserved() {
        let repo = Arc::new(FakeRepo::default());
        GlobalSearch::new(repo.clone()).execute("42").unwrap();
        assert_eq!(repo.last_query().as_deref(), Some("\"42\"*"));
    }

    #[test]
    fn blank_query_returns_empty_without_hitting_the_repo() {
        let repo = Arc::new(FakeRepo::with_hits(vec![hit("Acme")]));
        let out = GlobalSearch::new(repo.clone()).execute("   ").unwrap();
        assert!(out.is_empty());
        assert!(repo.last_query().is_none(), "repo must not be queried");
    }

    #[test]
    fn punctuation_only_query_returns_empty_without_hitting_the_repo() {
        let repo = Arc::new(FakeRepo::with_hits(vec![hit("Acme")]));
        let out = GlobalSearch::new(repo.clone()).execute("!@#$ %^&*").unwrap();
        assert!(out.is_empty());
        assert!(repo.last_query().is_none());
    }

    #[test]
    fn passes_the_result_cap_as_the_limit() {
        let repo = Arc::new(FakeRepo::default());
        GlobalSearch::new(repo.clone()).execute("acme").unwrap();
        assert_eq!(repo.last_limit(), Some(MAX_SEARCH_RESULTS));
    }

    #[test]
    fn returns_the_repository_hits() {
        let repo = Arc::new(FakeRepo::with_hits(vec![hit("Acme"), hit("Globex")]));
        let out = GlobalSearch::new(repo).execute("co").unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].title, "Acme");
        assert_eq!(out[1].title, "Globex");
    }
}
