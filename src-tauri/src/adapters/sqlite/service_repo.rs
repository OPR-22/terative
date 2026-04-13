use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::ServiceRepository;
use crate::application::RepoError;
use crate::domain::money::{Currency, Money};
use crate::domain::service::{Service, ServiceId};

pub struct SqliteServiceRepository {
    db: Db,
}

impl SqliteServiceRepository {
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

fn row_to_service(row: &Row<'_>) -> rusqlite::Result<Service> {
    let id_str: String = row.get("id")?;
    let id = ServiceId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let amount_cents: i64 = row.get("default_price")?;
    let currency_code: String = row.get("currency")?;
    let currency = Currency::new(&currency_code).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
        )
    })?;
    Ok(Service {
        id,
        name: row.get("name")?,
        default_price: Money::new(amount_cents, currency),
        active: row.get::<_, i64>("active")? != 0,
    })
}

impl ServiceRepository for SqliteServiceRepository {
    fn insert(&self, s: &Service) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO services (id, name, default_price, currency, active)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                s.id.to_string(),
                s.name,
                s.default_price.amount_cents,
                s.default_price.currency.code(),
                s.active as i64,
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, s: &Service) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE services
                 SET name = ?2, default_price = ?3, currency = ?4, active = ?5
                 WHERE id = ?1",
                params![
                    s.id.to_string(),
                    s.name,
                    s.default_price.amount_cents,
                    s.default_price.currency.code(),
                    s.active as i64,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: ServiceId) -> Result<Option<Service>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, default_price, currency, active FROM services WHERE id = ?1",
            params![id.to_string()],
            row_to_service,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self, include_inactive: bool) -> Result<Vec<Service>, RepoError> {
        let conn = self.db.lock();
        let sql = if include_inactive {
            "SELECT id, name, default_price, currency, active FROM services ORDER BY name COLLATE NOCASE ASC"
        } else {
            "SELECT id, name, default_price, currency, active FROM services WHERE active = 1 ORDER BY name COLLATE NOCASE ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_service).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn delete(&self, id: ServiceId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute("DELETE FROM services WHERE id = ?1", params![id.to_string()])
            .map_err(map_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;
    use crate::domain::service::NewService;

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn make(name: &str, cents: i64) -> Service {
        Service::create(NewService {
            name: name.into(),
            default_price: Money::new(cents, eur()),
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteServiceRepository::new(db);
        let s = make("Consulting", 15000);
        repo.insert(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Consulting");
        assert_eq!(loaded.default_price.amount_cents, 15000);
        assert_eq!(loaded.default_price.currency.code(), "EUR");
    }

    #[test]
    fn update_changes_price_and_currency() {
        let db = open_memory();
        let repo = SqliteServiceRepository::new(db);
        let mut s = make("Consulting", 15000);
        repo.insert(&s).unwrap();
        let usd = Currency::new("USD").unwrap();
        s.default_price = Money::new(20000, usd);
        repo.update(&s).unwrap();
        let loaded = repo.get(s.id).unwrap().unwrap();
        assert_eq!(loaded.default_price.currency.code(), "USD");
        assert_eq!(loaded.default_price.amount_cents, 20000);
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let repo = SqliteServiceRepository::new(db);
        let s = make("Ghost", 0);
        assert!(matches!(repo.update(&s), Err(RepoError::NotFound)));
    }

    #[test]
    fn list_excludes_inactive_by_default() {
        let db = open_memory();
        let repo = SqliteServiceRepository::new(db);
        let mut a = make("A", 100);
        a.active = false;
        repo.insert(&a).unwrap();
        repo.insert(&make("B", 200)).unwrap();
        let list = repo.list(false).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "B");
    }

    #[test]
    fn delete_removes_row() {
        let db = open_memory();
        let repo = SqliteServiceRepository::new(db);
        let s = make("X", 100);
        repo.insert(&s).unwrap();
        repo.delete(s.id).unwrap();
        assert!(repo.get(s.id).unwrap().is_none());
    }
}
