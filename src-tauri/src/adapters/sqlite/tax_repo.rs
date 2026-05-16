use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::TaxRepository;
use crate::application::RepoError;
use crate::domain::tax::{TaxDefinition, TaxId};

pub struct SqliteTaxRepository {
    db: Db,
}

impl SqliteTaxRepository {
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

fn row_to_tax(row: &Row<'_>) -> rusqlite::Result<TaxDefinition> {
    let id_str: String = row.get("id")?;
    let id = TaxId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let pct: f64 = row.get("percentage")?;
    let percentage = Decimal::from_f64(pct).unwrap_or(Decimal::ZERO);
    let archived_at = parse_archived_at(row, "archived_at")?;
    Ok(TaxDefinition {
        id,
        name: row.get("name")?,
        percentage,
        tax_id_number: row.get("tax_id_number")?,
        archived_at,
        pending_events: crate::domain::events::EventBuffer::default(),
    })
}

fn parse_archived_at(
    row: &Row<'_>,
    column: &str,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    match row.get::<_, Option<String>>(column)? {
        None => Ok(None),
        Some(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            }),
    }
}

fn pct_to_f64(d: Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}

impl TaxRepository for SqliteTaxRepository {
    fn insert(&self, t: &TaxDefinition) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO tax_definitions (id, name, percentage, tax_id_number, archived_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                t.id.to_string(),
                t.name,
                pct_to_f64(t.percentage),
                t.tax_id_number,
                t.archived_at.map(|d| d.to_rfc3339()),
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, t: &TaxDefinition) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE tax_definitions
                 SET name = ?2, percentage = ?3, tax_id_number = ?4, archived_at = ?5
                 WHERE id = ?1",
                params![
                    t.id.to_string(),
                    t.name,
                    pct_to_f64(t.percentage),
                    t.tax_id_number,
                    t.archived_at.map(|d| d.to_rfc3339()),
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: TaxId) -> Result<Option<TaxDefinition>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, percentage, tax_id_number, archived_at FROM tax_definitions WHERE id = ?1",
            params![id.to_string()],
            row_to_tax,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self, include_archived: bool) -> Result<Vec<TaxDefinition>, RepoError> {
        let conn = self.db.lock();
        let sql = if include_archived {
            "SELECT id, name, percentage, tax_id_number, archived_at FROM tax_definitions ORDER BY name COLLATE NOCASE ASC"
        } else {
            "SELECT id, name, percentage, tax_id_number, archived_at FROM tax_definitions WHERE archived_at IS NULL ORDER BY name COLLATE NOCASE ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_tax).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn get_many(&self, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.db.lock();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, name, percentage, tax_id_number, archived_at FROM tax_definitions WHERE id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let id_strs: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
        let params = rusqlite::params_from_iter(id_strs.iter());
        let rows = stmt.query_map(params, row_to_tax).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        // Preserve caller-supplied order.
        let mut ordered = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(pos) = out.iter().position(|t| t.id == *id) {
                ordered.push(out.swap_remove(pos));
            }
        }
        Ok(ordered)
    }

    fn delete(&self, id: TaxId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM tax_definitions WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn labels_for(
        &self,
        ids: &[TaxId],
    ) -> Result<std::collections::HashMap<TaxId, String>, RepoError> {
        use std::collections::HashMap;
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT id, name FROM tax_definitions WHERE id IN ({})",
            placeholders.join(", ")
        );
        let id_strs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let params: Vec<&dyn rusqlite::ToSql> =
            id_strs.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map(params.as_slice(), |row| {
                let id_str: String = row.get("id")?;
                let uuid = uuid::Uuid::parse_str(&id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                let name: String = row.get("name")?;
                Ok((TaxId(uuid), name))
            })
            .map_err(map_err)?;
        let mut out = HashMap::new();
        for r in rows {
            let (id, name) = r.map_err(map_err)?;
            out.insert(id, name);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::tax::NewTaxDefinition;
    use rust_decimal_macros::dec;

    fn make(name: &str, pct: Decimal) -> TaxDefinition {
        TaxDefinition::create(NewTaxDefinition {
            name: name.into(),
            percentage: pct,
            tax_id_number: None,
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteTaxRepository::new(db);
        let t = make("TVA", dec!(21));
        repo.insert(&t).unwrap();
        let loaded = repo.get(t.id).unwrap().unwrap();
        assert_eq!(loaded.name, "TVA");
        assert_eq!(loaded.percentage, dec!(21));
    }

    #[test]
    fn get_many_preserves_order() {
        let db = open_memory();
        let repo = SqliteTaxRepository::new(db);
        let a = make("A", dec!(10));
        let b = make("B", dec!(20));
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        let r = repo.get_many(&[b.id, a.id]).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].id, b.id);
        assert_eq!(r[1].id, a.id);
    }

    #[test]
    fn list_excludes_archived_by_default() {
        let db = open_memory();
        let repo = SqliteTaxRepository::new(db);
        let mut t = make("Old", dec!(5));
        t.archived_at = Some(Utc::now());
        repo.insert(&t).unwrap();
        repo.insert(&make("New", dec!(10))).unwrap();
        assert_eq!(repo.list(false).unwrap().len(), 1);
        assert_eq!(repo.list(true).unwrap().len(), 2);
    }
}
