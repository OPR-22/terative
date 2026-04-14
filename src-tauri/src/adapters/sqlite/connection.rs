use std::path::Path;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub type Db = Arc<Mutex<Connection>>;

const INITIAL_SQL: &str = include_str!("../../../migrations/001_initial.sql");
const NOTEBOOK_SQL: &str = include_str!("../../../migrations/002_notebook.sql");

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(INITIAL_SQL), M::up(NOTEBOOK_SQL)])
}

pub fn open(path: &Path) -> anyhow::Result<Db> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrations().to_latest(&mut conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

#[cfg(test)]
pub fn open_memory() -> Db {
    let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    migrations().to_latest(&mut conn).expect("run migrations");
    Arc::new(Mutex::new(conn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_cleanly_in_memory() {
        let db = open_memory();
        let conn = db.lock();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clients'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn singleton_settings_rows_are_seeded() {
        let db = open_memory();
        let conn = db.lock();
        let seller_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM seller_profile WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(seller_ok, 1);
        let cur_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM currency_config WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(cur_ok, 1);
        let prefs_ok: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_preferences WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(prefs_ok, 1);
    }

    #[test]
    fn views_are_created() {
        let db = open_memory();
        let conn = db.lock();
        for view in ["v_invoice_payment_status", "v_client_balance", "v_aging_report"] {
            let c: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name=?1",
                    rusqlite::params![view],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(c, 1, "view {view} should exist");
        }
    }
}
