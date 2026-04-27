use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::BookmarkRepository;
use crate::application::RepoError;
use crate::domain::bookmark::{Bookmark, BookmarkId};

pub struct SqliteBookmarkRepository {
    db: Db,
}

impl SqliteBookmarkRepository {
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

fn row_to_bookmark(row: &Row<'_>) -> rusqlite::Result<Bookmark> {
    let id_str: String = row.get("id")?;
    let id = BookmarkId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    Ok(Bookmark {
        id,
        label: row.get("label")?,
        url: row.get("url")?,
        sort_order: row.get::<_, i64>("sort_order")? as i32,
    })
}

impl BookmarkRepository for SqliteBookmarkRepository {
    fn insert(&self, b: &Bookmark) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO bookmarks (id, label, url, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![b.id.to_string(), b.label, b.url, b.sort_order as i64],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, b: &Bookmark) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE bookmarks SET label = ?2, url = ?3, sort_order = ?4 WHERE id = ?1",
                params![b.id.to_string(), b.label, b.url, b.sort_order as i64],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: BookmarkId) -> Result<Option<Bookmark>, RepoError> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, label, url, sort_order FROM bookmarks WHERE id = ?1",
            params![id.to_string()],
            row_to_bookmark,
        )
        .optional()
        .map_err(map_err)
    }

    fn list(&self) -> Result<Vec<Bookmark>, RepoError> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, label, url, sort_order FROM bookmarks
                 ORDER BY sort_order ASC, label COLLATE NOCASE ASC",
            )
            .map_err(map_err)?;
        let rows = stmt.query_map([], row_to_bookmark).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn delete(&self, id: BookmarkId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM bookmarks WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn reorder(&self, ordered_ids: &[BookmarkId]) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        for (idx, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE bookmarks SET sort_order = ?2 WHERE id = ?1",
                params![id.to_string(), idx as i64],
            )
            .map_err(map_err)?;
        }
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn max_sort_order(&self) -> Result<i32, RepoError> {
        let conn = self.db.lock();
        let max: Option<i64> = conn
            .query_row("SELECT MAX(sort_order) FROM bookmarks", [], |r| r.get(0))
            .optional()
            .map_err(map_err)?
            .flatten();
        Ok(max.map(|n| n as i32).unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    fn make(label: &str, url: &str, sort_order: i32) -> Bookmark {
        Bookmark::create(label.into(), url.into(), sort_order).unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        let b = make("Google", "https://google.com", 0);
        repo.insert(&b).unwrap();
        let loaded = repo.get(b.id).unwrap().unwrap();
        assert_eq!(loaded.label, "Google");
        assert_eq!(loaded.url, "https://google.com");
        assert_eq!(loaded.sort_order, 0);
    }

    #[test]
    fn update_changes_fields() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        let mut b = make("Old", "https://old.com", 0);
        repo.insert(&b).unwrap();
        b.label = "New".into();
        b.url = "https://new.com".into();
        b.sort_order = 5;
        repo.update(&b).unwrap();
        let loaded = repo.get(b.id).unwrap().unwrap();
        assert_eq!(loaded.label, "New");
        assert_eq!(loaded.url, "https://new.com");
        assert_eq!(loaded.sort_order, 5);
    }

    #[test]
    fn update_missing_is_not_found() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        let b = make("Ghost", "https://ghost.com", 0);
        assert!(matches!(repo.update(&b), Err(RepoError::NotFound)));
    }

    #[test]
    fn list_sorts_by_sort_order_then_label() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        repo.insert(&make("Zed", "https://zed.com", 2)).unwrap();
        repo.insert(&make("Alpha", "https://alpha.com", 0)).unwrap();
        repo.insert(&make("Beta", "https://beta.com", 1)).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(
            list.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "Beta", "Zed"]
        );
    }

    #[test]
    fn delete_removes_row() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        let b = make("X", "https://x.com", 0);
        repo.insert(&b).unwrap();
        repo.delete(b.id).unwrap();
        assert!(repo.get(b.id).unwrap().is_none());
    }

    #[test]
    fn reorder_assigns_contiguous_sort_order() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        let a = make("A", "https://a.com", 0);
        let b = make("B", "https://b.com", 1);
        let c = make("C", "https://c.com", 2);
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        repo.insert(&c).unwrap();
        repo.reorder(&[c.id, a.id, b.id]).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(
            list.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(),
            vec!["C", "A", "B"]
        );
        assert_eq!(list[0].sort_order, 0);
        assert_eq!(list[1].sort_order, 1);
        assert_eq!(list[2].sort_order, 2);
    }

    #[test]
    fn max_sort_order_returns_minus_one_when_empty() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        assert_eq!(repo.max_sort_order().unwrap(), -1);
    }

    #[test]
    fn max_sort_order_returns_highest() {
        let db = open_memory();
        let repo = SqliteBookmarkRepository::new(db);
        repo.insert(&make("A", "https://a.com", 2)).unwrap();
        repo.insert(&make("B", "https://b.com", 5)).unwrap();
        repo.insert(&make("C", "https://c.com", 3)).unwrap();
        assert_eq!(repo.max_sort_order().unwrap(), 5);
    }
}
