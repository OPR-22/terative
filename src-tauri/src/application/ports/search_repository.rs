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

/// A single search result.
///
/// `title` is the primary display string (client name, catalog item name,
/// invoice number). `snippet` is a short excerpt of the matched secondary
/// text (contact name, reference…) or, for an email/phone match, the
/// matching value — empty when there is nothing secondary to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub kind: SearchEntityKind,
    pub entity_id: Uuid,
    pub title: String,
    pub snippet: String,
}

/// Read-only port for global search across clients, invoices and catalog
/// items.
pub trait SearchRepository: Send + Sync {
    /// Searches everything for `query` — raw text as the user typed it —
    /// and returns the best matches, at most `limit`.
    ///
    /// *How* it searches is the implementation's business: the SQLite
    /// adapter runs a full-text match for names, references and invoice
    /// numbers, plus substring matches for client emails and phone
    /// numbers, then merges them into one de-duplicated list.
    fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError>;
}
