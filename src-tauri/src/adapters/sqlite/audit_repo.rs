//! SQLite adapter for [`AuditRepository`]. Append-only: `insert` plus the
//! three read shapes the UI needs. Mirrors `email_log_repo` — `metadata_json`
//! is stored and returned as opaque `TEXT`; the projector owns its shape.

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{AuditRepository, Page, PaginationParams};
use crate::application::RepoError;
use crate::domain::audit::{Audit, AuditId};
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;

pub struct SqliteAuditRepository {
    db: Db,
}

impl SqliteAuditRepository {
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

fn parse_uuid<T>(s: &str, wrap: impl Fn(Uuid) -> T) -> rusqlite::Result<T> {
    Uuid::parse_str(s).map(wrap).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn row_to_activity(row: &Row<'_>) -> rusqlite::Result<Audit> {
    let id_str: String = row.get("id")?;
    let id: AuditId = parse_uuid(&id_str, AuditId)?;
    let client_id_str: Option<String> = row.get("client_id")?;
    let client_id = client_id_str
        .as_deref()
        .map(|s| parse_uuid(s, ClientId))
        .transpose()?;
    let invoice_id_str: Option<String> = row.get("invoice_id")?;
    let invoice_id = invoice_id_str
        .as_deref()
        .map(|s| parse_uuid(s, InvoiceId))
        .transpose()?;
    let occurred_at_str: String = row.get("occurred_at")?;
    let occurred_at = DateTime::parse_from_rfc3339(&occurred_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    Ok(Audit {
        id,
        event_type: row.get("event_type")?,
        entity_type: row.get("entity_type")?,
        entity_id: row.get("entity_id")?,
        client_id,
        invoice_id,
        metadata_json: row.get("metadata_json")?,
        occurred_at,
    })
}

const SELECT_COLS: &str =
    "id, event_type, entity_type, entity_id, client_id, invoice_id, metadata_json, occurred_at";

impl AuditRepository for SqliteAuditRepository {
    fn insert(&self, audit: &Audit) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO audits
               (id, event_type, entity_type, entity_id, client_id, invoice_id, metadata_json, occurred_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                audit.id.to_string(),
                audit.event_type,
                audit.entity_type,
                audit.entity_id,
                audit.client_id.map(|c| c.to_string()),
                audit.invoice_id.map(|i| i.to_string()),
                audit.metadata_json,
                audit.occurred_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn paginate_recent(&self, params: &PaginationParams) -> Result<Page<Audit>, RepoError> {
        let conn = self.db.lock();
        let total: u64 = conn
            .query_row("SELECT COUNT(*) FROM audits", [], |r| r.get::<_, i64>(0))
            .map_err(map_err)? as u64;
        let limit = params.per_page as u64;
        let offset = params.offset();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM audits \
             ORDER BY occurred_at DESC, id DESC LIMIT {limit} OFFSET {offset}",
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_activity).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(Page::new(out, total, params))
    }

    fn paginate_by_client(
        &self,
        client_id: ClientId,
        params: &PaginationParams,
    ) -> Result<Page<Audit>, RepoError> {
        let conn = self.db.lock();
        let total: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audits WHERE client_id = ?1",
                params![client_id.to_string()],
                |r| r.get::<_, i64>(0),
            )
            .map_err(map_err)? as u64;
        let limit = params.per_page as u64;
        let offset = params.offset();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM audits \
             WHERE client_id = ?1 \
             ORDER BY occurred_at DESC, id DESC LIMIT {limit} OFFSET {offset}",
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params![client_id.to_string()], row_to_activity)
            .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(Page::new(out, total, params))
    }

    fn paginate_by_invoice(
        &self,
        invoice_id: InvoiceId,
        params: &PaginationParams,
    ) -> Result<Page<Audit>, RepoError> {
        let conn = self.db.lock();
        let total: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audits WHERE invoice_id = ?1",
                params![invoice_id.to_string()],
                |r| r.get::<_, i64>(0),
            )
            .map_err(map_err)? as u64;
        let limit = params.per_page as u64;
        let offset = params.offset();
        // Newest first — consistent with `paginate_recent` /
        // `paginate_by_client`. The timeline UI renders most-recent at the
        // top across every audit surface.
        let sql = format!(
            "SELECT {SELECT_COLS} FROM audits \
             WHERE invoice_id = ?1 \
             ORDER BY occurred_at DESC, id DESC LIMIT {limit} OFFSET {offset}",
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params![invoice_id.to_string()], row_to_activity)
            .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(Page::new(out, total, params))
    }

    fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "DELETE FROM audits WHERE occurred_at < ?1",
                params![cutoff.to_rfc3339()],
            )
            .map_err(map_err)?;
        Ok(affected as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::SqliteClientRepository;
    use crate::application::ports::ClientRepository;
    use crate::domain::audit::NewAudit;
    use crate::domain::client::{Client, NewClient};
    use chrono::Duration;

    fn page(per_page: u32) -> PaginationParams {
        PaginationParams { page: 1, per_page }
    }

    fn seed_client(repo: &SqliteClientRepository, name: &str) -> Client {
        let client = Client::create(
            NewClient {
                name: name.into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        repo.insert(&client).unwrap();
        client
    }

    fn audit(
        event_type: &str,
        client_id: Option<ClientId>,
        invoice_id: Option<InvoiceId>,
        occurred_at: DateTime<Utc>,
    ) -> Audit {
        Audit::record(NewAudit {
            event_type: event_type.into(),
            entity_type: "test".into(),
            entity_id: None,
            client_id,
            invoice_id,
            metadata_json: "{}".into(),
            occurred_at,
        })
        .unwrap()
    }

    #[test]
    fn insert_and_paginate_recent_round_trip_newest_first() {
        let db = open_memory();
        let repo = SqliteAuditRepository::new(db);
        let now = Utc::now();
        repo.insert(&audit("a.old", None, None, now - Duration::hours(2)))
            .unwrap();
        repo.insert(&audit("a.new", None, None, now)).unwrap();
        repo.insert(&audit("a.mid", None, None, now - Duration::hours(1)))
            .unwrap();

        let listed = repo.paginate_recent(&page(10)).unwrap();
        assert_eq!(listed.total, 3);
        assert_eq!(listed.data.len(), 3);
        assert_eq!(listed.data[0].event_type, "a.new");
        assert_eq!(listed.data[1].event_type, "a.mid");
        assert_eq!(listed.data[2].event_type, "a.old");
        assert_eq!(listed.next, None);
    }

    #[test]
    fn s() {
        let db = open_memory();
        let repo = SqliteAuditRepository::new(db);
        let now = Utc::now();
        for i in 0..5 {
            repo.insert(&audit("a.x", None, None, now - Duration::minutes(i)))
                .unwrap();
        }
        let listed = repo.paginate_recent(&page(2)).unwrap();
        assert_eq!(listed.data.len(), 2);
        assert_eq!(listed.total, 5);
        assert_eq!(listed.next, Some(2));
    }

    #[test]
    fn paginate_recent_second_page_returns_remaining_rows() {
        let db = open_memory();
        let repo = SqliteAuditRepository::new(db);
        let now = Utc::now();
        for i in 0..5 {
            repo.insert(&audit("a.x", None, None, now - Duration::minutes(i)))
                .unwrap();
        }
        let p2 = repo
            .paginate_recent(&PaginationParams { page: 2, per_page: 2 })
            .unwrap();
        assert_eq!(p2.data.len(), 2);
        assert_eq!(p2.previous, Some(1));
        assert_eq!(p2.next, Some(3));
    }

    #[test]
    fn paginate_by_client_filters_and_orders_newest_first() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let repo = SqliteAuditRepository::new(db);
        let alice = seed_client(&clients, "Alice");
        let bob = seed_client(&clients, "Bob");
        let now = Utc::now();
        repo.insert(&audit("a.1", Some(alice.id), None, now - Duration::hours(1)))
            .unwrap();
        repo.insert(&audit("a.2", Some(alice.id), None, now)).unwrap();
        repo.insert(&audit("b.1", Some(bob.id), None, now)).unwrap();

        let alice_rows = repo.paginate_by_client(alice.id, &page(10)).unwrap();
        assert_eq!(alice_rows.total, 2);
        assert_eq!(alice_rows.data.len(), 2);
        assert_eq!(alice_rows.data[0].event_type, "a.2");
        assert_eq!(alice_rows.data[1].event_type, "a.1");
    }

    #[test]
    fn paginate_by_invoice_orders_newest_first() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let repo = SqliteAuditRepository::new(db.clone());
        let client = seed_client(&clients, "Acme");
        let invoice_id = InvoiceId::new();
        // Seed a parent invoice so the FK is satisfiable.
        db.lock()
            .execute(
                "INSERT INTO invoices (id, number, client_id, template_id, date, due_date, subtotal, tax_total, total, currency, status, pdf_path, notes, created_at, updated_at)
                 VALUES (?1, NULL, ?2, NULL, '2026-01-01', NULL, 0, 0, 0, 'EUR', 'Draft', NULL, NULL, ?3, ?3)",
                params![invoice_id.to_string(), client.id.to_string(), Utc::now().to_rfc3339()],
            )
            .unwrap();
        let now = Utc::now();
        repo.insert(&audit("i.late", None, Some(invoice_id), now))
            .unwrap();
        repo.insert(&audit("i.early", None, Some(invoice_id), now - Duration::hours(3)))
            .unwrap();

        let rows = repo.paginate_by_invoice(invoice_id, &page(50)).unwrap();
        assert_eq!(rows.total, 2);
        assert_eq!(rows.data.len(), 2);
        assert_eq!(rows.data[0].event_type, "i.late");
        assert_eq!(rows.data[1].event_type, "i.early");
    }

    #[test]
    fn client_hard_delete_nulls_scope_but_keeps_the_row() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let repo = SqliteAuditRepository::new(db.clone());
        let client = seed_client(&clients, "Goner");
        repo.insert(&audit("c.created", Some(client.id), None, Utc::now()))
            .unwrap();

        // Hard-delete via raw SQL (the client repo only soft-archives).
        db.lock()
            .execute(
                "DELETE FROM clients WHERE id = ?1",
                params![client.id.to_string()],
            )
            .unwrap();

        // ON DELETE SET NULL: the audit row survives as a tombstone, with
        // its client scope cleared but its metadata still renderable.
        let listed = repo.paginate_recent(&page(10)).unwrap();
        assert_eq!(listed.total, 1);
        assert_eq!(listed.data[0].event_type, "c.created");
        assert_eq!(listed.data[0].client_id, None);
        assert!(repo
            .paginate_by_client(client.id, &page(10))
            .unwrap()
            .data
            .is_empty());
    }
}
