use rusqlite::{params, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{SearchEntityKind, SearchHit, SearchRepository};
use crate::application::RepoError;

pub struct SqliteSearchRepository {
    db: Db,
}

impl SqliteSearchRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn map_err(e: rusqlite::Error) -> RepoError {
    RepoError::Storage(e.to_string())
}

fn row_to_hit(row: &Row<'_>) -> rusqlite::Result<SearchHit> {
    let kind_str: String = row.get("entity_type")?;
    let kind = SearchEntityKind::parse(&kind_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown search entity_type: {kind_str}").into(),
        )
    })?;
    let id_str: String = row.get("entity_id")?;
    let entity_id = Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(SearchHit {
        kind,
        entity_id,
        title: row.get("title")?,
        snippet: row.get("snippet")?,
    })
}

impl SearchRepository for SqliteSearchRepository {
    fn search(&self, fts_query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
        let conn = self.db.lock();
        // `snippet(search_index, 3, …)` excerpts the `body` column (index 3:
        // entity_type=0, entity_id=1, title=2, body=3). `ORDER BY rank` is
        // FTS5's bm25 relevance — ascending puts the best matches first.
        let mut stmt = conn
            .prepare(
                "SELECT entity_type, entity_id, title, \
                        snippet(search_index, 3, '', '', '…', 12) AS snippet \
                 FROM search_index \
                 WHERE search_index MATCH ?1 \
                 ORDER BY rank \
                 LIMIT ?2",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map(params![fts_query, limit], row_to_hit)
            .map_err(map_err)?;
        let mut hits = Vec::new();
        for row in rows {
            hits.push(row.map_err(map_err)?);
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    fn insert_client(db: &Db, id: Uuid, name: &str, contact: Option<&str>) {
        db.lock()
            .execute(
                "INSERT INTO clients (id, kind, name, contact_name, default_currency, created_at) \
                 VALUES (?1, 'Individual', ?2, ?3, 'EUR', '2026-01-01T00:00:00Z')",
                params![id.to_string(), name, contact],
            )
            .unwrap();
    }

    fn insert_catalog_item(db: &Db, id: Uuid, name: &str, reference: Option<&str>) {
        db.lock()
            .execute(
                "INSERT INTO catalog_items (id, name, kind, reference) \
                 VALUES (?1, ?2, 'Service', ?3)",
                params![id.to_string(), name, reference],
            )
            .unwrap();
    }

    /// Inserts an invoice. A `None` number is a draft — drafts carry nothing
    /// searchable and are deliberately kept out of the index.
    fn insert_invoice(db: &Db, id: Uuid, client_id: Uuid, number: Option<i64>) {
        db.lock()
            .execute(
                "INSERT INTO invoices \
                   (id, number, client_id, date, subtotal, tax_total, total, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, '2026-01-01', 0, 0, 0, \
                         '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                params![id.to_string(), number, client_id.to_string()],
            )
            .unwrap();
    }

    fn repo(db: &Db) -> SqliteSearchRepository {
        SqliteSearchRepository::new(db.clone())
    }

    #[test]
    fn finds_client_by_name() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);

        let hits = repo(&db).search("\"acme\"*", 20).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, SearchEntityKind::Client);
        assert_eq!(hits[0].entity_id, id);
        assert_eq!(hits[0].title, "Acme Corporation");
    }

    #[test]
    fn finds_client_by_secondary_text() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", Some("Wile E. Coyote"));

        let hits = repo(&db).search("\"coyote\"*", 20).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, id);
    }

    #[test]
    fn finds_catalog_item_by_name_and_reference() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_catalog_item(&db, id, "Consulting hour", Some("CONS-01"));

        let by_name = repo(&db).search("\"consulting\"*", 20).unwrap();
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].kind, SearchEntityKind::CatalogItem);

        let by_ref = repo(&db).search("\"CONS\"*", 20).unwrap();
        assert_eq!(by_ref.len(), 1);
        assert_eq!(by_ref[0].entity_id, id);
    }

    #[test]
    fn finds_invoice_by_number() {
        let db = open_memory();
        let client_id = Uuid::new_v4();
        insert_client(&db, client_id, "Acme", None);
        let invoice_id = Uuid::new_v4();
        insert_invoice(&db, invoice_id, client_id, Some(42));

        let hits = repo(&db).search("\"42\"*", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, SearchEntityKind::Invoice);
        assert_eq!(hits[0].entity_id, invoice_id);
        assert_eq!(hits[0].title, "42");
    }

    #[test]
    fn draft_invoice_is_not_indexed() {
        let db = open_memory();
        let client_id = Uuid::new_v4();
        insert_client(&db, client_id, "Acme", None);
        insert_invoice(&db, Uuid::new_v4(), client_id, None);

        let indexed: i64 = db
            .lock()
            .query_row(
                "SELECT count(*) FROM search_index WHERE entity_type = 'invoice'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(indexed, 0, "a draft has no number and nothing else to index");
    }

    #[test]
    fn finalizing_a_draft_adds_it_to_the_index() {
        let db = open_memory();
        let client_id = Uuid::new_v4();
        insert_client(&db, client_id, "Acme", None);
        let invoice_id = Uuid::new_v4();
        insert_invoice(&db, invoice_id, client_id, None);

        // Assigning a number is what `FinalizeInvoice` does.
        db.lock()
            .execute(
                "UPDATE invoices SET number = 99 WHERE id = ?1",
                params![invoice_id.to_string()],
            )
            .unwrap();

        let hits = repo(&db).search("\"99\"*", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, invoice_id);
    }

    #[test]
    fn prefix_query_matches_partial_words() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Acme Corporation", None);

        let hits = repo(&db).search("\"acm\"*", 20).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn diacritics_are_folded() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Café Belge", None);

        // Query without the accent still matches the accented name.
        let hits = repo(&db).search("\"cafe\"*", 20).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn update_trigger_keeps_index_in_sync() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        db.lock()
            .execute(
                "UPDATE clients SET name = 'Globex Corporation' WHERE id = ?1",
                params![id.to_string()],
            )
            .unwrap();

        assert!(repo(&db).search("\"acme\"*", 20).unwrap().is_empty());
        let hits = repo(&db).search("\"globex\"*", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Globex Corporation");
    }

    #[test]
    fn delete_trigger_removes_from_index() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        db.lock()
            .execute("DELETE FROM clients WHERE id = ?1", params![id.to_string()])
            .unwrap();

        assert!(repo(&db).search("\"acme\"*", 20).unwrap().is_empty());
    }

    #[test]
    fn one_query_spans_multiple_entity_types() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Northwind Traders", None);
        insert_catalog_item(&db, Uuid::new_v4(), "Northwind delivery", None);

        let hits = repo(&db).search("\"northwind\"*", 20).unwrap();
        assert_eq!(hits.len(), 2);
        let kinds: std::collections::HashSet<_> = hits.iter().map(|h| h.kind).collect();
        assert!(kinds.contains(&SearchEntityKind::Client));
        assert!(kinds.contains(&SearchEntityKind::CatalogItem));
    }

    #[test]
    fn limit_caps_the_result_count() {
        let db = open_memory();
        for i in 0..10 {
            insert_client(&db, Uuid::new_v4(), &format!("Acme {i}"), None);
        }

        let hits = repo(&db).search("\"acme\"*", 3).unwrap();
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn no_match_returns_empty() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Acme Corporation", None);

        let hits = repo(&db).search("\"nonexistent\"*", 20).unwrap();
        assert!(hits.is_empty());
    }
}
