use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::ClientNotebookRepository;
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::notebook::{ClientNotebook, NotebookEntry, NotebookSectionId};

pub struct SqliteClientNotebookRepository {
    db: Db,
}

impl SqliteClientNotebookRepository {
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

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<NotebookEntry> {
    let section_id_str: String = row.get("section_id")?;
    let section_id = NotebookSectionId(Uuid::parse_str(&section_id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    Ok(NotebookEntry {
        section_id,
        content: row.get("content")?,
    })
}

impl ClientNotebookRepository for SqliteClientNotebookRepository {
    fn save(&self, notebook: &ClientNotebook) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        let updated_at = notebook.updated_at.to_rfc3339();
        for entry in &notebook.entries {
            tx.execute(
                "INSERT INTO client_notebook_entries
                    (id, client_id, section_id, content, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(client_id, section_id) DO UPDATE SET
                     content = excluded.content,
                     updated_at = excluded.updated_at",
                params![
                    Uuid::new_v4().to_string(),
                    notebook.client_id.to_string(),
                    entry.section_id.to_string(),
                    entry.content,
                    updated_at,
                ],
            )
            .map_err(map_err)?;
        }
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn load(&self, client_id: ClientId) -> Result<ClientNotebook, RepoError> {
        let conn = self.db.lock();

        // Fetch the latest updated_at for stamping; if no rows exist, use now.
        let updated_at: Option<String> = conn
            .query_row(
                "SELECT MAX(updated_at) FROM client_notebook_entries WHERE client_id = ?1",
                params![client_id.to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        let updated_at = updated_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let mut stmt = conn
            .prepare(
                "SELECT section_id, content FROM client_notebook_entries
                 WHERE client_id = ?1",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map(params![client_id.to_string()], row_to_entry)
            .map_err(map_err)?;
        let mut entries = Vec::new();
        for r in rows {
            entries.push(r.map_err(map_err)?);
        }
        Ok(ClientNotebook {
            client_id,
            entries,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::{SqliteClientRepository, SqliteNotebookSectionRepository};
    use crate::application::ports::{ClientRepository as _, NotebookSectionRepository as _};
    use crate::domain::client::{Client, NewClient};
    use crate::domain::notebook::NotebookSection;

    fn seed_client_and_sections(
        db: &Db,
    ) -> (ClientId, NotebookSectionId, NotebookSectionId) {
        let client_repo = SqliteClientRepository::new(db.clone());
        let section_repo = SqliteNotebookSectionRepository::new(db.clone());
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        client_repo.insert(&client).unwrap();
        let a = NotebookSection::create("Background".into(), 0).unwrap();
        let b = NotebookSection::create("Goals".into(), 1).unwrap();
        section_repo.insert(&a).unwrap();
        section_repo.insert(&b).unwrap();
        (client.id, a.id, b.id)
    }

    #[test]
    fn save_then_load_round_trip() {
        let db = open_memory();
        let (client_id, a, b) = seed_client_and_sections(&db);
        let repo = SqliteClientNotebookRepository::new(db);

        let notebook = ClientNotebook::create(
            client_id,
            vec![
                NotebookEntry {
                    section_id: a,
                    content: "history".into(),
                },
                NotebookEntry {
                    section_id: b,
                    content: "targets".into(),
                },
            ],
            Utc::now(),
        )
        .unwrap();
        repo.save(&notebook).unwrap();

        let loaded = repo.load(client_id).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        let by_section: std::collections::HashMap<_, _> =
            loaded.entries.iter().map(|e| (e.section_id, &e.content)).collect();
        assert_eq!(by_section.get(&a).unwrap().as_str(), "history");
        assert_eq!(by_section.get(&b).unwrap().as_str(), "targets");
    }

    #[test]
    fn save_is_upsert_preserves_unchanged_and_updates_changed() {
        let db = open_memory();
        let (client_id, a, b) = seed_client_and_sections(&db);
        let repo = SqliteClientNotebookRepository::new(db);

        let first = ClientNotebook::create(
            client_id,
            vec![
                NotebookEntry {
                    section_id: a,
                    content: "v1".into(),
                },
                NotebookEntry {
                    section_id: b,
                    content: "keep".into(),
                },
            ],
            Utc::now(),
        )
        .unwrap();
        repo.save(&first).unwrap();

        let second = ClientNotebook::create(
            client_id,
            vec![
                NotebookEntry {
                    section_id: a,
                    content: "v2".into(),
                },
                NotebookEntry {
                    section_id: b,
                    content: "keep".into(),
                },
            ],
            Utc::now(),
        )
        .unwrap();
        repo.save(&second).unwrap();

        let loaded = repo.load(client_id).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        let by_section: std::collections::HashMap<_, _> =
            loaded.entries.iter().map(|e| (e.section_id, &e.content)).collect();
        assert_eq!(by_section.get(&a).unwrap().as_str(), "v2");
        assert_eq!(by_section.get(&b).unwrap().as_str(), "keep");
    }

    #[test]
    fn load_empty_returns_empty_vec() {
        let db = open_memory();
        let (client_id, _, _) = seed_client_and_sections(&db);
        let repo = SqliteClientNotebookRepository::new(db);
        let loaded = repo.load(client_id).unwrap();
        assert!(loaded.entries.is_empty());
        assert_eq!(loaded.client_id, client_id);
    }

    #[test]
    fn client_cascade_removes_notebook_rows() {
        let db = open_memory();
        let (client_id, a, _) = seed_client_and_sections(&db);
        let notebook_repo = SqliteClientNotebookRepository::new(db.clone());
        notebook_repo
            .save(
                &ClientNotebook::create(
                    client_id,
                    vec![NotebookEntry {
                        section_id: a,
                        content: "x".into(),
                    }],
                    Utc::now(),
                )
                .unwrap(),
            )
            .unwrap();

        let client_repo = SqliteClientRepository::new(db.clone());
        client_repo.delete(client_id).unwrap();

        let loaded = notebook_repo.load(client_id).unwrap();
        assert!(loaded.entries.is_empty());
    }
}
