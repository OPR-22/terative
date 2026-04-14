use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::NotebookSectionRepository;
use crate::application::RepoError;
use crate::domain::notebook::{NotebookSection, NotebookSectionId};

pub struct SqliteNotebookSectionRepository {
    db: Db,
}

impl SqliteNotebookSectionRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn map_err(e: rusqlite::Error) -> RepoError {
    match e {
        rusqlite::Error::QueryReturnedNoRows => RepoError::NotFound,
        rusqlite::Error::SqliteFailure(ref f, _)
            if f.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            RepoError::Conflict(e.to_string())
        }
        other => RepoError::Storage(other.to_string()),
    }
}

fn row_to_section(row: &Row<'_>) -> rusqlite::Result<NotebookSection> {
    let id_str: String = row.get("id")?;
    let id = NotebookSectionId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    Ok(NotebookSection {
        id,
        name: row.get("name")?,
        sort_order: row.get::<_, i64>("sort_order")? as i32,
    })
}

impl NotebookSectionRepository for SqliteNotebookSectionRepository {
    fn insert(&self, s: &NotebookSection) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO notebook_sections (id, name, sort_order) VALUES (?1, ?2, ?3)",
            params![s.id.to_string(), s.name, s.sort_order as i64],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, s: &NotebookSection) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE notebook_sections SET name = ?2, sort_order = ?3 WHERE id = ?1",
                params![s.id.to_string(), s.name, s.sort_order as i64],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: NotebookSectionId) -> Result<Option<NotebookSection>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, sort_order FROM notebook_sections WHERE id = ?1",
            params![id.to_string()],
            row_to_section,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self) -> Result<Vec<NotebookSection>, RepoError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, sort_order FROM notebook_sections
                 ORDER BY sort_order ASC, name COLLATE NOCASE ASC",
            )
            .map_err(map_err)?;
        let rows = stmt.query_map([], row_to_section).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn delete(&self, id: NotebookSectionId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM notebook_sections WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn count_entries(&self, id: NotebookSectionId) -> Result<u64, RepoError> {
        let conn = self.db.lock();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM client_notebook_entries WHERE section_id = ?1",
                params![id.to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        Ok(count as u64)
    }

    fn reorder(&self, ordered_ids: &[NotebookSectionId]) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        for (idx, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE notebook_sections SET sort_order = ?2 WHERE id = ?1",
                params![id.to_string(), idx as i64],
            )
            .map_err(map_err)?;
        }
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn max_sort_order(&self) -> Result<i32, RepoError> {
        let conn = self.db.lock();
        let max: Option<i64> = conn
            .query_row(
                "SELECT MAX(sort_order) FROM notebook_sections",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(map_err)?
            .flatten();
        Ok(max.map(|n| n as i32).unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    fn make(name: &str, sort_order: i32) -> NotebookSection {
        let mut s = NotebookSection::create(name.into(), sort_order).unwrap();
        s.sort_order = sort_order;
        s
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        let s = make("Background", 0);
        repo.insert(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Background");
        assert_eq!(loaded.sort_order, 0);
    }

    #[test]
    fn update_changes_name_and_sort_order() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        let mut s = make("Old", 0);
        repo.insert(&s).unwrap();
        s.name = "New".into();
        s.sort_order = 5;
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.name, "New");
        assert_eq!(loaded.sort_order, 5);
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        let s = make("Ghost", 0);
        assert!(matches!(repo.update(&s), Err(RepoError::NotFound)));
    }

    #[test]
    fn list_sorts_by_sort_order_then_name() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        repo.insert(&make("Zed", 2)).unwrap();
        repo.insert(&make("Alpha", 0)).unwrap();
        repo.insert(&make("Beta", 1)).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(
            list.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "Beta", "Zed"]
        );
    }

    #[test]
    fn reorder_assigns_contiguous_sort_order() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        let a = make("A", 0);
        let b = make("B", 1);
        let c = make("C", 2);
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        repo.insert(&c).unwrap();
        repo.reorder(&[c.id, a.id, b.id]).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(
            list.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["C", "A", "B"]
        );
        assert_eq!(list[0].sort_order, 0);
        assert_eq!(list[1].sort_order, 1);
        assert_eq!(list[2].sort_order, 2);
    }

    #[test]
    fn max_sort_order_returns_minus_one_when_empty() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        assert_eq!(repo.max_sort_order().unwrap(), -1);
    }

    #[test]
    fn max_sort_order_returns_highest() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        repo.insert(&make("A", 2)).unwrap();
        repo.insert(&make("B", 5)).unwrap();
        repo.insert(&make("C", 3)).unwrap();
        assert_eq!(repo.max_sort_order().unwrap(), 5);
    }

    #[test]
    fn count_entries_is_zero_when_empty() {
        let db = open_memory();
        let repo = SqliteNotebookSectionRepository::new(db);
        let s = make("X", 0);
        repo.insert(&s).unwrap();
        assert_eq!(repo.count_entries(s.id).unwrap(), 0);
    }

    #[test]
    fn delete_cascades_to_notebook_entries() {
        use crate::adapters::sqlite::{
            SqliteClientNotebookRepository, SqliteClientRepository,
        };
        use crate::application::ports::{
            ClientNotebookRepository as _, ClientRepository as _,
        };
        use crate::domain::client::{Client, NewClient};
        use crate::domain::notebook::{ClientNotebook, NotebookEntry};
        use chrono::Utc;

        let db = open_memory();
        let section_repo = SqliteNotebookSectionRepository::new(db.clone());
        let client_repo = SqliteClientRepository::new(db.clone());
        let notebook_repo = SqliteClientNotebookRepository::new(db.clone());

        let section = make("Background", 0);
        section_repo.insert(&section).unwrap();

        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo.insert(&client).unwrap();

        let notebook = ClientNotebook::create(
            client.id,
            vec![NotebookEntry {
                section_id: section.id,
                content: "notes".into(),
            }],
            Utc::now(),
        )
        .unwrap();
        notebook_repo.save(&notebook).unwrap();

        assert_eq!(section_repo.count_entries(section.id).unwrap(), 1);
        section_repo.delete(section.id).unwrap();
        // Cascade removed the notebook row.
        let loaded = notebook_repo.load(client.id).unwrap();
        assert!(loaded.entries.is_empty());
    }
}
