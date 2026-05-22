use uuid::Uuid;

use crate::application::RepoError;

/// The kind of entity a search hit refers to. Determines which route the
/// frontend navigates to when a hit is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchEntityKind {
    Client,
    Invoice,
    CatalogItem,
}

impl SearchEntityKind {
    /// The `entity_type` discriminator stored in the `search_index` table.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Invoice => "invoice",
            Self::CatalogItem => "catalog_item",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "client" => Some(Self::Client),
            "invoice" => Some(Self::Invoice),
            "catalog_item" => Some(Self::CatalogItem),
            _ => None,
        }
    }
}

/// A single full-text search result.
///
/// `title` is the primary display string (client name, catalog item name,
/// invoice number). `snippet` is a short excerpt of the matched secondary
/// text (contact name, reference…) — empty when the entity has no secondary
/// text to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub kind: SearchEntityKind,
    pub entity_id: Uuid,
    pub title: String,
    pub snippet: String,
}

/// Read-only port for the cross-entity full-text search backed by the
/// `search_index` FTS5 table (migration 002).
pub trait SearchRepository: Send + Sync {
    /// Runs an FTS5 `MATCH` query, best matches first.
    ///
    /// `fts_query` must already be valid FTS5 query syntax — the
    /// `GlobalSearch` use case is responsible for turning raw user input
    /// into a safe query string. `limit` caps the number of rows returned.
    fn search(&self, fts_query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError>;
}
