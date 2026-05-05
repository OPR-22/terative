use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{ClientAttributeValues, ClientRepository, ListClientsQuery, Page};
use crate::application::RepoError;
use crate::domain::client::{
    Client, ClientAddress, ClientAddressId, ClientId, ClientKind, ContactEntry, ContactEntryId,
};

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

const CLIENT_COLUMNS: &str = "id, kind, name, contact_name, tax_id, registration_number, \
                              notes, referred_by, date_of_birth, sex, gender, pronouns, \
                              occupation, language, archived_at, created_at";

/// Reads the bare client row. Contact lists / addresses are loaded separately.
fn row_to_bare_client(row: &Row<'_>) -> rusqlite::Result<Client> {
    let id = ClientId(parse_uuid(&row.get::<_, String>("id")?)?);
    let kind_str: String = row.get("kind")?;
    let kind = ClientKind::parse(&kind_str).unwrap_or(ClientKind::Individual);
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
    let date_of_birth = match row.get::<_, Option<String>>("date_of_birth")? {
        Some(s) => Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?),
        None => None,
    };
    let archived_at = match row.get::<_, Option<String>>("archived_at")? {
        None => None,
        Some(s) => Some(
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc),
        ),
    };
    Ok(Client {
        id,
        kind,
        name: row.get("name")?,
        contact_name: row.get("contact_name")?,
        tax_id: row.get("tax_id")?,
        registration_number: row.get("registration_number")?,
        emails: Vec::new(),
        phones: Vec::new(),
        addresses: Vec::new(),
        notes: row.get("notes")?,
        referred_by,
        date_of_birth,
        sex: row.get("sex")?,
        gender: row.get("gender")?,
        pronouns: row.get("pronouns")?,
        occupation: row.get("occupation")?,
        language: row.get("language")?,
        archived_at,
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

fn row_to_address(row: &Row<'_>) -> rusqlite::Result<(Uuid, ClientAddress)> {
    let client_id = parse_uuid(&row.get::<_, String>("client_id")?)?;
    let id = ClientAddressId(parse_uuid(&row.get::<_, String>("id")?)?);
    Ok((
        client_id,
        ClientAddress {
            id,
            label: row.get("label")?,
            street: row.get("street")?,
            apt_suite: row.get("apt_suite")?,
            city: row.get("city")?,
            state_province: row.get("state_province")?,
            postal_code: row.get("postal_code")?,
            country: row.get("country")?,
            is_billing: row.get::<_, i64>("is_billing")? != 0,
            is_shipping: row.get::<_, i64>("is_shipping")? != 0,
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

fn load_addresses(
    conn: &Connection,
    client_ids: &[ClientId],
) -> Result<HashMap<Uuid, Vec<ClientAddress>>, RepoError> {
    if client_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders: Vec<String> = (1..=client_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT id, client_id, label, street, apt_suite, city, state_province,
                postal_code, country, is_billing, is_shipping
         FROM client_addresses
         WHERE client_id IN ({})
         ORDER BY sort_order ASC, street ASC",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql).map_err(map_err)?;
    let ids: Vec<String> = client_ids.iter().map(|id| id.to_string()).collect();
    let params: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(params.as_slice(), row_to_address)
        .map_err(map_err)?;
    let mut out: HashMap<Uuid, Vec<ClientAddress>> = HashMap::new();
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

fn write_addresses(
    conn: &Connection,
    client_id: ClientId,
    entries: &[ClientAddress],
) -> Result<(), RepoError> {
    // Delete-then-insert: with everything cleared first, the partial unique
    // partial indexes WHERE is_billing=1 / WHERE is_shipping=1 can't
    // be tripped by a transient duplicate.
    conn.execute(
        "DELETE FROM client_addresses WHERE client_id = ?1",
        params![client_id.to_string()],
    )
    .map_err(map_err)?;
    for (idx, addr) in entries.iter().enumerate() {
        conn.execute(
            "INSERT INTO client_addresses
             (id, client_id, label, street, apt_suite, city, state_province,
              postal_code, country, is_billing, is_shipping, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                addr.id.to_string(),
                client_id.to_string(),
                addr.label,
                addr.street,
                addr.apt_suite,
                addr.city,
                addr.state_province,
                addr.postal_code,
                addr.country,
                addr.is_billing as i64,
                addr.is_shipping as i64,
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
    let mut addresses = load_addresses(conn, &ids)?;
    for c in clients.iter_mut() {
        c.emails = emails.remove(&c.id.0).unwrap_or_default();
        c.phones = phones.remove(&c.id.0).unwrap_or_default();
        c.addresses = addresses.remove(&c.id.0).unwrap_or_default();
    }
    Ok(())
}

impl ClientRepository for SqliteClientRepository {
    fn insert(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO clients
             (id, kind, name, contact_name, tax_id, registration_number, notes, referred_by,
              date_of_birth, sex, gender, pronouns, occupation, language, archived_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                c.id.to_string(),
                c.kind.as_str(),
                c.name,
                c.contact_name,
                c.tax_id,
                c.registration_number,
                c.notes,
                c.referred_by.map(|r| r.to_string()),
                c.date_of_birth.map(|d| d.format("%Y-%m-%d").to_string()),
                c.sex,
                c.gender,
                c.pronouns,
                c.occupation,
                c.language,
                c.archived_at.map(|d| d.to_rfc3339()),
                c.created_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        write_contacts(&conn, "client_emails", c.id, &c.emails)?;
        write_contacts(&conn, "client_phones", c.id, &c.phones)?;
        write_addresses(&conn, c.id, &c.addresses)?;
        Ok(())
    }

    fn update(&self, c: &Client) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE clients
                 SET kind = ?2, name = ?3, contact_name = ?4, tax_id = ?5,
                     registration_number = ?6, notes = ?7, referred_by = ?8,
                     date_of_birth = ?9, sex = ?10, gender = ?11,
                     pronouns = ?12, occupation = ?13, language = ?14,
                     archived_at = ?15
                 WHERE id = ?1",
                params![
                    c.id.to_string(),
                    c.kind.as_str(),
                    c.name,
                    c.contact_name,
                    c.tax_id,
                    c.registration_number,
                    c.notes,
                    c.referred_by.map(|r| r.to_string()),
                    c.date_of_birth.map(|d| d.format("%Y-%m-%d").to_string()),
                    c.sex,
                    c.gender,
                    c.pronouns,
                    c.occupation,
                    c.language,
                    c.archived_at.map(|d| d.to_rfc3339()),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        write_contacts(&conn, "client_emails", c.id, &c.emails)?;
        write_contacts(&conn, "client_phones", c.id, &c.phones)?;
        write_addresses(&conn, c.id, &c.addresses)?;
        Ok(())
    }

    fn get(&self, id: ClientId) -> Result<Option<Client>, RepoError> {
        let conn = self.db.lock();
        let mut client = conn
            .query_row(
                &format!("SELECT {CLIENT_COLUMNS} FROM clients WHERE id = ?1"),
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
        if !query.include_archived {
            clauses.push("archived_at IS NULL");
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
            "SELECT {CLIENT_COLUMNS} FROM clients{where_clause} \
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

    fn names_for(
        &self,
        ids: &[ClientId],
    ) -> Result<HashMap<ClientId, String>, RepoError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let conn = self.db.lock();
        // Bind each id as a numbered placeholder. SQLite's IN-list has no
        // upper bound for our ~tens-of-rows reads, but if we ever batch
        // thousands here we should chunk into IN(...) groups under the 999
        // bind-variable default.
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT id, name FROM clients WHERE id IN ({})",
            placeholders.join(", ")
        );
        let id_strs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let params: Vec<&dyn rusqlite::ToSql> =
            id_strs.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params.as_slice(), |row| {
                let id = parse_uuid(&row.get::<_, String>("id")?)?;
                let name: String = row.get("name")?;
                Ok((ClientId(id), name))
            })
            .map_err(map_err)?;
        let mut out = HashMap::new();
        for r in rows {
            let (id, name) = r.map_err(map_err)?;
            out.insert(id, name);
        }
        Ok(out)
    }

    fn distinct_attribute_values(&self) -> Result<ClientAttributeValues, RepoError> {
        let conn = self.db.lock();
        let read = |column: &str| -> Result<Vec<String>, RepoError> {
            // Trim and ignore blank-after-trim values so historical whitespace
            // entries don't pollute the suggestion list.
            let sql = format!(
                "SELECT DISTINCT TRIM({column}) AS v
                 FROM clients
                 WHERE {column} IS NOT NULL AND TRIM({column}) != ''
                 ORDER BY v COLLATE NOCASE ASC"
            );
            let mut stmt = conn.prepare(&sql).map_err(map_err)?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>("v"))
                .map_err(map_err)?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(map_err)?);
            }
            Ok(out)
        };
        Ok(ClientAttributeValues {
            gender: read("gender")?,
            pronouns: read("pronouns")?,
            occupation: read("occupation")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::client::{NewClient, NewClientAddress, NewContactEntry};

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
        assert!(!loaded.is_archived());
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
    fn insert_persists_addresses() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![NewClientAddress {
                    label: Some("HQ".into()),
                    street: "1 Way".into(),
                    city: "Brussels".into(),
                    postal_code: "1000".into(),
                    country: "BE".into(),
                    is_billing: true,
                    is_shipping: true,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.addresses.len(), 1);
        assert_eq!(loaded.billing_address().unwrap().street, "1 Way");
        // Same row carries the shipping flag.
        assert_eq!(loaded.shipping_address().unwrap().street, "1 Way");
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
    fn update_replaces_addresses() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut c = Client::create(
            NewClient {
                name: "Acme".into(),
                addresses: vec![NewClientAddress {
                    street: "Old St".into(),
                    city: "Brussels".into(),
                    postal_code: "1000".into(),
                    country: "BE".into(),
                    is_billing: true,
                    is_shipping: false,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&c).unwrap();
        c.replace_addresses(vec![NewClientAddress {
            street: "New St".into(),
            city: "Brussels".into(),
            postal_code: "1000".into(),
            country: "BE".into(),
            is_billing: true,
            is_shipping: false,
            ..Default::default()
        }])
        .unwrap();
        repo.update(&c).unwrap();
        let loaded = repo.get(c.id).unwrap().unwrap();
        assert_eq!(loaded.addresses.len(), 1);
        assert_eq!(loaded.addresses[0].street, "New St");
    }

    #[test]
    fn db_rejects_two_billing_addresses_for_same_client() {
        // Sanity check on the partial unique index. Bypasses
        // sanitize_addresses by writing raw SQL.
        let db = open_memory();
        let repo = SqliteClientRepository::new(db.clone());
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        let first = ClientAddressId::new();
        let second = ClientAddressId::new();
        let cid = c.id.to_string();
        let conn = db.lock();
        conn.execute(
            "INSERT INTO client_addresses (id, client_id, label, street, apt_suite, city, state_province, postal_code, country, is_billing, is_shipping, sort_order)
             VALUES (?1, ?2, NULL, 'A St', NULL, 'Brussels', NULL, '1000', 'BE', 1, 0, 0)",
            params![first.to_string(), cid],
        )
        .unwrap();
        let err = conn.execute(
            "INSERT INTO client_addresses (id, client_id, label, street, apt_suite, city, state_province, postal_code, country, is_billing, is_shipping, sort_order)
             VALUES (?1, ?2, NULL, 'B St', NULL, 'Brussels', NULL, '1000', 'BE', 1, 0, 1)",
            params![second.to_string(), cid],
        );
        assert!(err.is_err(), "second billing row must be rejected");
    }

    #[test]
    fn db_allows_combined_billing_and_shipping_row() {
        // A single row with both flags shouldn't trip either uniqueness
        // index — partial indexes only count the rows that match.
        let db = open_memory();
        let repo = SqliteClientRepository::new(db.clone());
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        let conn = db.lock();
        conn.execute(
            "INSERT INTO client_addresses (id, client_id, label, street, apt_suite, city, state_province, postal_code, country, is_billing, is_shipping, sort_order)
             VALUES (?1, ?2, 'HQ', 'A St', NULL, 'Brussels', NULL, '1000', 'BE', 1, 1, 0)",
            params![ClientAddressId::new().to_string(), c.id.to_string()],
        )
        .unwrap();
    }

    #[test]
    fn db_allows_address_with_no_active_role() {
        // Stored-but-not-active addresses are valid. The client just
        // hasn't picked this one as the current billing or shipping.
        let db = open_memory();
        let repo = SqliteClientRepository::new(db.clone());
        let c = make_client("Acme");
        repo.insert(&c).unwrap();
        let conn = db.lock();
        conn.execute(
            "INSERT INTO client_addresses (id, client_id, label, street, apt_suite, city, state_province, postal_code, country, is_billing, is_shipping, sort_order)
             VALUES (?1, ?2, 'Old site', 'A St', NULL, 'Brussels', NULL, '1000', 'BE', 0, 0, 0)",
            params![ClientAddressId::new().to_string(), c.id.to_string()],
        )
        .unwrap();
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
    fn list_excludes_archived_by_default() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut a = make_client("Alpha");
        let b = make_client("Beta");
        a.archived_at = Some(Utc::now());
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        let list = repo.list(ListClientsQuery::default()).unwrap();
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0].name, "Beta");
    }

    #[test]
    fn list_includes_archived_when_requested() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let mut a = make_client("Alpha");
        a.archived_at = Some(Utc::now());
        repo.insert(&a).unwrap();
        let list = repo
            .list(ListClientsQuery {
                include_archived: true,
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
                include_archived: false,
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

    #[test]
    fn distinct_attribute_values_returns_unique_sorted_non_null() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);

        let mut a = make_client("A");
        a.gender = Some("woman".into());
        a.pronouns = Some("she/her".into());
        a.occupation = Some("Architect".into());
        repo.insert(&a).unwrap();

        let mut b = make_client("B");
        b.gender = Some("man".into());
        b.pronouns = Some("he/him".into());
        b.occupation = Some("architect".into()); // dup, different case
        repo.insert(&b).unwrap();

        let mut c = make_client("C");
        c.gender = Some("woman".into()); // dup
        c.pronouns = Some("they/them".into());
        c.occupation = None;
        repo.insert(&c).unwrap();

        let values = repo.distinct_attribute_values().unwrap();
        assert_eq!(values.gender, vec!["man", "woman"]);
        assert_eq!(values.pronouns, vec!["he/him", "she/her", "they/them"]);
        // Case-insensitive sort but distinct keeps both spellings.
        assert!(values.occupation.iter().any(|s| s == "Architect"));
        assert!(values.occupation.iter().any(|s| s == "architect"));
    }

    #[test]
    fn distinct_attribute_values_empty_db() {
        let db = open_memory();
        let repo = SqliteClientRepository::new(db);
        let values = repo.distinct_attribute_values().unwrap();
        assert!(values.gender.is_empty());
        assert!(values.pronouns.is_empty());
        assert!(values.occupation.is_empty());
    }
}
