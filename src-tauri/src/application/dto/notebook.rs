use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::notebook_usecases::{
    ClientNotebookSection, ClientNotebookView, RenameNotebookSectionInput,
    SaveClientNotebookInput, UpdateJournalEntryInput,
};
use crate::domain::client::ClientId;
use crate::domain::notebook::{
    ClientJournalEntry, JournalEntryId, NewJournalEntry, NotebookEntry, NotebookSection,
    NotebookSectionId,
};

// ---- NotebookSectionDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct NotebookSectionDto {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i32,
}

impl From<&NotebookSection> for NotebookSectionDto {
    fn from(s: &NotebookSection) -> Self {
        Self {
            id: s.id.0,
            name: s.name.clone(),
            sort_order: s.sort_order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RenameNotebookSectionDto {
    pub id: Uuid,
    pub name: String,
}

impl From<RenameNotebookSectionDto> for RenameNotebookSectionInput {
    fn from(dto: RenameNotebookSectionDto) -> Self {
        RenameNotebookSectionInput {
            id: NotebookSectionId(dto.id),
            name: dto.name,
        }
    }
}

// ---- NotebookEntryDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct NotebookEntryDto {
    pub section_id: Uuid,
    pub content: String,
}

impl From<&NotebookEntry> for NotebookEntryDto {
    fn from(e: &NotebookEntry) -> Self {
        Self {
            section_id: e.section_id.0,
            content: e.content.clone(),
        }
    }
}

impl From<NotebookEntryDto> for NotebookEntry {
    fn from(dto: NotebookEntryDto) -> Self {
        NotebookEntry {
            section_id: NotebookSectionId(dto.section_id),
            content: dto.content,
        }
    }
}

// ---- ClientNotebookViewDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ClientNotebookSectionDto {
    pub section: NotebookSectionDto,
    pub content: String,
}

impl From<&ClientNotebookSection> for ClientNotebookSectionDto {
    fn from(s: &ClientNotebookSection) -> Self {
        Self {
            section: (&s.section).into(),
            content: s.content.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ClientNotebookViewDto {
    pub client_id: Uuid,
    pub sections: Vec<ClientNotebookSectionDto>,
}

impl From<&ClientNotebookView> for ClientNotebookViewDto {
    fn from(v: &ClientNotebookView) -> Self {
        Self {
            client_id: v.client_id.0,
            sections: v.sections.iter().map(Into::into).collect(),
        }
    }
}

impl From<ClientNotebookView> for ClientNotebookViewDto {
    fn from(v: ClientNotebookView) -> Self {
        (&v).into()
    }
}

// ---- SaveClientNotebookDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SaveClientNotebookDto {
    pub client_id: Uuid,
    pub entries: Vec<NotebookEntryDto>,
}

impl From<SaveClientNotebookDto> for SaveClientNotebookInput {
    fn from(dto: SaveClientNotebookDto) -> Self {
        SaveClientNotebookInput {
            client_id: ClientId(dto.client_id),
            entries: dto.entries.into_iter().map(Into::into).collect(),
        }
    }
}

// ---- ClientJournalEntryDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ClientJournalEntryDto {
    pub id: Uuid,
    pub client_id: Uuid,
    pub entry_date: NaiveDate,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ClientJournalEntry> for ClientJournalEntryDto {
    fn from(e: &ClientJournalEntry) -> Self {
        Self {
            id: e.id.0,
            client_id: e.client_id.0,
            entry_date: e.entry_date,
            content: e.content.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

impl From<ClientJournalEntry> for ClientJournalEntryDto {
    fn from(e: ClientJournalEntry) -> Self {
        (&e).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewJournalEntryDto {
    pub client_id: Uuid,
    pub entry_date: NaiveDate,
    pub content: String,
}

impl From<NewJournalEntryDto> for NewJournalEntry {
    fn from(dto: NewJournalEntryDto) -> Self {
        NewJournalEntry {
            client_id: ClientId(dto.client_id),
            entry_date: dto.entry_date,
            content: dto.content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateJournalEntryDto {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub content: String,
}

impl From<UpdateJournalEntryDto> for UpdateJournalEntryInput {
    fn from(dto: UpdateJournalEntryDto) -> Self {
        UpdateJournalEntryInput {
            id: JournalEntryId(dto.id),
            entry_date: dto.entry_date,
            content: dto.content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_round_trip() {
        let domain = NotebookSection {
            id: NotebookSectionId::new(),
            name: "Background".into(),
            sort_order: 2,
        };
        let dto: NotebookSectionDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.name, "Background");
        assert_eq!(dto.sort_order, 2);
    }

    #[test]
    fn notebook_entry_dto_round_trips() {
        let id = Uuid::new_v4();
        let dto = NotebookEntryDto {
            section_id: id,
            content: "notes".into(),
        };
        let domain: NotebookEntry = dto.clone().into();
        assert_eq!(domain.section_id.0, id);
        let back: NotebookEntryDto = (&domain).into();
        assert_eq!(back, dto);
    }

    #[test]
    fn save_client_notebook_dto_maps_to_input() {
        let dto = SaveClientNotebookDto {
            client_id: Uuid::new_v4(),
            entries: vec![NotebookEntryDto {
                section_id: Uuid::new_v4(),
                content: "x".into(),
            }],
        };
        let input: SaveClientNotebookInput = dto.into();
        assert_eq!(input.entries.len(), 1);
    }

    #[test]
    fn journal_entry_round_trip() {
        let domain = ClientJournalEntry {
            id: JournalEntryId::new(),
            client_id: ClientId::new(),
            entry_date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            content: "session".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let dto: ClientJournalEntryDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.content, "session");
    }

    #[test]
    fn rename_section_dto_maps_to_input() {
        let id = Uuid::new_v4();
        let dto = RenameNotebookSectionDto {
            id,
            name: "New".into(),
        };
        let input: RenameNotebookSectionInput = dto.into();
        assert_eq!(input.id.0, id);
        assert_eq!(input.name, "New");
    }
}
