use std::sync::Arc;

use chrono::Utc;

use crate::application::ports::{
    ClientNotebookRepository, ClientJournalRepository, NotebookSectionRepository,
};
use crate::application::AppError;
use crate::domain::client::ClientId;
use crate::domain::notebook::{
    NotebookEntry, ClientNotebook, ClientJournalEntry, JournalEntryId, NewJournalEntry,
    NotebookSection, NotebookSectionId,
};
use chrono::NaiveDate;

// ---- section management (Settings) ----

pub struct CreateNotebookSection {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl CreateNotebookSection {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, name: String) -> Result<NotebookSection, AppError> {
        let next_sort_order = self.repo.max_sort_order()? + 1;
        let section = NotebookSection::create(name, next_sort_order)?;
        self.repo.insert(&section)?;
        Ok(section)
    }
}

#[derive(Debug, Clone)]
pub struct RenameNotebookSectionInput {
    pub id: NotebookSectionId,
    pub name: String,
}

pub struct RenameNotebookSection {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl RenameNotebookSection {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(
        &self,
        input: RenameNotebookSectionInput,
    ) -> Result<NotebookSection, AppError> {
        let mut section = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        section.rename(input.name)?;
        self.repo.update(&section)?;
        Ok(section)
    }
}

pub struct DeleteNotebookSection {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl DeleteNotebookSection {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: NotebookSectionId) -> Result<(), AppError> {
        if self.repo.get(id)?.is_none() {
            return Err(AppError::NotFound);
        }
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct CountSectionEntries {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl CountSectionEntries {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: NotebookSectionId) -> Result<u64, AppError> {
        Ok(self.repo.count_entries(id)?)
    }
}

pub struct ReorderNotebookSections {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl ReorderNotebookSections {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, ordered_ids: Vec<NotebookSectionId>) -> Result<(), AppError> {
        self.repo.reorder(&ordered_ids)?;
        Ok(())
    }
}

pub struct ListNotebookSections {
    repo: Arc<dyn NotebookSectionRepository>,
}

impl ListNotebookSections {
    pub fn new(repo: Arc<dyn NotebookSectionRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self) -> Result<Vec<NotebookSection>, AppError> {
        Ok(self.repo.list()?)
    }
}

// ---- notebook (per client) ----

/// A section + its (possibly empty) content for one client. Returned by
/// `GetClientNotebook`, which merges the global section list with the sparse
/// stored rows so every section shows up, in order.
#[derive(Debug, Clone)]
pub struct ClientNotebookView {
    pub client_id: ClientId,
    pub sections: Vec<ClientNotebookSection>,
}

#[derive(Debug, Clone)]
pub struct ClientNotebookSection {
    pub section: NotebookSection,
    pub content: String,
}

pub struct GetClientNotebook {
    sections: Arc<dyn NotebookSectionRepository>,
    notebook: Arc<dyn ClientNotebookRepository>,
}

impl GetClientNotebook {
    pub fn new(
        sections: Arc<dyn NotebookSectionRepository>,
        notebook: Arc<dyn ClientNotebookRepository>,
    ) -> Self {
        Self {
            sections,
            notebook,
        }
    }
    pub fn execute(&self, client_id: ClientId) -> Result<ClientNotebookView, AppError> {
        let sections = self.sections.list()?;
        let stored = self.notebook.load(client_id)?;
        let by_section: std::collections::HashMap<NotebookSectionId, String> = stored
            .entries
            .into_iter()
            .map(|e| (e.section_id, e.content))
            .collect();
        let merged: Vec<ClientNotebookSection> = sections
            .into_iter()
            .map(|section| {
                let content = by_section
                    .get(&section.id)
                    .cloned()
                    .unwrap_or_default();
                ClientNotebookSection { section, content }
            })
            .collect();
        Ok(ClientNotebookView {
            client_id,
            sections: merged,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SaveClientNotebookInput {
    pub client_id: ClientId,
    pub entries: Vec<NotebookEntry>,
}

pub struct SaveClientNotebook {
    notebook: Arc<dyn ClientNotebookRepository>,
}

impl SaveClientNotebook {
    pub fn new(notebook: Arc<dyn ClientNotebookRepository>) -> Self {
        Self { notebook }
    }
    pub fn execute(&self, input: SaveClientNotebookInput) -> Result<(), AppError> {
        let aggregate = ClientNotebook::create(input.client_id, input.entries, Utc::now())?;
        self.notebook.save(&aggregate)?;
        Ok(())
    }
}

// ---- journal (per client) ----

pub struct CreateJournalEntry {
    repo: Arc<dyn ClientJournalRepository>,
}

impl CreateJournalEntry {
    pub fn new(repo: Arc<dyn ClientJournalRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: NewJournalEntry) -> Result<ClientJournalEntry, AppError> {
        let entry = ClientJournalEntry::create(input, Utc::now())?;
        self.repo.insert(&entry)?;
        Ok(entry)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateJournalEntryInput {
    pub id: JournalEntryId,
    pub entry_date: NaiveDate,
    pub content: String,
}

pub struct UpdateJournalEntry {
    repo: Arc<dyn ClientJournalRepository>,
}

impl UpdateJournalEntry {
    pub fn new(repo: Arc<dyn ClientJournalRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(
        &self,
        input: UpdateJournalEntryInput,
    ) -> Result<ClientJournalEntry, AppError> {
        let mut entry = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        entry.edit(input.entry_date, input.content, Utc::now())?;
        self.repo.update(&entry)?;
        Ok(entry)
    }
}

pub struct DeleteJournalEntry {
    repo: Arc<dyn ClientJournalRepository>,
}

impl DeleteJournalEntry {
    pub fn new(repo: Arc<dyn ClientJournalRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: JournalEntryId) -> Result<(), AppError> {
        if self.repo.get(id)?.is_none() {
            return Err(AppError::NotFound);
        }
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct ListClientJournal {
    repo: Arc<dyn ClientJournalRepository>,
}

impl ListClientJournal {
    pub fn new(repo: Arc<dyn ClientJournalRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, client_id: ClientId) -> Result<Vec<ClientJournalEntry>, AppError> {
        Ok(self.repo.list_for_client(client_id)?)
    }
}

pub struct GetJournalEntry {
    repo: Arc<dyn ClientJournalRepository>,
}

impl GetJournalEntry {
    pub fn new(repo: Arc<dyn ClientJournalRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: JournalEntryId) -> Result<ClientJournalEntry, AppError> {
        self.repo.get(id)?.ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    // ---- fakes ----

    #[derive(Default)]
    struct InMemorySectionRepo {
        inner: Mutex<HashMap<NotebookSectionId, NotebookSection>>,
    }
    impl NotebookSectionRepository for InMemorySectionRepo {
        fn insert(&self, s: &NotebookSection) -> Result<(), RepoError> {
            self.inner.lock().insert(s.id, s.clone());
            Ok(())
        }
        fn update(&self, s: &NotebookSection) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&s.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(s.id, s.clone());
            Ok(())
        }
        fn get(&self, id: NotebookSectionId) -> Result<Option<NotebookSection>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self) -> Result<Vec<NotebookSection>, RepoError> {
            let mut v: Vec<NotebookSection> = self.inner.lock().values().cloned().collect();
            v.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then(a.name.cmp(&b.name)));
            Ok(v)
        }
        fn delete(&self, id: NotebookSectionId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
        fn count_entries(&self, _: NotebookSectionId) -> Result<u64, RepoError> {
            Ok(0)
        }
        fn reorder(&self, ordered_ids: &[NotebookSectionId]) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            for (idx, id) in ordered_ids.iter().enumerate() {
                if let Some(s) = g.get_mut(id) {
                    s.sort_order = idx as i32;
                }
            }
            Ok(())
        }
        fn max_sort_order(&self) -> Result<i32, RepoError> {
            Ok(self
                .inner
                .lock()
                .values()
                .map(|s| s.sort_order)
                .max()
                .unwrap_or(-1))
        }
    }

    #[derive(Default)]
    struct InMemoryNotebookRepo {
        inner: Mutex<HashMap<ClientId, ClientNotebook>>,
    }
    impl ClientNotebookRepository for InMemoryNotebookRepo {
        fn save(&self, a: &ClientNotebook) -> Result<(), RepoError> {
            // Upsert semantics: merge into existing entries by section id.
            let mut g = self.inner.lock();
            let stored = g.entry(a.client_id).or_insert_with(|| ClientNotebook {
                client_id: a.client_id,
                entries: vec![],
                updated_at: a.updated_at,
            });
            let mut by_section: HashMap<NotebookSectionId, String> = stored
                .entries
                .iter()
                .map(|e| (e.section_id, e.content.clone()))
                .collect();
            for e in &a.entries {
                by_section.insert(e.section_id, e.content.clone());
            }
            stored.entries = by_section
                .into_iter()
                .map(|(section_id, content)| NotebookEntry {
                    section_id,
                    content,
                })
                .collect();
            stored.updated_at = a.updated_at;
            Ok(())
        }
        fn load(&self, client_id: ClientId) -> Result<ClientNotebook, RepoError> {
            Ok(self
                .inner
                .lock()
                .get(&client_id)
                .cloned()
                .unwrap_or_else(|| ClientNotebook {
                    client_id,
                    entries: vec![],
                    updated_at: Utc::now(),
                }))
        }
    }

    #[derive(Default)]
    struct InMemoryJournalRepo {
        inner: Mutex<HashMap<JournalEntryId, ClientJournalEntry>>,
    }
    impl ClientJournalRepository for InMemoryJournalRepo {
        fn insert(&self, e: &ClientJournalEntry) -> Result<(), RepoError> {
            self.inner.lock().insert(e.id, e.clone());
            Ok(())
        }
        fn update(&self, e: &ClientJournalEntry) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&e.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(e.id, e.clone());
            Ok(())
        }
        fn get(&self, id: JournalEntryId) -> Result<Option<ClientJournalEntry>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list_for_client(
            &self,
            id: ClientId,
        ) -> Result<Vec<ClientJournalEntry>, RepoError> {
            let mut v: Vec<ClientJournalEntry> = self
                .inner
                .lock()
                .values()
                .filter(|e| e.client_id == id)
                .cloned()
                .collect();
            v.sort_by(|a, b| b.entry_date.cmp(&a.entry_date));
            Ok(v)
        }
        fn delete(&self, id: JournalEntryId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    // ---- section management tests ----

    #[test]
    fn create_section_auto_assigns_sort_order_at_end() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let uc = CreateNotebookSection::new(repo.clone());
        let a = uc.execute("A".into()).unwrap();
        let b = uc.execute("B".into()).unwrap();
        let c = uc.execute("C".into()).unwrap();
        assert_eq!(a.sort_order, 0);
        assert_eq!(b.sort_order, 1);
        assert_eq!(c.sort_order, 2);
    }

    #[test]
    fn create_section_rejects_empty_name() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let err = CreateNotebookSection::new(repo)
            .execute("  ".into())
            .unwrap_err();
        assert!(matches!(err, AppError::NotebookSection(_)));
    }

    #[test]
    fn rename_section_persists_new_name() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let created = CreateNotebookSection::new(repo.clone())
            .execute("Old".into())
            .unwrap();
        let renamed = RenameNotebookSection::new(repo.clone())
            .execute(RenameNotebookSectionInput {
                id: created.id,
                name: "New".into(),
            })
            .unwrap();
        assert_eq!(renamed.name, "New");
    }

    #[test]
    fn rename_missing_section_returns_not_found() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let err = RenameNotebookSection::new(repo)
            .execute(RenameNotebookSectionInput {
                id: NotebookSectionId::new(),
                name: "X".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn delete_missing_section_returns_not_found() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let err = DeleteNotebookSection::new(repo)
            .execute(NotebookSectionId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn reorder_sections_updates_order() {
        let repo = Arc::new(InMemorySectionRepo::default());
        let create = CreateNotebookSection::new(repo.clone());
        let a = create.execute("A".into()).unwrap();
        let b = create.execute("B".into()).unwrap();
        let c = create.execute("C".into()).unwrap();

        ReorderNotebookSections::new(repo.clone())
            .execute(vec![c.id, a.id, b.id])
            .unwrap();

        let list = ListNotebookSections::new(repo).execute().unwrap();
        assert_eq!(
            list.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["C", "A", "B"]
        );
    }

    // ---- notebook tests ----

    #[test]
    fn get_client_notebook_merges_sections_with_sparse_stored_rows() {
        let section_repo = Arc::new(InMemorySectionRepo::default());
        let notebook_repo = Arc::new(InMemoryNotebookRepo::default());
        let create = CreateNotebookSection::new(section_repo.clone());
        let a = create.execute("A".into()).unwrap();
        let b = create.execute("B".into()).unwrap();
        let c = create.execute("C".into()).unwrap();

        let client = ClientId::new();
        // Only B has stored content; A and C must come back empty.
        SaveClientNotebook::new(notebook_repo.clone())
            .execute(SaveClientNotebookInput {
                client_id: client,
                entries: vec![NotebookEntry {
                    section_id: b.id,
                    content: "mid content".into(),
                }],
            })
            .unwrap();

        let view = GetClientNotebook::new(section_repo, notebook_repo)
            .execute(client)
            .unwrap();
        assert_eq!(view.sections.len(), 3);
        assert_eq!(view.sections[0].section.id, a.id);
        assert_eq!(view.sections[0].content, "");
        assert_eq!(view.sections[1].section.id, b.id);
        assert_eq!(view.sections[1].content, "mid content");
        assert_eq!(view.sections[2].section.id, c.id);
        assert_eq!(view.sections[2].content, "");
    }

    #[test]
    fn save_client_notebook_rejects_duplicate_section() {
        let notebook_repo = Arc::new(InMemoryNotebookRepo::default());
        let section = NotebookSectionId::new();
        let err = SaveClientNotebook::new(notebook_repo)
            .execute(SaveClientNotebookInput {
                client_id: ClientId::new(),
                entries: vec![
                    NotebookEntry {
                        section_id: section,
                        content: "a".into(),
                    },
                    NotebookEntry {
                        section_id: section,
                        content: "b".into(),
                    },
                ],
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Notebook(_)));
    }

    // ---- journal tests ----

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn create_journal_entry_persists() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let e = CreateJournalEntry::new(repo.clone())
            .execute(NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(2026, 4, 14),
                content: "session".into(),
            })
            .unwrap();
        assert_eq!(e.content, "session");
        assert_eq!(repo.inner.lock().len(), 1);
    }

    #[test]
    fn create_journal_entry_rejects_empty_content() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let err = CreateJournalEntry::new(repo)
            .execute(NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(2026, 4, 14),
                content: "".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::JournalEntry(_)));
    }

    #[test]
    fn update_journal_entry_edits_and_rejects_empty() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let created = CreateJournalEntry::new(repo.clone())
            .execute(NewJournalEntry {
                client_id: ClientId::new(),
                entry_date: date(2026, 4, 14),
                content: "before".into(),
            })
            .unwrap();
        let updated = UpdateJournalEntry::new(repo.clone())
            .execute(UpdateJournalEntryInput {
                id: created.id,
                entry_date: date(2026, 4, 15),
                content: "after".into(),
            })
            .unwrap();
        assert_eq!(updated.content, "after");
        assert_eq!(updated.entry_date, date(2026, 4, 15));

        let err = UpdateJournalEntry::new(repo)
            .execute(UpdateJournalEntryInput {
                id: created.id,
                entry_date: date(2026, 4, 15),
                content: "  ".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::JournalEntry(_)));
    }

    #[test]
    fn update_missing_journal_entry_returns_not_found() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let err = UpdateJournalEntry::new(repo)
            .execute(UpdateJournalEntryInput {
                id: JournalEntryId::new(),
                entry_date: date(2026, 4, 14),
                content: "x".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn delete_missing_journal_entry_returns_not_found() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let err = DeleteJournalEntry::new(repo)
            .execute(JournalEntryId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn list_client_journal_returns_entries_for_client_only() {
        let repo = Arc::new(InMemoryJournalRepo::default());
        let me = ClientId::new();
        let other = ClientId::new();
        let create = CreateJournalEntry::new(repo.clone());
        create
            .execute(NewJournalEntry {
                client_id: me,
                entry_date: date(2026, 4, 14),
                content: "mine".into(),
            })
            .unwrap();
        create
            .execute(NewJournalEntry {
                client_id: other,
                entry_date: date(2026, 4, 14),
                content: "theirs".into(),
            })
            .unwrap();

        let list = ListClientJournal::new(repo).execute(me).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].content, "mine");
    }
}
