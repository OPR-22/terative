use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::ClientJournalRepository;
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::notebook::{ClientJournalEntry, JournalEntryId};

pub struct SqliteClientJournalRepository {
    db: Db,
}

impl SqliteClientJournalRepository {
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

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<ClientJournalEntry> {
    let id_str: String = row.get("id")?;
    let id = JournalEntryId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let client_id_str: String = row.get("client_id")?;
    let client_id = ClientId(Uuid::parse_str(&client_id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let entry_date_str: String = row.get("entry_date")?;
    let entry_date = NaiveDate::parse_from_str(&entry_date_str, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let created_at_str: String = row.get("created_at")?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    let updated_at_str: String = row.get("updated_at")?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    Ok(ClientJournalEntry {
        id,
        client_id,
        entry_date,
        content: row.get("content")?,
        created_at,
        updated_at,
    })
}

const SELECT_COLS: &str = "id, client_id, entry_date, content, created_at, updated_at";

impl ClientJournalRepository for SqliteClientJournalRepository {
    fn insert(&self, e: &ClientJournalEntry) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO client_journal_entries
                (id, client_id, entry_date, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                e.id.to_string(),
                e.client_id.to_string(),
                e.entry_date.format("%Y-%m-%d").to_string(),
                e.content,
                e.created_at.to_rfc3339(),
                e.updated_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, e: &ClientJournalEntry) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE client_journal_entries
                 SET entry_date = ?2, content = ?3, updated_at = ?4
                 WHERE id = ?1",
                params![
                    e.id.to_string(),
                    e.entry_date.format("%Y-%m-%d").to_string(),
                    e.content,
                    e.updated_at.to_rfc3339(),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: JournalEntryId) -> Result<Option<ClientJournalEntry>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_COLS} FROM client_journal_entries WHERE id = ?1");
        conn.query_row(&sql, params![id.to_string()], row_to_entry)
            .optional()
            .map_err(map_err)
    }

    fn list_for_client(&self, id: ClientId) -> Result<Vec<ClientJournalEntry>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM client_journal_entries
             WHERE client_id = ?1
             ORDER BY entry_date DESC, created_at DESC"
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params![id.to_string()], row_to_entry)
            .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn delete(&self, id: JournalEntryId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM client_journal_entries WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(map_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::SqliteClientRepository;
    use crate::application::ports::ClientRepository as _;
    use crate::domain::client::{Client, NewClient};
    use crate::domain::notebook::NewJournalEntry;

    fn seed_client(db: &Db) -> ClientId {
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        SqliteClientRepository::new(db.clone()).insert(&client).unwrap();
        client.id
    }

    fn make(client_id: ClientId, date_str: &str, content: &str) -> ClientJournalEntry {
        ClientJournalEntry::create(
            NewJournalEntry {
                client_id,
                entry_date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(),
                content: content.into(),
            },
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db);
        let e = make(client_id, "2026-04-14", "session one");
        repo.insert(&e).unwrap();
        let loaded = repo.get(e.id).unwrap().unwrap();
        assert_eq!(loaded.content, "session one");
        assert_eq!(loaded.entry_date, e.entry_date);
    }

    #[test]
    fn list_for_client_sorted_newest_first() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db);
        repo.insert(&make(client_id, "2026-01-01", "old")).unwrap();
        repo.insert(&make(client_id, "2026-04-14", "recent")).unwrap();
        repo.insert(&make(client_id, "2026-02-01", "mid")).unwrap();
        let list = repo.list_for_client(client_id).unwrap();
        assert_eq!(
            list.iter().map(|e| e.content.as_str()).collect::<Vec<_>>(),
            vec!["recent", "mid", "old"]
        );
    }

    #[test]
    fn update_modifies_fields() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db);
        let mut e = make(client_id, "2026-04-14", "before");
        repo.insert(&e).unwrap();
        e.content = "after".into();
        e.entry_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        repo.update(&e).unwrap();
        let loaded = repo.get(e.id).unwrap().unwrap();
        assert_eq!(loaded.content, "after");
        assert_eq!(loaded.entry_date, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db);
        let e = make(client_id, "2026-04-14", "ghost");
        assert!(matches!(repo.update(&e), Err(RepoError::NotFound)));
    }

    #[test]
    fn delete_removes_entry() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db);
        let e = make(client_id, "2026-04-14", "bye");
        repo.insert(&e).unwrap();
        repo.delete(e.id).unwrap();
        assert!(repo.get(e.id).unwrap().is_none());
    }

    #[test]
    fn client_cascade_removes_journal_entries() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteClientJournalRepository::new(db.clone());
        repo.insert(&make(client_id, "2026-04-14", "note")).unwrap();
        db.lock()
            .execute(
                "DELETE FROM clients WHERE id = ?1",
                rusqlite::params![client_id.to_string()],
            )
            .unwrap();
        assert!(repo.list_for_client(client_id).unwrap().is_empty());
    }
}
