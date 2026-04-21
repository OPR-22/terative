use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{ClientRepository, ListClientsQuery, Page};
use crate::application::RepoError;
use crate::domain::client::{Client, ClientId, ContactEntry, ContactEntryId};

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

fn parse_uuid(s: &str) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Reads the bare client row. Contact lists are loaded separately.
fn row_to_bare_client(row: &Row<'_>) -> rusqlite::Result<Client> {
    let id = ClientId(parse_uuid(&row.get::<_, String>("id")?)?);
    let created_at_str: String = row.get("created_at")?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    let referred_by = match row.get::<_, Option<String>>("referred_by")? {
        Some(s) => Some(ClientId(parse_uuid(&s)?)),
        None => None,
    };
    Ok(Client {
        id,
        name: row.get("name")?,
        emails: Vec::new(),
        phones: Vec::new(),
        address: row.get("address")?,
        notes: row.get("notes")?,
        referred_by,
        active: row.get::<_, i64>("active")? != 0,
        created_at,
    })
}

fn row_to_contact(row: &Row<'_>) -> rusqlite::Result<(Uuid, ContactEntry)> {
    let client_id = parse_uuid(&row.get::<_, String>("client_id")?)?;
    let id = ContactEntryId(parse_uuid(&row.get::<_, String>("id")?)?);
    Ok((
        client_id,
        ContactEntry {
            id,
            value: row.get("value")?,
            label: row.get("label")?,
            is_default: row.get::<_, i64>("is_default")? != 0,
        },
    ))
}

fn load_contacts(
    conn: &Connection,
    table: &str,
    client_ids: &[ClientId],
) -> Result<HashMap<Uuid, Vec<ContactEntry>>, RepoError> {
    if client_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders: Vec<String> = (1..=client_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT id, client_id, value, label, is_default
         FROM {table}
         WHERE client_id IN ({})
         ORDER BY is_default DESC, sort_order ASC, value ASC",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql).map_err(map_err)?;
    let ids: Vec<String> = client_ids.iter().map(|id| id.to_string()).collect();
    let params: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(params.as_slice(), row_to_contact)
        .map_err(map_err)?;
    let mut out: HashMap<Uuid, Vec<ContactEntry>> = HashMap::new();
    for row in rows {
        let (cid, entry) = row.map_err(map_err)?;
        out.entry(cid).or_default().push(entry);
    }
    Ok(out)
}

fn write_contacts(
    conn: &Connection,
    table: &str,
    client_id: ClientId,
    entries: &[ContactEntry],
) -> Result<(), RepoError> {
    conn.execute(
        &format!("DELETE FROM {table} WHERE client_id = ?1"),
        params![client_id.to_string()],
    )
    .map_err(map_err)?;
    for (idx, entry) in entries.iter().enumerate() {
        conn.execute(
            &format!(
                "INSERT INTO {table} (id, client_id, value, label, is_default, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            ),
            params![
                entry.id.to_string(),
                client_id.to_string(),
                entry.value,
                entry.label,
                entry.is_default as i64,
                idx as i64,
            ],
        )
        .map_err(map_err)?;
    }
    Ok(())
}

fn hydrate_contacts(conn: &Connection, clients: &mut [Client]) -> Result<(), RepoError> {
    if clients.is_empty() {
        return Ok(());
    }
    let ids: Vec<ClientId> = clients.iter().map(|c| c.id).collect();
    let mut emails = load_contacts(conn, "client_emails", &ids)?;
    let mut phones = load_contacts(conn, "client_phones", &ids)?;
    for c in clients.iter_mut() {
        c.emails = emails.remove(&c.id.0).unwrap_or_default();
        c.phones = phones.remove(&c.id.0).unwrap_or_default();
    }
    Ok(())
}

impl ClientRepository for SqliteClientRepository {
    fn insert(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO clients (id, name, address, notes, referred_by, active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                c.id.to_string(),
                c.name,
                c.address,
                c.notes,
                c.referred_by.map(|r| r.to_string()),
                c.active as i64,
                c.created_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        write_contacts(&conn, "client_emails", c.id, &c.emails)?;
        write_contacts(&conn, "client_phones", c.id, &c.phones)?;
        Ok(())
    }

    fn update(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE clients
                 SET name = ?2, address = ?3, notes = ?4, referred_by = ?5, active = ?6
                 WHERE id = ?1",
                params![
                    c.id.to_string(),
                    c.name,
                    c.address,
                    c.notes,
                    c.referred_by.map(|r| r.to_string()),
                    c.active as i64,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        write_contacts(&conn, "client_emails", c.id, &c.emails)?;
        write_contacts(&conn, "client_phones", c.id, &c.phones)?;
        Ok(())
    }

    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError> {
        let conn = self.db.lock();
        let mut client = conn
            .query_row(
                "SELECT id, name, address, notes, referred_by, active, created_at
                 FROM clients WHERE id = ?1",
                params![id.to_string()],
                row_to_bare_client,
            )
            .optional()
            .map_err(map_err)?;
        if let Some(ref mut c) = client {
            let mut slice = std::slice::from_mut(c);
            hydrate_contacts(&conn, &mut slice)?;
        }
        Ok(client)
    }

    fn list(&self, query: ListClientsQuery) -> Result<Page<Client>, RepoError> {
        let conn = self.db.lock();

        let mut where_clause = String::new();
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
            where_clause = format!(" WHERE {}", clauses.join(" AND "));
        }

        // Count total matching rows.
        let count_sql = format!("SELECT COUNT(*) FROM clients{where_clause}");
        let total: u64 = if let Some(ref pat) = search_pattern {
            conn.query_row(&count_sql, params![pat], |r| r.get::<_, i64>(0))
        } else {
            conn.query_row(&count_sql, [], |r| r.get::<_, i64>(0))
        }
        .map_err(map_err)? as u64;

        // Fetch the page.
        let offset = query.pagination.offset();
        let limit = query.pagination.per_page as u64;
        let select_sql = format!(
            "SELECT id, name, address, notes, referred_by, active, created_at FROM clients{where_clause} \
             ORDER BY name COLLATE NOCASE ASC LIMIT {limit} OFFSET {offset}"
        );

        let mut stmt = conn.prepare(&select_sql).map_err(map_err)?;
        let rows = if let Some(pat) = search_pattern {
            stmt.query_map(params![pat], row_to_bare_client)
        } else {
            stmt.query_map([], row_to_bare_client)
        }
        .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        drop(stmt);
        hydrate_contacts(&conn, &mut out)?;
        Ok(Page::new(out, total, &query.pagination))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::client::{NewClient, NewContactEntry};

    fn email(value: &str, is_default: bool) -> NewContactEntry {
        NewContactEntry {
            value: value.into(),
            label: None,
            is_default,
        }
    }

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
    fn insert_persists_email_and_phone_lists() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                emails: vec![email("a@x.com", false), email("b@x.com", true)],
                phones: vec![NewContactEntry {
                    value: "555-0100".into(),
                    label: Some("Mobile".into()),
                    is_default: true,
                }],
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.emails.len(), 2);
        assert_eq!(loaded.default_email(), Some("b@x.com"));
        assert_eq!(loaded.phones.len(), 1);
        assert_eq!(loaded.phones[0].label.as_deref(), Some("Mobile"));
    }

    #[test]
    fn update_replaces_contact_lists() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                emails: vec![email("old@x.com", true)],
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&c).unwrap();
        c.replace_emails(vec![email("new@x.com", true), email("other@x.com", false)])
            .unwrap();
        repo.update(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.emails.len(), 2);
        assert_eq!(loaded.default_email(), Some("new@x.com"));
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
    fn referred_by_round_trips() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let referrer = make_client("Referrer");
        repo.insert(&referrer).unwrap();
        let mut referred = make_client("Referred");
        referred.set_referred_by(Some(referrer.id)).unwrap();
        repo.insert(&referred).unwrap();
        let loaded = repo.get(referred.id).unwrap().unwrap();
        assert_eq!(loaded.referred_by, Some(referrer.id));
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
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].name, "Beta");
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
                ..Default::default()
            })
            .unwrap();
        assert_eq!(list.data.len(), 1);
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
                ..Default::default()
            })
            .unwrap();
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].name, "Acme Corp");
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
            list.data.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "mid", "Zeta"]
        );
    }
}
