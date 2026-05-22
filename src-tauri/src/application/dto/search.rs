use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::ports::{SearchEntityKind, SearchHit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum SearchEntityKindDto {
    Client,
    Invoice,
    CatalogItem,
}

impl From<SearchEntityKind> for SearchEntityKindDto {
    fn from(k: SearchEntityKind) -> Self {
        match k {
            SearchEntityKind::Client => Self::Client,
            SearchEntityKind::Invoice => Self::Invoice,
            SearchEntityKind::CatalogItem => Self::CatalogItem,
        }
    }
}

/// One global-search result on the wire. `kind` tells the frontend which
/// route to navigate to; `entity_id` is the row to open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct SearchHitDto {
    pub kind: SearchEntityKindDto,
    pub entity_id: Uuid,
    pub title: String,
    pub snippet: String,
}

impl From<&SearchHit> for SearchHitDto {
    fn from(h: &SearchHit) -> Self {
        Self {
            kind: h.kind.into(),
            entity_id: h.entity_id,
            title: h.title.clone(),
            snippet: h.snippet.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_hit_maps_to_dto() {
        let id = Uuid::new_v4();
        let hit = SearchHit {
            kind: SearchEntityKind::Invoice,
            entity_id: id,
            title: "42".into(),
            snippet: "annual retainer".into(),
        };
        let dto: SearchHitDto = (&hit).into();
        assert_eq!(dto.entity_id, id);
        assert_eq!(dto.title, "42");
        assert_eq!(dto.snippet, "annual retainer");
        assert!(matches!(dto.kind, SearchEntityKindDto::Invoice));
    }

    #[test]
    fn entity_kind_maps_each_variant() {
        assert!(matches!(
            SearchEntityKindDto::from(SearchEntityKind::Client),
            SearchEntityKindDto::Client
        ));
        assert!(matches!(
            SearchEntityKindDto::from(SearchEntityKind::Invoice),
            SearchEntityKindDto::Invoice
        ));
        assert!(matches!(
            SearchEntityKindDto::from(SearchEntityKind::CatalogItem),
            SearchEntityKindDto::CatalogItem
        ));
    }
}
