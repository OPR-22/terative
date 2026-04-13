use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{ClientRepository, ListClientsQuery};
use crate::application::RepoError;
use crate::domain::client::{Client, ClientId};

pub struct SqliteClientRepository {
    db: Db,
}

impl SqliteClientRepository {
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

fn row_to_client(row: &Row<'_>) -> rusqlite::Result<Client> {
    let id_str: String = row.get("id")?;
    let id = ClientId(
        Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?,
    );
    let created_at_str: String = row.get("created_at")?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?
        .with_timezone(&Utc);
    Ok(Client {
        id,
        name: row.get("name")?,
        email: row.get("email")?,
        address: row.get("address")?,
        phone: row.get("phone")?,
        notes: row.get("notes")?,
        active: row.get::<_, i64>("active")? != 0,
        created_at,
    })
}

impl ClientRepository for SqliteClientRepository {
    fn insert(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO clients (id, name, email, address, phone, notes, active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                c.id.to_string(),
                c.name,
                c.email,
                c.address,
                c.phone,
                c.notes,
                c.active as i64,
                c.created_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE clients
                 SET name = ?2, email = ?3, address = ?4, phone = ?5, notes = ?6, active = ?7
                 WHERE id = ?1",
                params![
                    c.id.to_string(),
                    c.name,
                    c.email,
                    c.address,
                    c.phone,
                    c.notes,
                    c.active as i64,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, email, address, phone, notes, active, created_at
             FROM clients WHERE id = ?1",
            params![id.to_string()],
            row_to_client,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self, query: ListClientsQuery) -> Result<Vec<Client>, RepoError> {
        let conn = self.db.lock();
        let mut sql = String::from(
            "SELECT id, name, email, address, phone, notes, active, created_at FROM clients",
        );
        let mut clauses: Vec<&str> = Vec::new();
        if !query.include_inactive {
            clauses.push("active = 1");
        }
        let search_pattern: Option<String> = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.trim().to_lowercase()));
        if search_pattern.is_some() {
            clauses.push("LOWER(name) LIKE ?1");
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY name COLLATE NOCASE ASC");

        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = if let Some(pat) = search_pattern {
            stmt.query_map(params![pat], row_to_client)
        } else {
            stmt.query_map([], row_to_client)
        }
        .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn has_invoices(&self, id: ClientId) -> Result<bool, RepoError> {
        let conn = self.db.lock();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM invoices WHERE client_id = ?1",
                params![id.to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        Ok(count > 0)
    }

    fn delete(&self, id: ClientId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM clients WHERE id = ?1", params![id.to_string()])
            .map_err(map_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::client::NewClient;

    fn make_client(name: &str) -> Client {
        Client::create(
            NewClient {
                name: name.into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Acme");
        assert_eq!(loaded.id, c.id);
        assert!(loaded.active);
    }

    #[test]
    fn get_missing_returns_none() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        assert!(repo.get(ClientId::new()).unwrap().is_none());
    }

    #[test]
    fn update_modifies_fields() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut c = make_client("Old");
        repo.insert(&c).unwrap();
        c.name = "New".into();
        c.email = Some("new@x.com".into());
        repo.update(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.name, "New");
        assert_eq!(loaded.email.as_deref(), Some("new@x.com"));
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = make_client("Ghost");
        let err = repo.update(&c).unwrap_err();
        assert!(matches!(err, RepoError::NotFound));
    }

    #[test]
    fn list_excludes_inactive_by_default() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut a = make_client("Alpha");
        let b = make_client("Beta");
        a.active = false;
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        let list = repo.list(ListClientsQuery::default()).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Beta");
    }

    #[test]
    fn list_includes_inactive_when_requested() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut a = make_client("Alpha");
        a.active = false;
        repo.insert(&a).unwrap();
        let list = repo
            .list(ListClientsQuery {
                include_inactive: true,
                search: None,
            })
            .unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn list_search_case_insensitive() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        repo.insert(&make_client("Acme Corp")).unwrap();
        repo.insert(&make_client("Globex")).unwrap();
        let list = repo
            .list(ListClientsQuery {
                search: Some("ACM".into()),
                include_inactive: false,
            })
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Acme Corp");
    }

    #[test]
    fn list_sorts_by_name_ascending() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        repo.insert(&make_client("Zeta")).unwrap();
        repo.insert(&make_client("alpha")).unwrap();
        repo.insert(&make_client("mid")).unwrap();
        let list = repo.list(ListClientsQuery::default()).unwrap();
        assert_eq!(
            list.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "mid", "Zeta"]
        );
    }

    #[test]
    fn delete_removes_row() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        repo.delete(c.id).unwrap();
        assert!(repo.get(c.id).unwrap().is_none());
    }

    #[test]
    fn has_invoices_is_false_when_none_exist() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        assert!(!repo.has_invoices(c.id).unwrap());
    }
}
