use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::client::ClientId;

// ---- NotebookSection ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotebookSectionId(pub Uuid);

impl NotebookSectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NotebookSectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NotebookSectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookSection {
    pub id: NotebookSectionId,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotebookSectionError {
    #[error("section name cannot be empty")]
    EmptyName,
}

impl NotebookSection {
    pub fn create(name: String, sort_order: i32) -> Result<Self, NotebookSectionError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(NotebookSectionError::EmptyName);
        }
        Ok(Self {
            id: NotebookSectionId::new(),
            name,
            sort_order,
        })
    }

    pub fn rename(&mut self, name: String) -> Result<(), NotebookSectionError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(NotebookSectionError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}

// ---- ClientNotebook ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookEntry {
    pub section_id: NotebookSectionId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientNotebook {
    pub client_id: ClientId,
    pub entries: Vec<NotebookEntry>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotebookError {
    #[error("notebook has duplicate entries for the same section")]
    DuplicateSection,
}

impl ClientNotebook {
    pub fn create(
        client_id: ClientId,
        entries: Vec<NotebookEntry>,
        now: DateTime<Utc>,
    ) -> Result<Self, NotebookError> {
        let trimmed = normalize_entries(entries)?;
        Ok(Self {
            client_id,
            entries: trimmed,
            updated_at: now,
        })
    }

    pub fn replace_entries(
        &mut self,
        entries: Vec<NotebookEntry>,
        now: DateTime<Utc>,
    ) -> Result<(), NotebookError> {
        self.entries = normalize_entries(entries)?;
        self.updated_at = now;
        Ok(())
    }
}

fn normalize_entries(
    entries: Vec<NotebookEntry>,
) -> Result<Vec<NotebookEntry>, NotebookError> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for e in entries {
        if !seen.insert(e.section_id) {
            return Err(NotebookError::DuplicateSection);
        }
        out.push(NotebookEntry {
            section_id: e.section_id,
            content: e.content.trim_end().to_string(),
        });
    }
    Ok(out)
}

// ---- ClientJournalEntry ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JournalEntryId(pub Uuid);

impl JournalEntryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JournalEntryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JournalEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientJournalEntry {
    pub id: JournalEntryId,
    pub client_id: ClientId,
    pub entry_date: NaiveDate,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum JournalEntryError {
    #[error("journal entry content cannot be empty")]
    EmptyContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalEntry {
    pub client_id: ClientId,
    pub entry_date: NaiveDate,
    pub content: String,
}

impl ClientJournalEntry {
    pub fn create(
        input: NewJournalEntry,
        now: DateTime<Utc>,
    ) -> Result<Self, JournalEntryError> {
        let content = input.content.trim().to_string();
        if content.is_empty() {
            return Err(JournalEntryError::EmptyContent);
        }
        Ok(Self {
            id: JournalEntryId::new(),
            client_id: input.client_id,
            entry_date: input.entry_date,
            content,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn edit(
        &mut self,
        entry_date: NaiveDate,
        content: String,
        now: DateTime<Utc>,
    ) -> Result<(), JournalEntryError> {
        let content = content.trim().to_string();
        if content.is_empty() {
            return Err(JournalEntryError::EmptyContent);
        }
        self.entry_date = entry_date;
        self.content = content;
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-04-14T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 14).unwrap()
    }

    // --- NotebookSection ---

    #[test]
    fn create_section_trims_name() {
        let s = NotebookSection::create("  Background  ".into(), 0).unwrap();
        assert_eq!(s.name, "Background");
        assert_eq!(s.sort_order, 0);
    }

    #[test]
    fn create_section_rejects_empty_name() {
        let err = NotebookSection::create("  ".into(), 0).unwrap_err();
        assert_eq!(err, NotebookSectionError::EmptyName);
    }

    #[test]
    fn rename_section_rejects_empty() {
        let mut s = NotebookSection::create("Start".into(), 0).unwrap();
        assert_eq!(
            s.rename("".into()).unwrap_err(),
            NotebookSectionError::EmptyName
        );
        assert_eq!(s.name, "Start");
    }

    #[test]
    fn rename_section_trims() {
        let mut s = NotebookSection::create("Start".into(), 0).unwrap();
        s.rename("  Finish  ".into()).unwrap();
        assert_eq!(s.name, "Finish");
    }

    // --- ClientNotebook ---

    #[test]
    fn create_notebook_dedupe_rejects_duplicate_sections() {
        let section = NotebookSectionId::new();
        let err = ClientNotebook::create(
            ClientId::new(),
            vec![
                NotebookEntry {
                    section_id: section,
                    content: "A".into(),
                },
                NotebookEntry {
                    section_id: section,
                    content: "B".into(),
                },
            ],
            now(),
        )
        .unwrap_err();
        assert_eq!(err, NotebookError::DuplicateSection);
    }

    #[test]
    fn create_notebook_allows_empty_content() {
        let section = NotebookSectionId::new();
        let a = ClientNotebook::create(
            ClientId::new(),
            vec![NotebookEntry {
                section_id: section,
                content: "".into(),
            }],
            now(),
        )
        .unwrap();
        assert_eq!(a.entries[0].content, "");
    }

    #[test]
    fn create_notebook_trims_trailing_whitespace() {
        let a = ClientNotebook::create(
            ClientId::new(),
            vec![NotebookEntry {
                section_id: NotebookSectionId::new(),
                content: "hello   \n".into(),
            }],
            now(),
        )
        .unwrap();
        assert_eq!(a.entries[0].content, "hello");
    }

    #[test]
    fn replace_entries_updates_timestamp_and_rejects_duplicates() {
        let mut a = ClientNotebook::create(ClientId::new(), vec![], now()).unwrap();
        let section = NotebookSectionId::new();
        a.replace_entries(
            vec![NotebookEntry {
                section_id: section,
                content: "first".into(),
            }],
            now(),
        )
        .unwrap();
        assert_eq!(a.entries.len(), 1);

        let err = a
            .replace_entries(
                vec![
                    NotebookEntry {
                        section_id: section,
                        content: "a".into(),
                    },
                    NotebookEntry {
                        section_id: section,
                        content: "b".into(),
                    },
                ],
                now(),
            )
            .unwrap_err();
        assert_eq!(err, NotebookError::DuplicateSection);
    }

    // --- ClientJournalEntry ---

    #[test]
    fn create_journal_entry_valid() {
        let e = ClientJournalEntry::create(
            NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(),
                content: "  session notes  ".into(),
            },
            now(),
        )
        .unwrap();
        assert_eq!(e.content, "session notes");
        assert_eq!(e.created_at, now());
        assert_eq!(e.updated_at, now());
    }

    #[test]
    fn create_journal_entry_rejects_empty_content() {
        let err = ClientJournalEntry::create(
            NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(),
                content: "  ".into(),
            },
            now(),
        )
        .unwrap_err();
        assert_eq!(err, JournalEntryError::EmptyContent);
    }

    #[test]
    fn edit_journal_entry_updates_fields_and_timestamp() {
        let mut e = ClientJournalEntry::create(
            NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(),
                content: "old".into(),
            },
            now(),
        )
        .unwrap();
        let later = DateTime::parse_from_rfc3339("2026-04-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        e.edit(
            NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
            "new content".into(),
            later,
        )
        .unwrap();
        assert_eq!(e.content, "new content");
        assert_eq!(e.entry_date, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
        assert_eq!(e.updated_at, later);
    }

    #[test]
    fn edit_journal_entry_rejects_empty_content() {
        let mut e = ClientJournalEntry::create(
            NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(),
                content: "keep".into(),
            },
            now(),
        )
        .unwrap();
        let err = e.edit(date(), "  ".into(), now()).unwrap_err();
        assert_eq!(err, JournalEntryError::EmptyContent);
        assert_eq!(e.content, "keep");
    }
}
