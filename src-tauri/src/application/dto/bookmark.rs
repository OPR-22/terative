use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::bookmark_usecases::{CreateBookmarkInput, UpdateBookmarkInput};
use crate::domain::bookmark::{Bookmark, BookmarkId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct BookmarkDto {
    pub id: Uuid,
    pub label: String,
    pub url: String,
    pub sort_order: i32,
}

impl From<&Bookmark> for BookmarkDto {
    fn from(b: &Bookmark) -> Self {
        Self {
            id: b.id.0,
            label: b.label.clone(),
            url: b.url.clone(),
            sort_order: b.sort_order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewBookmarkDto {
    pub label: String,
    pub url: String,
}

impl From<NewBookmarkDto> for CreateBookmarkInput {
    fn from(dto: NewBookmarkDto) -> Self {
        Self {
            label: dto.label,
            url: dto.url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateBookmarkDto {
    pub id: Uuid,
    pub label: String,
    pub url: String,
}

impl From<UpdateBookmarkDto> for UpdateBookmarkInput {
    fn from(dto: UpdateBookmarkDto) -> Self {
        Self {
            id: BookmarkId(dto.id),
            label: dto.label,
            url: dto.url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_to_dto_round_trip() {
        let domain = Bookmark {
            id: BookmarkId::new(),
            label: "Google".into(),
            url: "https://google.com".into(),
            sort_order: 3,
        };
        let dto: BookmarkDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.label, "Google");
        assert_eq!(dto.url, "https://google.com");
        assert_eq!(dto.sort_order, 3);
    }

    #[test]
    fn new_bookmark_dto_maps_to_input() {
        let dto = NewBookmarkDto {
            label: "X".into(),
            url: "https://x.com".into(),
        };
        let input: CreateBookmarkInput = dto.into();
        assert_eq!(input.label, "X");
        assert_eq!(input.url, "https://x.com");
    }

    #[test]
    fn update_bookmark_dto_preserves_id() {
        let id = Uuid::new_v4();
        let dto = UpdateBookmarkDto {
            id,
            label: "L".into(),
            url: "https://l.com".into(),
        };
        let input: UpdateBookmarkInput = dto.into();
        assert_eq!(input.id.0, id);
    }
}
