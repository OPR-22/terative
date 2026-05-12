use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::CatalogItemRepository;
use crate::application::RepoError;
use crate::domain::catalog_item::{CatalogItem, CatalogItemId, CatalogItemKind};
use crate::domain::money::{Currency, Money};

pub struct SqliteCatalogItemRepository {
    db: Db,
}

impl SqliteCatalogItemRepository {
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

/// Reads only the columns that live on `catalog_items`. Prices are stored on
/// the child table and attached by the caller.
fn row_to_item_without_prices(row: &Row<'_>) -> rusqlite::Result<CatalogItem> {
    let id_str: String = row.get("id")?;
    let id = CatalogItemId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let kind_str: String = row.get("kind")?;
    let kind = CatalogItemKind::parse(&kind_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown catalog item kind: {kind_str}"),
            )),
        )
    })?;
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
    Ok(CatalogItem {
        id,
        name: row.get("name")?,
        kind,
        prices: Vec::new(),
        unit: row.get("unit")?,
        reference: row.get("reference")?,
        archived_at,
    })
}

fn load_prices_for(
    conn: &Connection,
    item_id: CatalogItemId,
) -> Result<Vec<Money>, RepoError> {
    let mut stmt = conn
        .prepare(
            "SELECT currency, amount FROM catalog_item_prices
             WHERE catalog_item_id = ?1
             ORDER BY currency ASC",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![item_id.to_string()], |row| {
            let code: String = row.get("currency")?;
            let amount: i64 = row.get("amount")?;
            Ok((code, amount))
        })
        .map_err(map_err)?;
    let mut prices = Vec::new();
    for r in rows {
        let (code, amount) = r.map_err(map_err)?;
        let currency = Currency::new(&code).map_err(|e| {
            RepoError::Storage(format!("invalid currency code {code:?} on catalog_item_prices: {e}"))
        })?;
        prices.push(Money::new(amount, currency));
    }
    Ok(prices)
}

/// Bulk-fetches prices for many items in one query and indexes them by item.
/// Used by `list` to avoid N+1 queries.
fn load_all_prices(conn: &Connection) -> Result<HashMap<CatalogItemId, Vec<Money>>, RepoError> {
    let mut stmt = conn
        .prepare(
            "SELECT catalog_item_id, currency, amount FROM catalog_item_prices
             ORDER BY catalog_item_id ASC, currency ASC",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get("catalog_item_id")?;
            let code: String = row.get("currency")?;
            let amount: i64 = row.get("amount")?;
            Ok((id, code, amount))
        })
        .map_err(map_err)?;
    let mut out: HashMap<CatalogItemId, Vec<Money>> = HashMap::new();
    for r in rows {
        let (id_str, code, amount) = r.map_err(map_err)?;
        let id = Uuid::parse_str(&id_str)
            .map(CatalogItemId)
            .map_err(|e| RepoError::Storage(format!("invalid catalog_item_id: {e}")))?;
        let currency = Currency::new(&code).map_err(|e| {
            RepoError::Storage(format!("invalid currency code {code:?}: {e}"))
        })?;
        out.entry(id).or_default().push(Money::new(amount, currency));
    }
    Ok(out)
}

fn write_prices(
    conn: &Connection,
    item_id: CatalogItemId,
    prices: &[Money],
) -> Result<(), RepoError> {
    conn.execute(
        "DELETE FROM catalog_item_prices WHERE catalog_item_id = ?1",
        params![item_id.to_string()],
    )
    .map_err(map_err)?;
    for m in prices {
        conn.execute(
            "INSERT INTO catalog_item_prices (catalog_item_id, currency, amount)
             VALUES (?1, ?2, ?3)",
            params![item_id.to_string(), m.currency().code(), m.minor_units()],
        )
        .map_err(map_err)?;
    }
    Ok(())
}

impl CatalogItemRepository for SqliteCatalogItemRepository {
    fn insert(&self, item: &CatalogItem) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        tx.execute(
            "INSERT INTO catalog_items
             (id, name, kind, unit, reference, archived_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id.to_string(),
                item.name,
                item.kind.as_str(),
                item.unit,
                item.reference,
                item.archived_at.map(|d| d.to_rfc3339()),
            ],
        )
        .map_err(map_err)?;
        write_prices(&tx, item.id, &item.prices)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn update(&self, item: &CatalogItem) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        let affected = tx
            .execute(
                "UPDATE catalog_items
                 SET name = ?2, kind = ?3, unit = ?4, reference = ?5, archived_at = ?6
                 WHERE id = ?1",
                params![
                    item.id.to_string(),
                    item.name,
                    item.kind.as_str(),
                    item.unit,
                    item.reference,
                    item.archived_at.map(|d| d.to_rfc3339()),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        write_prices(&tx, item.id, &item.prices)?;
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn get(&self, id: CatalogItemId) -> Result<Option<CatalogItem>, RepoError> {
        let conn = self.db.lock();
        let Some(mut item) = conn
            .query_row(
                "SELECT id, name, kind, unit, reference, archived_at
                 FROM catalog_items WHERE id = ?1",
                params![id.to_string()],
                row_to_item_without_prices,
            )
            .optional()
            .map_err(map_err)?
        else {
            return Ok(None);
        };
        item.prices = load_prices_for(&conn, id)?;
        Ok(Some(item))
    }

    fn list(&self, include_archived: bool) -> Result<Vec<CatalogItem>, RepoError> {
        let conn = self.db.lock();
        let sql = if include_archived {
            "SELECT id, name, kind, unit, reference, archived_at
             FROM catalog_items
             ORDER BY kind ASC, name COLLATE NOCASE ASC"
        } else {
            "SELECT id, name, kind, unit, reference, archived_at
             FROM catalog_items
             WHERE archived_at IS NULL
             ORDER BY kind ASC, name COLLATE NOCASE ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_item_without_prices).map_err(map_err)?;
        let mut items: Vec<CatalogItem> = Vec::new();
        for r in rows {
            items.push(r.map_err(map_err)?);
        }
        let mut prices_by_id = load_all_prices(&conn)?;
        for item in items.iter_mut() {
            if let Some(p) = prices_by_id.remove(&item.id) {
                item.prices = p;
            }
        }
        Ok(items)
    }

    fn delete(&self, id: CatalogItemId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        // ON DELETE CASCADE on catalog_item_prices takes care of prices.
        conn.execute(
            "DELETE FROM catalog_items WHERE id = ?1",
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
    use crate::domain::catalog_item::NewCatalogItem;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    fn make_service(name: &str, cents: i64) -> CatalogItem {
        CatalogItem::create(NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(cents, eur())],
            unit: Some("hour".into()),
            reference: None,
        })
        .unwrap()
    }

    fn make_product(name: &str, cents: i64, sku: &str) -> CatalogItem {
        CatalogItem::create(NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Product,
            prices: vec![Money::new(cents, eur())],
            unit: Some("piece".into()),
            reference: Some(sku.into()),
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Consulting");
        assert_eq!(loaded.kind, CatalogItemKind::Service);
        assert_eq!(loaded.prices.len(), 1);
        assert_eq!(loaded.prices[0].minor_units(), 15000);
        assert_eq!(loaded.prices[0].currency().code(), "EUR");
        assert_eq!(loaded.unit.as_deref(), Some("hour"));
        assert_eq!(loaded.reference, None);
    }

    #[test]
    fn insert_persists_product_fields() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let p = make_product("Book", 2500, "SKU-042");
        repo.insert(&p).unwrap();
        let loaded = repo.get(p.id).unwrap().unwrap();
        assert_eq!(loaded.kind, CatalogItemKind::Product);
        assert_eq!(loaded.unit.as_deref(), Some("piece"));
        assert_eq!(loaded.reference.as_deref(), Some("SKU-042"));
    }

    #[test]
    fn insert_persists_multiple_currency_prices() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let s = CatalogItem::create(NewCatalogItem {
            name: "Consulting".into(),
            kind: CatalogItemKind::Service,
            prices: vec![Money::new(15000, eur()), Money::new(17000, usd())],
            unit: None,
            reference: None,
        })
        .unwrap();
        repo.insert(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.prices.len(), 2);
        assert_eq!(loaded.price_for(eur()).unwrap().minor_units(), 15000);
        assert_eq!(loaded.price_for(usd()).unwrap().minor_units(), 17000);
    }

    #[test]
    fn update_replaces_prices_wholesale() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();
        // Replace EUR-only with USD-only.
        s.replace_prices(vec![Money::new(20000, usd())]).unwrap();
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.prices.len(), 1);
        assert_eq!(loaded.prices[0].currency().code(), "USD");
        assert_eq!(loaded.prices[0].minor_units(), 20000);
    }

    #[test]
    fn update_can_clear_all_prices() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();
        s.replace_prices(vec![]).unwrap();
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert!(loaded.prices.is_empty());
    }

    #[test]
    fn update_round_trips_kind_unit_and_reference() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();

        s.kind = CatalogItemKind::Product;
        s.unit = Some("license".into());
        s.reference = Some("LIC-42".into());
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.kind, CatalogItemKind::Product);
        assert_eq!(loaded.unit.as_deref(), Some("license"));
        assert_eq!(loaded.reference.as_deref(), Some("LIC-42"));

        s.kind = CatalogItemKind::Service;
        s.unit = None;
        s.reference = None;
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.kind, CatalogItemKind::Service);
        assert_eq!(loaded.unit, None);
        assert_eq!(loaded.reference, None);
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let s = make_service("Ghost", 0);
        assert!(matches!(repo.update(&s), Err(RepoError::NotFound)));
    }

    #[test]
    fn list_excludes_archived_by_default() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut a = make_service("A", 100);
        a.archived_at = Some(Utc::now());
        repo.insert(&a).unwrap();
        repo.insert(&make_service("B", 200)).unwrap();
        let list = repo.list(false).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "B");
    }

    #[test]
    fn list_sorts_by_kind_then_name_and_attaches_prices() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        repo.insert(&make_service("Consulting", 15000)).unwrap();
        repo.insert(&make_product("Book", 2500, "SKU-001")).unwrap();
        repo.insert(&make_service("Audit", 20000)).unwrap();
        let list = repo.list(false).unwrap();
        assert_eq!(list[0].name, "Book");
        assert_eq!(list[1].name, "Audit");
        assert_eq!(list[2].name, "Consulting");
        for item in &list {
            assert_eq!(item.prices.len(), 1, "{}", item.name);
        }
    }

    #[test]
    fn delete_removes_row_and_prices() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let s = make_service("X", 100);
        repo.insert(&s).unwrap();
        repo.delete(s.id).unwrap();
        assert!(repo.get(s.id).unwrap().is_none());
        // Prices should have been cascade-deleted.
        let leftover: i64 = {
            let conn = repo.db.lock();
            conn.query_row(
                "SELECT COUNT(*) FROM catalog_item_prices WHERE catalog_item_id = ?1",
                params![s.id.to_string()],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(leftover, 0);
    }
}
