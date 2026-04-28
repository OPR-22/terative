use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
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

fn row_to_item(row: &Row<'_>) -> rusqlite::Result<CatalogItem> {
    let id_str: String = row.get("id")?;
    let id = CatalogItemId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let amount_minor: i64 = row.get("default_price")?;
    let currency_code: String = row.get("currency")?;
    let currency = Currency::new(&currency_code).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
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
        default_price: Money::new(amount_minor, currency),
        unit: row.get("unit")?,
        reference: row.get("reference")?,
        archived_at,
    })
}

impl CatalogItemRepository for SqliteCatalogItemRepository {
    fn insert(&self, item: &CatalogItem) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO catalog_items
             (id, name, kind, default_price, currency, unit, reference, archived_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                item.id.to_string(),
                item.name,
                item.kind.as_str(),
                item.default_price.minor_units(),
                item.default_price.currency().code(),
                item.unit,
                item.reference,
                item.archived_at.map(|d| d.to_rfc3339()),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, item: &CatalogItem) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE catalog_items
                 SET name = ?2, kind = ?3, default_price = ?4, currency = ?5,
                     unit = ?6, reference = ?7, archived_at = ?8
                 WHERE id = ?1",
                params![
                    item.id.to_string(),
                    item.name,
                    item.kind.as_str(),
                    item.default_price.minor_units(),
                    item.default_price.currency().code(),
                    item.unit,
                    item.reference,
                    item.archived_at.map(|d| d.to_rfc3339()),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: CatalogItemId) -> Result<Option<CatalogItem>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, kind, default_price, currency, unit, reference, archived_at
             FROM catalog_items WHERE id = ?1",
            params![id.to_string()],
            row_to_item,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self, include_archived: bool) -> Result<Vec<CatalogItem>, RepoError> {
        let conn = self.db.lock();
        let sql = if include_archived {
            "SELECT id, name, kind, default_price, currency, unit, reference, archived_at
             FROM catalog_items
             ORDER BY kind ASC, name COLLATE NOCASE ASC"
        } else {
            "SELECT id, name, kind, default_price, currency, unit, reference, archived_at
             FROM catalog_items
             WHERE archived_at IS NULL
             ORDER BY kind ASC, name COLLATE NOCASE ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_item).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn delete(&self, id: CatalogItemId) -> Result<(), RepoError> {
        let conn = self.db.lock();
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

    fn make_service(name: &str, cents: i64) -> CatalogItem {
        CatalogItem::create(NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Service,
            default_price: Money::new(cents, eur()),
            unit: Some("hour".into()),
            reference: None,
        })
        .unwrap()
    }

    fn make_product(name: &str, cents: i64, sku: &str) -> CatalogItem {
        CatalogItem::create(NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Product,
            default_price: Money::new(cents, eur()),
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
        assert_eq!(loaded.default_price.minor_units(), 15000);
        assert_eq!(loaded.default_price.currency().code(),"EUR");
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
    fn update_changes_price_and_currency() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();
        let usd = Currency::new("USD").unwrap();
        s.default_price = Money::new(20000, usd);
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.default_price.currency().code(),"USD");
        assert_eq!(loaded.default_price.minor_units(), 20000);
    }

    #[test]
    fn update_round_trips_kind_unit_and_reference() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let mut s = make_service("Consulting", 15000);
        repo.insert(&s).unwrap();

        // Change kind, rewrite unit, assign a reference.
        s.kind = CatalogItemKind::Product;
        s.unit = Some("license".into());
        s.reference = Some("LIC-42".into());
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.kind, CatalogItemKind::Product);
        assert_eq!(loaded.unit.as_deref(), Some("license"));
        assert_eq!(loaded.reference.as_deref(), Some("LIC-42"));

        // Clear the optional fields to None and flip kind back.
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
    fn list_sorts_by_kind_then_name() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        repo.insert(&make_service("Consulting", 15000)).unwrap();
        repo.insert(&make_product("Book", 2500, "SKU-001")).unwrap();
        repo.insert(&make_service("Audit", 20000)).unwrap();
        let list = repo.list(false).unwrap();
        // Kinds sort alphabetically: Product before Service. Within a kind,
        // name order applies.
        assert_eq!(list[0].name, "Book");
        assert_eq!(list[1].name, "Audit");
        assert_eq!(list[2].name, "Consulting");
    }

    #[test]
    fn delete_removes_row() {
        let db = open_memory();
        let repo = SqliteCatalogItemRepository::new(db);
        let s = make_service("X", 100);
        repo.insert(&s).unwrap();
        repo.delete(s.id).unwrap();
        assert!(repo.get(s.id).unwrap().is_none());
    }
}
