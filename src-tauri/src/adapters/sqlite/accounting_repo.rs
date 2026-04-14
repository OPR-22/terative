use chrono::{Datelike, NaiveDate};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{
    AccountingQueries, AgingBucket, AgingRow, ClientBalance, DashboardSummary,
    DerivedPaymentStatus, InvoicePaymentRow, RevenueBucket, RevenueByClient, RevenueGrouping,
};
use crate::application::RepoError;
use crate::domain::client::ClientId;
use crate::domain::invoice::{InvoiceId, InvoiceStatus};
use crate::domain::money::{Currency, Money};

pub struct SqliteAccountingRepository {
    db: Db,
}

impl SqliteAccountingRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn map_err(e: rusqlite::Error) -> RepoError {
    RepoError::Storage(e.to_string())
}

fn parse_uuid<T>(s: &str, wrap: impl Fn(Uuid) -> T) -> rusqlite::Result<T> {
    Uuid::parse_str(s).map(wrap).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn currency_from_code(code: &str) -> Currency {
    Currency::new(code).unwrap_or_else(|_| Currency::new("EUR").unwrap())
}

fn parse_date(s: &str) -> rusqlite::Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn row_to_invoice_payment_row(row: &Row<'_>) -> rusqlite::Result<InvoicePaymentRow> {
    let id_str: String = row.get("id")?;
    let invoice_id: InvoiceId = parse_uuid(&id_str, InvoiceId)?;
    let number: Option<i64> = row.get("number")?;
    let number = number.map(|n| n as u64);
    let client_id_str: String = row.get("client_id")?;
    let client_id: ClientId = parse_uuid(&client_id_str, ClientId)?;
    let client_name: String = row.get("client_name")?;
    let date_str: String = row.get("date")?;
    let date = parse_date(&date_str)?;
    let due_str: Option<String> = row.get("due_date")?;
    let due_date = match due_str {
        Some(s) => Some(parse_date(&s)?),
        None => None,
    };
    let total_cents: i64 = row.get("total")?;
    let paid_cents: i64 = row.get("amount_paid")?;
    let currency_code: String = row.get("currency")?;
    let currency = currency_from_code(&currency_code);
    let status_str: String = row.get("status")?;
    let status = InvoiceStatus::parse(&status_str).unwrap_or(InvoiceStatus::Draft);
    let payment_status_str: String = row.get("payment_status")?;
    let payment_status =
        DerivedPaymentStatus::parse(&payment_status_str).unwrap_or(DerivedPaymentStatus::Unpaid);
    Ok(InvoicePaymentRow {
        invoice_id,
        number,
        client_id,
        client_name,
        date,
        due_date,
        total: Money::new(total_cents, currency),
        amount_paid: Money::new(paid_cents, currency),
        amount_due: Money::new(total_cents - paid_cents, currency),
        status,
        payment_status,
    })
}

const PAYMENT_ROW_SELECT: &str = "SELECT vps.id, vps.number, vps.client_id, c.name AS client_name,
    vps.date, vps.due_date, vps.total, vps.amount_paid, vps.currency, vps.status, vps.payment_status
 FROM v_invoice_payment_status vps
 JOIN clients c ON c.id = vps.client_id";

impl AccountingQueries for SqliteAccountingRepository {
    fn list_outstanding_invoices(&self) -> Result<Vec<InvoicePaymentRow>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "{PAYMENT_ROW_SELECT}
             WHERE vps.status IN ('Finalized', 'Sent')
               AND vps.total - vps.amount_paid > 0
             ORDER BY vps.due_date ASC"
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_invoice_payment_row).map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn list_overdue_invoices(
        &self,
        today: NaiveDate,
    ) -> Result<Vec<InvoicePaymentRow>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "{PAYMENT_ROW_SELECT}
             WHERE vps.status IN ('Finalized', 'Sent')
               AND vps.due_date IS NOT NULL
               AND vps.due_date < ?1
               AND vps.total - vps.amount_paid > 0
             ORDER BY vps.due_date ASC"
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(
                params![today.format("%Y-%m-%d").to_string()],
                row_to_invoice_payment_row,
            )
            .map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn revenue_by_period(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        grouping: RevenueGrouping,
    ) -> Result<Vec<RevenueBucket>, RepoError> {
        let conn = self.db.lock();
        let (format_expr, bucket_start_sql) = match grouping {
            RevenueGrouping::Day => ("%Y-%m-%d", "date"),
            RevenueGrouping::Month => ("%Y-%m", "date || '-01'"),
            RevenueGrouping::Year => ("%Y", "date || '-01-01'"),
        };
        let sql = format!(
            "SELECT strftime('{fmt}', date) AS bucket_key,
                    MIN({bs}) AS bucket_start,
                    SUM(total) AS amount,
                    COUNT(*) AS invoice_count,
                    MAX(currency) AS currency
             FROM invoices
             WHERE status IN ('Finalized', 'Sent')
               AND date >= ?1 AND date <= ?2
             GROUP BY bucket_key
             ORDER BY bucket_key ASC",
            fmt = format_expr,
            bs = bucket_start_sql,
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(
                params![
                    start.format("%Y-%m-%d").to_string(),
                    end.format("%Y-%m-%d").to_string()
                ],
                |row| {
                    let bucket_start_str: String = row.get("bucket_start")?;
                    let normalized = normalize_bucket(&bucket_start_str, grouping);
                    let bucket_start = parse_date(&normalized)?;
                    let amount_cents: i64 = row.get("amount")?;
                    let count: i64 = row.get("invoice_count")?;
                    let currency_code: Option<String> = row.get("currency")?;
                    let currency = currency_code
                        .as_deref()
                        .map(currency_from_code)
                        .unwrap_or_else(|| Currency::new("EUR").unwrap());
                    Ok(RevenueBucket {
                        bucket_start,
                        amount: Money::new(amount_cents, currency),
                        invoice_count: count as u64,
                    })
                },
            )
            .map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn revenue_by_client(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RevenueByClient>, RepoError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT i.client_id, c.name, SUM(i.total) AS total, COUNT(*) AS cnt, MAX(i.currency) AS currency
                 FROM invoices i
                 JOIN clients c ON c.id = i.client_id
                 WHERE i.status IN ('Finalized', 'Sent')
                   AND i.date >= ?1 AND i.date <= ?2
                 GROUP BY i.client_id
                 ORDER BY total DESC",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map(
                params![
                    start.format("%Y-%m-%d").to_string(),
                    end.format("%Y-%m-%d").to_string()
                ],
                |row| {
                    let cid_str: String = row.get("client_id")?;
                    let client_id: ClientId = parse_uuid(&cid_str, ClientId)?;
                    let name: String = row.get("name")?;
                    let total: i64 = row.get("total")?;
                    let cnt: i64 = row.get("cnt")?;
                    let currency_code: Option<String> = row.get("currency")?;
                    let currency = currency_code
                        .as_deref()
                        .map(currency_from_code)
                        .unwrap_or_else(|| Currency::new("EUR").unwrap());
                    Ok(RevenueByClient {
                        client_id,
                        client_name: name,
                        total_invoiced: Money::new(total, currency),
                        invoice_count: cnt as u64,
                    })
                },
            )
            .map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn client_balance(&self, client_id: ClientId) -> Result<ClientBalance, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT c.id, c.name,
                    COALESCE(inv.total_invoiced, 0) AS total_invoiced,
                    COALESCE(pay.total_paid, 0) AS total_paid,
                    COALESCE(inv.total_invoiced, 0) - COALESCE(pay.total_paid, 0) AS outstanding
             FROM clients c
             LEFT JOIN (
                 SELECT client_id, SUM(total) AS total_invoiced
                 FROM invoices
                 WHERE status IN ('Finalized', 'Sent')
                 GROUP BY client_id
             ) inv ON inv.client_id = c.id
             LEFT JOIN (
                 SELECT client_id, SUM(amount) AS total_paid
                 FROM payments
                 GROUP BY client_id
             ) pay ON pay.client_id = c.id
             WHERE c.id = ?1",
            params![client_id.to_string()],
            row_to_client_balance,
        )
        .map_err(map_err)
    }

    fn client_balances(&self) -> Result<Vec<ClientBalance>, RepoError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.name,
                        COALESCE(inv.total_invoiced, 0) AS total_invoiced,
                        COALESCE(pay.total_paid, 0) AS total_paid,
                        COALESCE(inv.total_invoiced, 0) - COALESCE(pay.total_paid, 0) AS outstanding
                 FROM clients c
                 LEFT JOIN (
                     SELECT client_id, SUM(total) AS total_invoiced
                     FROM invoices
                     WHERE status IN ('Finalized', 'Sent')
                     GROUP BY client_id
                 ) inv ON inv.client_id = c.id
                 LEFT JOIN (
                     SELECT client_id, SUM(amount) AS total_paid
                     FROM payments
                     GROUP BY client_id
                 ) pay ON pay.client_id = c.id
                 WHERE c.active = 1
                 ORDER BY outstanding DESC",
            )
            .map_err(map_err)?;
        let rows = stmt.query_map([], row_to_client_balance).map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn aging_report(&self, today: NaiveDate) -> Result<Vec<AgingRow>, RepoError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT i.id, i.number, i.client_id, c.name AS client_name, i.total, i.currency,
                        i.due_date, COALESCE(alloc.allocated, 0) AS allocated,
                        i.total - COALESCE(alloc.allocated, 0) AS amount_due
                 FROM invoices i
                 JOIN clients c ON c.id = i.client_id
                 LEFT JOIN (
                     SELECT invoice_id, SUM(amount) AS allocated
                     FROM payment_allocations
                     GROUP BY invoice_id
                 ) alloc ON alloc.invoice_id = i.id
                 WHERE i.status IN ('Finalized', 'Sent')
                   AND i.total - COALESCE(alloc.allocated, 0) > 0
                 ORDER BY i.due_date ASC",
            )
            .map_err(map_err)?;
        let today_str = today.format("%Y-%m-%d").to_string();
        let rows = stmt
            .query_map([], |row| {
                let id_str: String = row.get("id")?;
                let invoice_id: InvoiceId = parse_uuid(&id_str, InvoiceId)?;
                let number: Option<i64> = row.get("number")?;
                let cid_str: String = row.get("client_id")?;
                let client_id: ClientId = parse_uuid(&cid_str, ClientId)?;
                let client_name: String = row.get("client_name")?;
                let total_cents: i64 = row.get("total")?;
                let currency_code: String = row.get("currency")?;
                let currency = currency_from_code(&currency_code);
                let due_str: Option<String> = row.get("due_date")?;
                let due_date = match &due_str {
                    Some(s) => Some(parse_date(s)?),
                    None => None,
                };
                let amount_due: i64 = row.get("amount_due")?;
                let bucket = compute_bucket(due_date, &today_str);
                Ok(AgingRow {
                    invoice_id,
                    number: number.map(|n| n as u64),
                    client_id,
                    client_name,
                    total: Money::new(total_cents, currency),
                    amount_due: Money::new(amount_due, currency),
                    due_date,
                    bucket,
                })
            })
            .map_err(map_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
    }

    fn dashboard_summary(&self, today: NaiveDate) -> Result<DashboardSummary, RepoError> {
        let conn = self.db.lock();
        let year_start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
        let revenue_cents: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM invoices
                 WHERE status IN ('Finalized', 'Sent') AND date >= ?1",
                params![year_start.format("%Y-%m-%d").to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        let currency_code: Option<String> = conn
            .query_row(
                "SELECT MAX(currency) FROM invoices",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(map_err)?
            .flatten();
        let currency = currency_code
            .as_deref()
            .map(currency_from_code)
            .unwrap_or_else(|| Currency::new("EUR").unwrap());

        let outstanding_cents: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total - amount_paid), 0)
                 FROM v_invoice_payment_status
                 WHERE status IN ('Finalized', 'Sent')",
                [],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        let overdue_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM v_invoice_payment_status
                 WHERE status IN ('Finalized', 'Sent')
                   AND due_date IS NOT NULL
                   AND due_date < ?1
                   AND total - amount_paid > 0",
                params![today.format("%Y-%m-%d").to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        let draft_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM invoices WHERE status = 'Draft'", [], |r| {
                r.get(0)
            })
            .map_err(map_err)?;
        let finalized_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM invoices WHERE status = 'Finalized'",
                [],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        let sent_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM invoices WHERE status = 'Sent'", [], |r| {
                r.get(0)
            })
            .map_err(map_err)?;

        Ok(DashboardSummary {
            revenue_this_year: Money::new(revenue_cents, currency),
            outstanding_total: Money::new(outstanding_cents, currency),
            overdue_count: overdue_count as u64,
            draft_count: draft_count as u64,
            finalized_count: finalized_count as u64,
            sent_count: sent_count as u64,
        })
    }
}

fn row_to_client_balance(row: &Row<'_>) -> rusqlite::Result<ClientBalance> {
    let id_str: String = row.get("id")?;
    let client_id: ClientId = parse_uuid(&id_str, ClientId)?;
    let name: String = row.get("name")?;
    let total_invoiced: i64 = row.get("total_invoiced")?;
    let total_paid: i64 = row.get("total_paid")?;
    let outstanding: i64 = row.get("outstanding")?;
    // Balances are denominated in the app currency; we pick a sensible fallback.
    let currency = Currency::new("EUR").unwrap();
    Ok(ClientBalance {
        client_id,
        client_name: name,
        total_invoiced: Money::new(total_invoiced, currency),
        total_paid: Money::new(total_paid, currency),
        outstanding: Money::new(outstanding, currency),
    })
}

fn normalize_bucket(raw: &str, grouping: RevenueGrouping) -> String {
    match grouping {
        RevenueGrouping::Day => raw.to_string(),
        RevenueGrouping::Month => {
            // raw is "YYYY-MM-DD" from `date`, project to first-of-month
            if raw.len() >= 7 {
                format!("{}-01", &raw[0..7])
            } else {
                raw.to_string()
            }
        }
        RevenueGrouping::Year => {
            if raw.len() >= 4 {
                format!("{}-01-01", &raw[0..4])
            } else {
                raw.to_string()
            }
        }
    }
}

fn compute_bucket(due_date: Option<NaiveDate>, today_str: &str) -> AgingBucket {
    let today = NaiveDate::parse_from_str(today_str, "%Y-%m-%d").unwrap_or_default();
    let Some(due) = due_date else {
        return AgingBucket::Current;
    };
    let days = (today - due).num_days();
    if days <= 0 {
        AgingBucket::Current
    } else if days <= 30 {
        AgingBucket::Days1To30
    } else if days <= 60 {
        AgingBucket::Days31To60
    } else if days <= 90 {
        AgingBucket::Days61To90
    } else {
        AgingBucket::Days91Plus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::adapters::sqlite::{
        SqliteClientRepository, SqliteInvoiceRepository, SqlitePaymentRepository,
    };
    use crate::application::ports::{
        ClientRepository as _, InvoiceRepository as _, PaymentRepository as _,
    };
    use crate::domain::client::{Client, NewClient};
    use crate::domain::invoice::{Invoice, InvoiceNumber, NewInvoice};
    use crate::domain::line_item::NewLineItem;
    use crate::domain::money::Money;
    use crate::domain::payment::{NewPayment, NewPaymentAllocation, Payment, PaymentMethod};
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn seed_client(db: &Db, name: &str) -> ClientId {
        let client = Client::create(
            NewClient {
                name: name.into(),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        SqliteClientRepository::new(db.clone()).insert(&client).unwrap();
        client.id
    }

    static NEXT_NUMBER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

    fn seed_invoice(
        db: &Db,
        client_id: ClientId,
        date: NaiveDate,
        due_date: Option<NaiveDate>,
        total_cents: i64,
    ) -> InvoiceId {
        let mut invoice = Invoice::create_draft(
            NewInvoice {
                client_id,
                template_id: None,
                date,
                due_date,
                line_items: vec![NewLineItem {
                    description: "W".into(),
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

    fn seed_payment(
        db: &Db,
        client_id: ClientId,
        amount: i64,
        allocations: Vec<NewPaymentAllocation>,
    ) {
        let p = Payment::create(
            NewPayment {
                client_id,
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                amount: Money::new(amount, eur()),
                method: PaymentMethod::BankTransfer,
                reference: None,
                allocations,
                notes: None,
            },
            Utc::now(),
        )
        .unwrap();
        SqlitePaymentRepository::new(db.clone()).insert(&p).unwrap();
    }

    #[test]
    fn outstanding_excludes_fully_paid() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        let inv_open = seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 1),
            1000,
        );
        let inv_paid = seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 2),
            500,
        );
        seed_payment(
            &db,
            client,
            500,
            vec![NewPaymentAllocation {
                invoice_id: inv_paid,
                amount: Money::new(500, eur()),
            }],
        );

        let repo = SqliteAccountingRepository::new(db);
        let outstanding = repo.list_outstanding_invoices().unwrap();
        assert_eq!(outstanding.len(), 1);
        assert_eq!(outstanding[0].invoice_id, inv_open);
        assert_eq!(outstanding[0].amount_due.amount_cents, 1000);
    }

    #[test]
    fn overdue_uses_today_cutoff() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 1),
            1000,
        );
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 7, 1),
            2000,
        );
        let repo = SqliteAccountingRepository::new(db);
        let overdue = repo
            .list_overdue_invoices(NaiveDate::from_ymd_opt(2026, 4, 14).unwrap())
            .unwrap();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].total.amount_cents, 1000);
    }

    #[test]
    fn revenue_by_month_sums_totals() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            None,
            1000,
        );
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            None,
            2500,
        );
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            None,
            500,
        );
        let repo = SqliteAccountingRepository::new(db);
        let buckets = repo
            .revenue_by_period(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                RevenueGrouping::Month,
            )
            .unwrap();
        assert_eq!(buckets.len(), 2);
        let march = buckets.iter().find(|b| b.bucket_start.month() == 3).unwrap();
        assert_eq!(march.amount.amount_cents, 3500);
        assert_eq!(march.invoice_count, 2);
    }

    #[test]
    fn revenue_by_client_sorts_descending() {
        let db = open_memory();
        let a = seed_client(&db, "Alpha");
        let b = seed_client(&db, "Beta");
        seed_invoice(
            &db,
            a,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            None,
            1000,
        );
        seed_invoice(
            &db,
            b,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            None,
            5000,
        );
        let repo = SqliteAccountingRepository::new(db);
        let list = repo
            .revenue_by_client(
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            )
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].client_name, "Beta");
        assert_eq!(list[0].total_invoiced.amount_cents, 5000);
    }

    #[test]
    fn client_balance_reflects_payments() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        let inv = seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            None,
            1000,
        );
        seed_payment(
            &db,
            client,
            400,
            vec![NewPaymentAllocation {
                invoice_id: inv,
                amount: Money::new(400, eur()),
            }],
        );
        let repo = SqliteAccountingRepository::new(db);
        let balance = repo.client_balance(client).unwrap();
        assert_eq!(balance.total_invoiced.amount_cents, 1000);
        assert_eq!(balance.total_paid.amount_cents, 400);
        assert_eq!(balance.outstanding.amount_cents, 600);
    }

    #[test]
    fn aging_report_buckets_correctly() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        // today = 2026-04-14
        // due 2026-04-20 => Current
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 20),
            1000,
        );
        // due 2026-04-01 => 13 days late => Days1To30
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 1),
            2000,
        );
        // due 2026-01-01 => 103 days late => Days91Plus
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2025, 12, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 1),
            3000,
        );
        let repo = SqliteAccountingRepository::new(db);
        let today = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
        let rows = repo.aging_report(today).unwrap();
        assert_eq!(rows.len(), 3);
        let buckets: Vec<AgingBucket> = rows.iter().map(|r| r.bucket).collect();
        assert!(buckets.contains(&AgingBucket::Current));
        assert!(buckets.contains(&AgingBucket::Days1To30));
        assert!(buckets.contains(&AgingBucket::Days91Plus));
    }

    #[test]
    fn dashboard_summary_aggregates_counts_and_totals() {
        let db = open_memory();
        let client = seed_client(&db, "Acme");
        // Finalized, current year, not overdue (due far in future)
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 1),
            1000,
        );
        // Overdue: due 2026-01-01, today 2026-04-14
        seed_invoice(
            &db,
            client,
            NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 1),
            2000,
        );
        let repo = SqliteAccountingRepository::new(db);
        let today = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
        let summary = repo.dashboard_summary(today).unwrap();
        // Only the 2026 invoice counts toward revenue_this_year.
        assert_eq!(summary.revenue_this_year.amount_cents, 1000);
        assert_eq!(summary.outstanding_total.amount_cents, 3000);
        assert_eq!(summary.overdue_count, 1);
        assert_eq!(summary.finalized_count, 2);
    }
}
