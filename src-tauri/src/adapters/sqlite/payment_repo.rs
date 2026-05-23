use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{ListPaymentsQuery, PaymentRepository};
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::InvoiceId;
use crate::domain::money::{Currency, Money};
use crate::domain::payment::{Payment, PaymentAllocation, PaymentId, PaymentMethod};

pub struct SqlitePaymentRepository {
    db: Db,
}

impl SqlitePaymentRepository {
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

fn currency_from_str(code: &str) -> rusqlite::Result<Currency> {
    Currency::new(code).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
        )
    })
}

fn row_to_payment_head(row: &Row<'_>) -> rusqlite::Result<PaymentHead> {
    let id_str: String = row.get("id")?;
    let id: PaymentId = parse_uuid(&id_str, PaymentId)?;
    let client_id_str: String = row.get("client_id")?;
    let client_id: ClientId = parse_uuid(&client_id_str, ClientId)?;
    let date_str: String = row.get("date")?;
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let amount: i64 = row.get("amount")?;
    let currency_code: String = row.get("currency")?;
    let currency = currency_from_str(&currency_code)?;
    let method_str: String = row.get("method")?;
    let reference: Option<String> = row.get("reference")?;
    let notes: Option<String> = row.get("notes")?;
    let created_at_str: String = row.get("created_at")?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);
    Ok(PaymentHead {
        id,
        client_id,
        date,
        amount: Money::new(amount, currency),
        currency,
        method: PaymentMethod::parse_db_string(&method_str),
        reference,
        notes,
        created_at,
    })
}

struct PaymentHead {
    id: PaymentId,
    client_id: ClientId,
    date: NaiveDate,
    amount: Money,
    currency: Currency,
    method: PaymentMethod,
    reference: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

const SELECT_HEAD: &str =
    "id, client_id, date, amount, currency, method, reference, notes, created_at";

impl PaymentRepository for SqlitePaymentRepository {
    fn insert(&self, p: &Payment) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        tx.execute(
            "INSERT INTO payments (id, client_id, date, amount, currency, method, reference, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                p.id.to_string(),
                p.client_id.to_string(),
                p.date.format("%Y-%m-%d").to_string(),
                p.amount.minor_units(),
                p.amount.currency().code(),
                p.method.to_db_string(),
                p.reference,
                p.notes,
                p.created_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
        insert_allocations(&tx, p)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn update(&self, p: &Payment) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        let affected = tx
            .execute(
                "UPDATE payments SET
                    client_id = ?2, date = ?3, amount = ?4, currency = ?5,
                    method = ?6, reference = ?7, notes = ?8
                 WHERE id = ?1",
                params![
                    p.id.to_string(),
                    p.client_id.to_string(),
                    p.date.format("%Y-%m-%d").to_string(),
                    p.amount.minor_units(),
                    p.amount.currency().code(),
                    p.method.to_db_string(),
                    p.reference,
                    p.notes,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        tx.execute(
            "DELETE FROM payment_allocations WHERE payment_id = ?1",
            params![p.id.to_string()],
        )
        .map_err(map_err)?;
        insert_allocations(&tx, p)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn get(&self, id: PaymentId) -> Result<Option<Payment>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_HEAD} FROM payments WHERE id = ?1");
        let head = conn
            .query_row(&sql, params![id.to_string()], row_to_payment_head)
            .optional()
            .map_err(map_err)?;
        let Some(head) = head else {
            return Ok(None);
        };
        let allocations = load_allocations(&conn, head.id, head.currency)?;
        Ok(Some(assemble(head, allocations)))
    }

    fn list(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, RepoError> {
        let conn = self.db.lock();
        // Filtering by invoice requires joining payment_allocations.
        // SELECT DISTINCT because a payment can have multiple allocations
        // (different invoices), but we only want it once even if more than
        // one of those allocations matches.
        let (table_clause, distinct) = if query.invoice_id.is_some() {
            (
                " p INNER JOIN payment_allocations pa ON pa.payment_id = p.id",
                "DISTINCT ",
            )
        } else {
            ("", "")
        };
        // Qualify the selected columns when joining so SQLite knows which
        // table they come from. Otherwise use the bare names from before.
        let head_clause = if query.invoice_id.is_some() {
            SELECT_HEAD
                .split(", ")
                .map(|c| format!("p.{}", c.trim()))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            SELECT_HEAD.to_string()
        };
        let mut sql = format!(
            "SELECT {distinct}{head_clause} FROM payments{table_clause}"
        );
        let mut clauses: Vec<String> = Vec::new();
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let qualify = |col: &str| {
            if query.invoice_id.is_some() {
                format!("p.{col}")
            } else {
                col.to_string()
            }
        };
        if let Some(cid) = query.client_id {
            clauses.push(format!("{} = ?{}", qualify("client_id"), binds.len() + 1));
            binds.push(Box::new(cid.to_string()));
        }
        if let Some(iid) = query.invoice_id {
            clauses.push(format!("pa.invoice_id = ?{}", binds.len() + 1));
            binds.push(Box::new(iid.to_string()));
        }
        if let Some(pattern) = query.search.as_ref().and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(format!("%{}%", t.to_lowercase()))
            }
        }) {
            clauses.push(format!(
                "(LOWER(COALESCE({ref_col}, '')) LIKE ?{idx} OR LOWER(COALESCE({notes_col}, '')) LIKE ?{idx})",
                ref_col = qualify("reference"),
                notes_col = qualify("notes"),
                idx = binds.len() + 1
            ));
            binds.push(Box::new(pattern));
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(&format!(
            " ORDER BY {date} DESC, {created} DESC",
            date = qualify("date"),
            created = qualify("created_at"),
        ));
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> =
            binds.iter().map(|b| b.as_ref()).collect();
        let head_iter = stmt
            .query_map(params_ref.as_slice(), row_to_payment_head)
            .map_err(map_err)?;
        let mut heads = Vec::new();
        for h in head_iter {
            heads.push(h.map_err(map_err)?);
        }
        drop(stmt);

        let mut payments = Vec::with_capacity(heads.len());
        for head in heads {
            let allocations = load_allocations(&conn, head.id, head.currency)?;
            payments.push(assemble(head, allocations));
        }
        Ok(payments)
    }

    fn delete(&self, id: PaymentId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM payments WHERE id = ?1", params![id.to_string()])
            .map_err(map_err)?;
        Ok(())
    }

    fn allocated_for_invoice(
        &self,
        id: InvoiceId,
        invoice_currency: Currency,
    ) -> Result<Money, RepoError> {
        let conn = self.db.lock();
        // Strict silos: only sum allocations whose payment currency matches
        // the invoice's currency. The use-case layer should already prevent
        // mismatches from reaching the DB, but filtering here makes the
        // query correct even if a future bug let one through.
        let sum: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(pa.amount), 0)
                 FROM payment_allocations pa
                 JOIN payments p ON p.id = pa.payment_id
                 WHERE pa.invoice_id = ?1 AND p.currency = ?2",
                params![id.to_string(), invoice_currency.code()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        Ok(Money::new(sum, invoice_currency))
    }

    fn allocated_for_invoices(
        &self,
        ids: &[InvoiceId],
    ) -> Result<std::collections::HashMap<InvoiceId, Money>, RepoError> {
        if ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT pa.invoice_id, SUM(pa.amount), MAX(p.currency)
             FROM payment_allocations pa
             JOIN payments p ON p.id = pa.payment_id
             WHERE pa.invoice_id IN ({})
             GROUP BY pa.invoice_id",
            placeholders.join(", ")
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let params_vec: Vec<&dyn rusqlite::ToSql> = id_strings
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        let rows = stmt
            .query_map(params_vec.as_slice(), |r| {
                let id_str: String = r.get(0)?;
                let amount: i64 = r.get(1)?;
                let currency_code: Option<String> = r.get(2)?;
                Ok((id_str, amount, currency_code))
            })
            .map_err(map_err)?;
        let mut out = std::collections::HashMap::new();
        for row in rows {
            let (id_str, amount, currency_code) = row.map_err(map_err)?;
            let uuid = Uuid::parse_str(&id_str).map_err(|e| {
                RepoError::Storage(format!("invalid invoice_id in allocations: {e}"))
            })?;
            let currency = currency_code
                .as_deref()
                .and_then(|c| Currency::new(c).ok())
                .unwrap_or_else(|| Currency::new("EUR").unwrap());
            out.insert(InvoiceId(uuid), Money::new(amount, currency));
        }
        Ok(out)
    }
}

fn insert_allocations(tx: &rusqlite::Transaction<'_>, p: &Payment) -> Result<(), RepoError> {
    for a in &p.allocations {
        let row_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO payment_allocations (id, payment_id, invoice_id, amount)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                row_id,
                p.id.to_string(),
                a.invoice_id.to_string(),
                a.amount.minor_units(),
            ],
        )
        .map_err(map_err)?;
    }
    Ok(())
}

fn load_allocations(
    conn: &rusqlite::Connection,
    id: PaymentId,
    currency: Currency,
) -> Result<Vec<PaymentAllocation>, RepoError> {
    let mut stmt = conn
        .prepare(
            "SELECT invoice_id, amount FROM payment_allocations
             WHERE payment_id = ?1 ORDER BY rowid ASC",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![id.to_string()], |row| {
            let inv_str: String = row.get("invoice_id")?;
            let invoice_id: InvoiceId = parse_uuid(&inv_str, InvoiceId)?;
            let amount: i64 = row.get("amount")?;
            Ok(PaymentAllocation {
                invoice_id,
                amount: Money::new(amount, currency),
            })
        })
        .map_err(map_err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(map_err)?);
    }
    Ok(out)
}

fn assemble(head: PaymentHead, allocations: Vec<PaymentAllocation>) -> Payment {
    Payment {
        id: head.id,
        client_id: head.client_id,
        date: head.date,
        amount: head.amount,
        method: head.method,
        reference: head.reference,
        allocations,
        notes: head.notes,
        created_at: head.created_at,
        // A row loaded from SQLite has no pending events — they exist only
        // for the lifetime of an in-memory mutation.
        pending_events: crate::domain::events::EventBuffer::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::{SqliteClientRepository, SqliteInvoiceRepository};
    use crate::application::ports::{ClientRepository as _, InvoiceRepository as _};
    use crate::domain::client::{Client, NewClient};
    use crate::domain::invoice::{Invoice, InvoiceNumber, NewInvoice};
    use crate::domain::line_item::NewLineItem;
    use crate::domain::payment::{NewPayment, NewPaymentAllocation};
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

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

    static NEXT_NUMBER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

    fn seed_finalized_invoice(db: &Db, client_id: ClientId, total_cents: i64) -> InvoiceId {
        let mut invoice = Invoice::create_draft(
            NewInvoice {
                client_id,
                template_id: None,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                due_date: None,
                line_items: vec![NewLineItem {
                    id: None,
                    catalog_item_id: None,
                    description: "Widget".into(),
                    quantity: dec!(1),
                    unit_price: Money::new(total_cents, eur()),
                }],
                tax_ids: vec![],
                notes: None,
                currency: eur(),
            },
            &[],
            Utc::now(),
        )
        .unwrap();
        let n = NEXT_NUMBER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        invoice.finalize(InvoiceNumber(n), Utc::now()).unwrap();
        SqliteInvoiceRepository::new(db.clone()).insert(&invoice).unwrap();
        invoice.id
    }

    fn make_payment(client_id: ClientId, amount: i64, allocations: Vec<NewPaymentAllocation>) -> Payment {
        Payment::create(
            NewPayment {
                client_id,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                amount: Money::new(amount, eur()),
                method: PaymentMethod::BankTransfer,
                reference: Some("WIRE-1".into()),
                allocations,
                notes: None,
            },
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip_with_allocations() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let invoice_id = seed_finalized_invoice(&db, client_id, 1000);
        let repo = SqlitePaymentRepository::new(db.clone());

        let payment = make_payment(
            client_id,
            1000,
            vec![NewPaymentAllocation {
                invoice_id,
                amount: Money::new(1000, eur()),
            }],
        );
        repo.insert(&payment).unwrap();
        let loaded = repo.get(payment.id).unwrap().unwrap();
        assert_eq!(loaded.amount.minor_units(), 1000);
        assert_eq!(loaded.allocations.len(), 1);
        assert_eq!(loaded.allocations[0].invoice_id, invoice_id);
        assert_eq!(loaded.reference.as_deref(), Some("WIRE-1"));
        assert_eq!(loaded.method, PaymentMethod::BankTransfer);
    }

    #[test]
    fn update_replaces_allocations() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let inv_a = seed_finalized_invoice(&db, client_id, 500);
        let inv_b = seed_finalized_invoice(&db, client_id, 500);
        let repo = SqlitePaymentRepository::new(db.clone());

        let mut payment = make_payment(
            client_id,
            1000,
            vec![NewPaymentAllocation {
                invoice_id: inv_a,
                amount: Money::new(500, eur()),
            }],
        );
        repo.insert(&payment).unwrap();
        payment
            .replace_fields(
                payment.date,
                payment.amount,
                PaymentMethod::Cash,
                None,
                vec![NewPaymentAllocation {
                    invoice_id: inv_b,
                    amount: Money::new(500, eur()),
                }],
                None,
                Utc::now(),
            )
            .unwrap();
        repo.update(&payment).unwrap();
        let loaded = repo.get(payment.id).unwrap().unwrap();
        assert_eq!(loaded.allocations.len(), 1);
        assert_eq!(loaded.allocations[0].invoice_id, inv_b);
        assert_eq!(loaded.method, PaymentMethod::Cash);
    }

    #[test]
    fn allocated_for_invoice_sums_across_payments() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let invoice_id = seed_finalized_invoice(&db, client_id, 2000);
        let repo = SqlitePaymentRepository::new(db.clone());

        repo.insert(&make_payment(
            client_id,
            500,
            vec![NewPaymentAllocation {
                invoice_id,
                amount: Money::new(500, eur()),
            }],
        ))
        .unwrap();
        repo.insert(&make_payment(
            client_id,
            800,
            vec![NewPaymentAllocation {
                invoice_id,
                amount: Money::new(700, eur()),
            }],
        ))
        .unwrap();

        let allocated = repo.allocated_for_invoice(invoice_id, eur()).unwrap();
        assert_eq!(allocated.minor_units(), 1200);
    }

    #[test]
    fn allocated_for_invoices_batches_and_skips_unallocated() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let inv_a = seed_finalized_invoice(&db, client_id, 1000);
        let inv_b = seed_finalized_invoice(&db, client_id, 2000);
        let inv_c = seed_finalized_invoice(&db, client_id, 3000);
        let repo = SqlitePaymentRepository::new(db.clone());

        repo.insert(&make_payment(
            client_id,
            500,
            vec![NewPaymentAllocation {
                invoice_id: inv_a,
                amount: Money::new(500, eur()),
            }],
        ))
        .unwrap();
        repo.insert(&make_payment(
            client_id,
            700,
            vec![NewPaymentAllocation {
                invoice_id: inv_b,
                amount: Money::new(700, eur()),
            }],
        ))
        .unwrap();
        // inv_c has no allocations.

        let totals = repo
            .allocated_for_invoices(&[inv_a, inv_b, inv_c])
            .unwrap();
        assert_eq!(totals.get(&inv_a).unwrap().minor_units(), 500);
        assert_eq!(totals.get(&inv_b).unwrap().minor_units(), 700);
        assert!(totals.get(&inv_c).is_none());
    }

    #[test]
    fn allocated_for_invoices_empty_input_returns_empty() {
        let db = open_memory();
        let repo = SqlitePaymentRepository::new(db);
        let totals = repo.allocated_for_invoices(&[]).unwrap();
        assert!(totals.is_empty());
    }

    #[test]
    fn list_filters_by_client_id_and_search() {
        let db = open_memory();
        let client_a = seed_client(&db);
        let client_b_id = {
            let client = Client::create(
                NewClient {
                    name: "Other".into(),
                    ..Default::default()
                },
                Utc::now(),
            )
            .unwrap();
            SqliteClientRepository::new(db.clone()).insert(&client).unwrap();
            client.id
        };
        let repo = SqlitePaymentRepository::new(db.clone());

        let p_a = make_payment(client_a, 1000, vec![]);
        repo.insert(&p_a).unwrap();
        let mut p_b = make_payment(client_b_id, 2000, vec![]);
        p_b.reference = Some("OTHER-REF".into());
        repo.update(&p_b).ok();
        // reinsert because update returns NotFound on fresh row
        let mut p_b2 = p_b.clone();
        p_b2.id = PaymentId::new();
        repo.insert(&p_b2).unwrap();

        let client_a_list = repo
            .list(ListPaymentsQuery {
                client_id: Some(client_a),
                invoice_id: None,
                search: None,
            })
            .unwrap();
        assert_eq!(client_a_list.len(), 1);

        let search = repo
            .list(ListPaymentsQuery {
                client_id: None,
                invoice_id: None,
                search: Some("OTHER".into()),
            })
            .unwrap();
        assert_eq!(search.len(), 1);
    }

    #[test]
    fn delete_removes_payment_and_cascades_allocations() {
        let db = open_memory();
        let client_id = seed_client(&db);
        let invoice_id = seed_finalized_invoice(&db, client_id, 1000);
        let repo = SqlitePaymentRepository::new(db.clone());
        let payment = make_payment(
            client_id,
            1000,
            vec![NewPaymentAllocation {
                invoice_id,
                amount: Money::new(1000, eur()),
            }],
        );
        repo.insert(&payment).unwrap();
        repo.delete(payment.id).unwrap();
        assert!(repo.get(payment.id).unwrap().is_none());
        assert_eq!(
            repo.allocated_for_invoice(invoice_id, eur()).unwrap().minor_units(),
            0
        );
    }
}
