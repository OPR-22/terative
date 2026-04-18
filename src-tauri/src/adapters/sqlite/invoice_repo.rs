use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, OptionalExtension, Row};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{InvoiceRepository, ListInvoicesQuery};
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::{AppliedTax, Invoice, InvoiceId, InvoiceNumber, InvoiceStatus};
use crate::domain::line_item::{LineItem, LineItemId};
use crate::domain::money::{Currency, Money};
use crate::domain::tax::TaxId;
use crate::domain::template::TemplateId;

pub struct SqliteInvoiceRepository {
    db: Db,
}

impl SqliteInvoiceRepository {
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
    Uuid::parse_str(s)
        .map(wrap)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}

fn row_to_invoice_head(row: &Row<'_>) -> rusqlite::Result<InvoiceHead> {
    let id_str: String = row.get("id")?;
    let id: InvoiceId = parse_uuid(&id_str, InvoiceId)?;
    let client_id_str: String = row.get("client_id")?;
    let client_id: ClientId = parse_uuid(&client_id_str, ClientId)?;
    let template_id: Option<TemplateId> = match row.get::<_, Option<String>>("template_id")? {
        Some(s) => Some(parse_uuid(&s, TemplateId)?),
        None => None,
    };
    let number: Option<i64> = row.get("number")?;
    let number = number.map(|n| InvoiceNumber(n as u64));
    let date_str: String = row.get("date")?;
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let due_date: Option<NaiveDate> = match row.get::<_, Option<String>>("due_date")? {
        Some(s) => Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?),
        None => None,
    };
    let subtotal: i64 = row.get("subtotal")?;
    let tax_total: i64 = row.get("tax_total")?;
    let total: i64 = row.get("total")?;
    let currency_code: String = row.get("currency")?;
    let currency = Currency::new(&currency_code).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
        )
    })?;
    let status_str: String = row.get("status")?;
    let status = InvoiceStatus::parse(&status_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown status: {status_str}"),
            )),
        )
    })?;
    let pdf_path: Option<String> = row.get("pdf_path")?;
    let notes: Option<String> = row.get("notes")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
        .with_timezone(&Utc);
    Ok(InvoiceHead {
        id,
        number,
        client_id,
        template_id,
        date,
        due_date,
        subtotal: Money::new(subtotal, currency),
        tax_total: Money::new(tax_total, currency),
        total: Money::new(total, currency),
        currency,
        status,
        pdf_path,
        notes,
        created_at,
        updated_at,
    })
}

struct InvoiceHead {
    id: InvoiceId,
    number: Option<InvoiceNumber>,
    client_id: ClientId,
    template_id: Option<TemplateId>,
    date: NaiveDate,
    due_date: Option<NaiveDate>,
    subtotal: Money,
    tax_total: Money,
    total: Money,
    currency: Currency,
    status: InvoiceStatus,
    pdf_path: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

const SELECT_HEAD: &str = "id, number, client_id, template_id, date, due_date, subtotal, tax_total, \
    total, currency, status, pdf_path, notes, created_at, updated_at";

impl InvoiceRepository for SqliteInvoiceRepository {
    fn insert(&self, invoice: &Invoice) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        tx.execute(
            "INSERT INTO invoices (id, number, client_id, template_id, date, due_date, subtotal,
                tax_total, total, currency, status, pdf_path, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                invoice.id.to_string(),
                invoice.number.map(|n| n.0 as i64),
                invoice.client_id.to_string(),
                invoice.template_id.map(|t| t.to_string()),
                invoice.date.format("%Y-%m-%d").to_string(),
                invoice.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                invoice.subtotal.minor_units(),
                invoice.tax_total.minor_units(),
                invoice.total.minor_units(),
                invoice.currency.code(),
                invoice.status.as_str(),
                invoice.pdf_path,
                invoice.notes,
                invoice.created_at.to_rfc3339(),
                invoice.updated_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        insert_items_and_taxes(&tx, invoice)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn update(&self, invoice: &Invoice) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        let affected = tx
            .execute(
                "UPDATE invoices SET
                    number = ?2, client_id = ?3, template_id = ?4, date = ?5, due_date = ?6,
                    subtotal = ?7, tax_total = ?8, total = ?9, currency = ?10, status = ?11,
                    pdf_path = ?12, notes = ?13, updated_at = ?14
                 WHERE id = ?1",
                params![
                    invoice.id.to_string(),
                    invoice.number.map(|n| n.0 as i64),
                    invoice.client_id.to_string(),
                    invoice.template_id.map(|t| t.to_string()),
                    invoice.date.format("%Y-%m-%d").to_string(),
                    invoice.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    invoice.subtotal.minor_units(),
                    invoice.tax_total.minor_units(),
                    invoice.total.minor_units(),
                    invoice.currency.code(),
                    invoice.status.as_str(),
                    invoice.pdf_path,
                    invoice.notes,
                    invoice.updated_at.to_rfc3339(),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        tx.execute(
            "DELETE FROM invoice_line_items WHERE invoice_id = ?1",
            params![invoice.id.to_string()],
        )
        .map_err(map_err)?;
        tx.execute(
            "DELETE FROM invoice_taxes WHERE invoice_id = ?1",
            params![invoice.id.to_string()],
        )
        .map_err(map_err)?;
        insert_items_and_taxes(&tx, invoice)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn get(&self, id: InvoiceId) -> Result<Option<Invoice>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_HEAD} FROM invoices WHERE id = ?1");
        let head = conn
            .query_row(&sql, params![id.to_string()], row_to_invoice_head)
            .optional()
            .map_err(map_err)?;
        let Some(head) = head else {
            return Ok(None);
        };
        let line_items = load_line_items(&conn, head.id, head.currency)?;
        let taxes_applied = load_taxes(&conn, head.id, head.currency)?;
        Ok(Some(assemble(head, line_items, taxes_applied)))
    }

    fn list(&self, query: ListInvoicesQuery) -> Result<Vec<Invoice>, RepoError> {
        let conn = self.db.lock();
        let mut sql = format!("SELECT {SELECT_HEAD} FROM invoices");
        let mut clauses: Vec<String> = Vec::new();
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(status) = query.status {
            clauses.push(format!("status = ?{}", binds.len() + 1));
            binds.push(Box::new(status.as_str().to_string()));
        }
        if let Some(cid) = query.client_id {
            clauses.push(format!("client_id = ?{}", binds.len() + 1));
            binds.push(Box::new(cid.to_string()));
        }
        if let Some(search) = query.search.as_ref().and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(format!("%{}%", t.to_lowercase()))
            }
        }) {
            clauses.push(format!(
                "(LOWER(CAST(COALESCE(number, '') AS TEXT)) LIKE ?{idx} OR LOWER(COALESCE(notes, '')) LIKE ?{idx})",
                idx = binds.len() + 1
            ));
            binds.push(Box::new(search));
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY date DESC, created_at DESC");
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> =
            binds.iter().map(|b| b.as_ref()).collect();
        let heads_iter = stmt
            .query_map(params_ref.as_slice(), row_to_invoice_head)
            .map_err(map_err)?;
        let mut heads = Vec::new();
        for h in heads_iter {
            heads.push(h.map_err(map_err)?);
        }
        drop(stmt);

        let mut invoices = Vec::with_capacity(heads.len());
        for head in heads {
            let items = load_line_items(&conn, head.id, head.currency)?;
            let taxes = load_taxes(&conn, head.id, head.currency)?;
            invoices.push(assemble(head, items, taxes));
        }
        Ok(invoices)
    }

    fn delete(&self, id: InvoiceId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM invoices WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(map_err)?;
        Ok(())
    }
}

fn insert_items_and_taxes(tx: &rusqlite::Transaction<'_>, invoice: &Invoice) -> Result<(), RepoError> {
    for (idx, li) in invoice.line_items.iter().enumerate() {
        let qty_f64 = li.quantity.to_f64().unwrap_or(0.0);
        tx.execute(
            "INSERT INTO invoice_line_items (id, invoice_id, description, quantity, unit_price, total, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                li.id.to_string(),
                invoice.id.to_string(),
                li.description,
                qty_f64,
                li.unit_price.minor_units(),
                li.total.minor_units(),
                idx as i64,
            ],
        )
        .map_err(map_err)?;
    }
    for t in &invoice.taxes_applied {
        let tax_row_id = Uuid::new_v4().to_string();
        let pct_f64 = t.percentage.to_f64().unwrap_or(0.0);
        tx.execute(
            "INSERT INTO invoice_taxes (id, invoice_id, tax_definition_id, tax_name, percentage, tax_id_number, computed_amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                tax_row_id,
                invoice.id.to_string(),
                t.tax_definition_id.map(|id| id.to_string()),
                t.tax_name,
                pct_f64,
                t.tax_id_number,
                t.computed_amount.minor_units(),
            ],
        )
        .map_err(map_err)?;
    }
    Ok(())
}

fn load_line_items(
    conn: &rusqlite::Connection,
    id: InvoiceId,
    currency: Currency,
) -> Result<Vec<LineItem>, RepoError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, description, quantity, unit_price, total FROM invoice_line_items
             WHERE invoice_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![id.to_string()], |row| {
            let id_str: String = row.get("id")?;
            let id: LineItemId = parse_uuid(&id_str, LineItemId)?;
            let qty_f64: f64 = row.get("quantity")?;
            let quantity = Decimal::from_f64(qty_f64).unwrap_or(Decimal::ZERO);
            let unit_price: i64 = row.get("unit_price")?;
            let total: i64 = row.get("total")?;
            Ok(LineItem {
                id,
                description: row.get("description")?,
                quantity,
                unit_price: Money::new(unit_price, currency),
                total: Money::new(total, currency),
            })
        })
        .map_err(map_err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(map_err)?);
    }
    Ok(out)
}

fn load_taxes(
    conn: &rusqlite::Connection,
    id: InvoiceId,
    currency: Currency,
) -> Result<Vec<AppliedTax>, RepoError> {
    let mut stmt = conn
        .prepare(
            "SELECT tax_definition_id, tax_name, percentage, tax_id_number, computed_amount
             FROM invoice_taxes WHERE invoice_id = ?1 ORDER BY rowid ASC",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![id.to_string()], |row| {
            let td: Option<String> = row.get("tax_definition_id")?;
            let tax_definition_id = match td {
                Some(s) => Some(parse_uuid(&s, TaxId)?),
                None => None,
            };
            let pct_f64: f64 = row.get("percentage")?;
            let percentage = Decimal::from_f64(pct_f64).unwrap_or(Decimal::ZERO);
            let amt: i64 = row.get("computed_amount")?;
            Ok(AppliedTax {
                tax_definition_id,
                tax_name: row.get("tax_name")?,
                percentage,
                tax_id_number: row.get("tax_id_number")?,
                computed_amount: Money::new(amt, currency),
            })
        })
        .map_err(map_err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(map_err)?);
    }
    Ok(out)
}

fn assemble(head: InvoiceHead, line_items: Vec<LineItem>, taxes_applied: Vec<AppliedTax>) -> Invoice {
    Invoice {
        id: head.id,
        number: head.number,
        client_id: head.client_id,
        template_id: head.template_id,
        date: head.date,
        due_date: head.due_date,
        line_items,
        taxes_applied,
        subtotal: head.subtotal,
        tax_total: head.tax_total,
        total: head.total,
        currency: head.currency,
        status: head.status,
        pdf_path: head.pdf_path,
        notes: head.notes,
        created_at: head.created_at,
        updated_at: head.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::client::{Client, NewClient};
    use crate::domain::invoice::NewInvoice;
    use crate::domain::line_item::NewLineItem;
    use crate::domain::tax::{NewTaxDefinition, TaxDefinition};
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn seed_client(db: &Db) -> ClientId {
        use crate::adapters::sqlite::SqliteClientRepository;
        use crate::application::ports::ClientRepository as _;
        let client = Client::create(
            NewClient {
                name: "Acme".into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        let repo = SqliteClientRepository::new(db.clone());
        repo.insert(&client).unwrap();
        client.id
    }

    fn seed_tax(db: &Db) -> TaxDefinition {
        use crate::adapters::sqlite::SqliteTaxRepository;
        use crate::application::ports::TaxRepository as _;
        let tax = TaxDefinition::create(NewTaxDefinition {
            name: "TVA".into(),
            percentage: dec!(21),
            tax_id_number: None,
        })
        .unwrap();
        SqliteTaxRepository::new(db.clone()).insert(&tax).unwrap();
        tax
    }

    #[test]
    fn insert_get_round_trip_with_items_and_taxes() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteInvoiceRepository::new(db.clone());
        let tax = seed_tax(&db);
        let invoice = Invoice::create_draft(
            NewInvoice {
                client_id,
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    description: "Widget".into(),
                    quantity: dec!(2),
                    unit_price: Money::new(1000, eur()),
                }],
                tax_ids: vec![tax.id],
                notes: Some("thanks".into()),
                currency: eur(),
            },
            &[tax],
            Utc::now(),
        )
        .unwrap();
        repo.insert(&invoice).unwrap();
        let loaded = repo.get(invoice.id).unwrap().unwrap();
        assert_eq!(loaded.line_items.len(), 1);
        assert_eq!(loaded.line_items[0].description, "Widget");
        assert_eq!(loaded.subtotal.minor_units(), 2000);
        assert_eq!(loaded.taxes_applied.len(), 1);
        assert_eq!(loaded.taxes_applied[0].computed_amount.minor_units(), 420);
    }

    #[test]
    fn update_replaces_line_items_and_taxes() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteInvoiceRepository::new(db.clone());
        let tax = seed_tax(&db);
        let mut invoice = Invoice::create_draft(
            NewInvoice {
                client_id,
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    description: "Old".into(),
                    quantity: dec!(1),
                    unit_price: Money::new(500, eur()),
                }],
                tax_ids: vec![tax.id],
                notes: None,
                currency: eur(),
            },
            &[tax.clone()],
            Utc::now(),
        )
        .unwrap();
        repo.insert(&invoice).unwrap();
        invoice
            .update_draft(
                vec![NewLineItem {
                    description: "New".into(),
                    quantity: dec!(3),
                    unit_price: Money::new(1000, eur()),
                }],
                &[tax],
                None,
                NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                None,
                None,
                Utc::now(),
            )
            .unwrap();
        repo.update(&invoice).unwrap();
        let loaded = repo.get(invoice.id).unwrap().unwrap();
        assert_eq!(loaded.line_items.len(), 1);
        assert_eq!(loaded.line_items[0].description, "New");
        assert_eq!(loaded.subtotal.minor_units(), 3000);
    }

    #[test]
    fn list_filters_by_status() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let repo = SqliteInvoiceRepository::new(db.clone());
        let mut draft = Invoice::create_draft(
            NewInvoice {
                client_id,
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    description: "A".into(),
                    quantity: dec!(1),
                    unit_price: Money::new(100, eur()),
                }],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            Utc::now(),
        )
        .unwrap();
        repo.insert(&draft).unwrap();
        draft.finalize(InvoiceNumber(1), Utc::now()).unwrap();
        repo.update(&draft).unwrap();
        let list = repo
            .list(ListInvoicesQuery {
                status: Some(InvoiceStatus::Finalized),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(list.len(), 1);
    }
}
