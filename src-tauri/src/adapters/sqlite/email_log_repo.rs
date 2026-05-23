use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::EmailLogRepository;
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::email_log::{EmailLog, EmailLogId};
use crate::domain::email_template::EmailTemplateType;
use crate::domain::invoice::InvoiceId;

pub struct SqliteEmailLogRepository {
    db: Db,
}

impl SqliteEmailLogRepository {
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

fn row_to_log(row: &Row<'_>) -> rusqlite::Result<EmailLog> {
    let id_str: String = row.get("id")?;
    let id: EmailLogId = parse_uuid(&id_str, EmailLogId)?;
    let client_id_str: String = row.get("client_id")?;
    let client_id: ClientId = parse_uuid(&client_id_str, ClientId)?;
    let invoice_id_str: Option<String> = row.get("invoice_id")?;
    let invoice_id = invoice_id_str
        .as_deref()
        .map(|s| parse_uuid(s, InvoiceId))
        .transpose()?;
    let template_type_str: Option<String> = row.get("template_type")?;
    let template_type = template_type_str.and_then(|s| EmailTemplateType::parse(&s));
    let template_name: Option<String> = row.get("template_name")?;
    let to_address: String = row.get("to_address")?;
    let subject: String = row.get("subject")?;
    let sent_at_str: String = row.get("sent_at")?;
    let sent_at = DateTime::parse_from_rfc3339(&sent_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    Ok(EmailLog {
        id,
        client_id,
        invoice_id,
        template_type,
        template_name,
        to_address,
        subject,
        sent_at,
    })
}

const SELECT_COLS: &str =
    "id, client_id, invoice_id, template_type, template_name, to_address, subject, sent_at";

impl EmailLogRepository for SqliteEmailLogRepository {
    fn insert(&self, log: &EmailLog) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO email_logs (id, client_id, invoice_id, template_type, template_name, to_address, subject, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log.id.to_string(),
                log.client_id.to_string(),
                log.invoice_id.map(|i| i.to_string()),
                log.template_type.map(|t| t.as_str().to_string()),
                log.template_name.clone(),
                log.to_address,
                log.subject,
                log.sent_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn list_by_client(&self, client_id: ClientId) -> Result<Vec<EmailLog>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM email_logs WHERE client_id = ?1 ORDER BY sent_at DESC, id DESC",
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params![client_id.to_string()], row_to_log)
            .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn list_by_invoices(
        &self,
        invoice_ids: &[InvoiceId],
    ) -> Result<HashMap<InvoiceId, Vec<EmailLog>>, RepoError> {
        if invoice_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let conn = self.db.lock();
        // Build a parameterized IN clause; rusqlite needs explicit placeholders.
        let placeholders = (1..=invoice_ids.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT {SELECT_COLS} FROM email_logs \
             WHERE invoice_id IN ({placeholders}) \
             ORDER BY sent_at ASC, id ASC",
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let params_vec: Vec<String> = invoice_ids.iter().map(|i| i.to_string()).collect();
        let rows = stmt
            .query_map(rusqlite::params_from_iter(params_vec.iter()), row_to_log)
            .map_err(map_err)?;
        let mut out: HashMap<InvoiceId, Vec<EmailLog>> = HashMap::new();
        for r in rows {
            let log = r.map_err(map_err)?;
            if let Some(id) = log.invoice_id {
                out.entry(id).or_default().push(log);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::SqliteClientRepository;
    use crate::application::ports::ClientRepository;
    use crate::domain::client::{Client, NewClient};
    use crate::domain::email_log::{EmailLog, NewEmailLog};

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

    fn make_log(client_id: ClientId, subject: &str, sent_at: DateTime<Utc>) -> EmailLog {
        EmailLog::record(NewEmailLog {
            client_id,
            invoice_id: None,
            template_type: Some(EmailTemplateType::InitialContact),
            template_name: Some("Default".into()),
            to_address: "to@example.com".into(),
            subject: subject.into(),
            sent_at,
        })
        .unwrap()
    }

    #[test]
    fn insert_and_list_round_trip() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let logs = SqliteEmailLogRepository::new(db);
        let c = seed_client(&clients, "Acme");

        let now = Utc::now();
        let l1 = make_log(c.id, "First", now - chrono::Duration::hours(2));
        let l2 = make_log(c.id, "Second", now);
        logs.insert(&l1).unwrap();
        logs.insert(&l2).unwrap();

        let listed = logs.list_by_client(c.id).unwrap();
        assert_eq!(listed.len(), 2);
        // Newest first.
        assert_eq!(listed[0].subject, "Second");
        assert_eq!(listed[1].subject, "First");
        assert_eq!(listed[0].to_address, "to@example.com");
    }

    #[test]
    fn list_filters_by_client() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let logs = SqliteEmailLogRepository::new(db);
        let a = seed_client(&clients, "Alice");
        let b = seed_client(&clients, "Bob");

        logs.insert(&make_log(a.id, "to-a", Utc::now())).unwrap();
        logs.insert(&make_log(b.id, "to-b", Utc::now())).unwrap();

        let only_a = logs.list_by_client(a.id).unwrap();
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].subject, "to-a");
    }

    #[test]
    fn deleting_client_cascades_log() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let logs = SqliteEmailLogRepository::new(db.clone());
        let c = seed_client(&clients, "Goner");

        logs.insert(&make_log(c.id, "bye", Utc::now())).unwrap();
        // Hard-delete via raw SQL since the client repo only soft-archives.
        db.lock()
            .execute("DELETE FROM clients WHERE id = ?1", params![c.id.to_string()])
            .unwrap();

        let listed = logs.list_by_client(c.id).unwrap();
        assert!(listed.is_empty());
    }

    #[test]
    fn list_by_invoices_groups_by_invoice() {
        use crate::domain::invoice::InvoiceId;
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let logs = SqliteEmailLogRepository::new(db.clone());
        let c = seed_client(&clients, "Acme");
        let inv1 = InvoiceId::new();
        let inv2 = InvoiceId::new();
        // Seed parent invoices so the FK is satisfiable. We side-step the
        // invoice repo and insert a minimal placeholder via raw SQL.
        let conn = db.lock();
        for id in [inv1, inv2] {
            conn.execute(
                "INSERT INTO invoices (id, number, client_id, template_id, date, due_date, subtotal, tax_total, total, currency, status, pdf_path, notes, created_at, updated_at)
                 VALUES (?1, NULL, ?2, NULL, '2026-01-01', NULL, 0, 0, 0, 'EUR', 'Draft', NULL, NULL, ?3, ?3)",
                params![id.to_string(), c.id.to_string(), Utc::now().to_rfc3339()],
            )
            .unwrap();
        }
        drop(conn);

        let mut a = make_log(c.id, "first", Utc::now() - chrono::Duration::hours(2));
        a.invoice_id = Some(inv1);
        let mut b = make_log(c.id, "second", Utc::now() - chrono::Duration::hours(1));
        b.invoice_id = Some(inv1);
        let mut x = make_log(c.id, "other", Utc::now());
        x.invoice_id = Some(inv2);
        logs.insert(&a).unwrap();
        logs.insert(&b).unwrap();
        logs.insert(&x).unwrap();

        let grouped = logs.list_by_invoices(&[inv1, inv2]).unwrap();
        assert_eq!(grouped.get(&inv1).unwrap().len(), 2);
        // Ascending within group.
        assert_eq!(grouped.get(&inv1).unwrap()[0].subject, "first");
        assert_eq!(grouped.get(&inv2).unwrap().len(), 1);
    }

    #[test]
    fn list_by_invoices_handles_empty_input() {
        let db = open_memory();
        let logs = SqliteEmailLogRepository::new(db);
        let grouped = logs.list_by_invoices(&[]).unwrap();
        assert!(grouped.is_empty());
    }

    #[test]
    fn invoice_id_can_be_null() {
        let db = open_memory();
        let clients = SqliteClientRepository::new(db.clone());
        let logs = SqliteEmailLogRepository::new(db);
        let c = seed_client(&clients, "Acme");
        let log = make_log(c.id, "Ad-hoc", Utc::now());
        assert!(log.invoice_id.is_none());
        logs.insert(&log).unwrap();
        let listed = logs.list_by_client(c.id).unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].invoice_id.is_none());
    }
}
