use std::collections::HashSet;

use rusqlite::{params, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::{SearchEntityKind, SearchHit, SearchRepository};
use crate::application::RepoError;
use crate::kernel::text::digits_only;

/// Minimum length before a query runs the email substring scan. Below this
/// a `LIKE '%x%'` is mostly noise.
const MIN_LIKE_QUERY_LEN: usize = 3;

/// Minimum digits before a query is treated as a phone search. Below this a
/// query like "42" is an invoice-number lookup, not a phone number.
const MIN_PHONE_QUERY_DIGITS: usize = 3;

pub struct SqliteSearchRepository {
    db: Db,
}

impl SqliteSearchRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Full-text pass over `search_index`: client names / contact / tax IDs,
    /// catalog names / references, and invoice numbers. Best matches first.
    fn fts_hits(&self, raw: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
        let Some(fts_query) = build_fts_query(raw) else {
            return Ok(Vec::new());
        };
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

    /// Substring pass over client email addresses. `LIKE` matches any
    /// position, so "smith" finds "johnsmith@…" — something FTS prefix
    /// matching could not do.
    fn email_hits(&self, raw: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
        let trimmed = raw.trim();
        if trimmed.chars().count() < MIN_LIKE_QUERY_LEN {
            return Ok(Vec::new());
        }
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT c.id AS client_id, c.name AS client_name, e.value AS matched \
                 FROM client_emails e \
                 JOIN clients c ON c.id = e.client_id \
                 WHERE e.value LIKE ?1 ESCAPE '\\' \
                 ORDER BY c.name",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map(params![like_pattern(trimmed)], contact_row)
            .map_err(map_err)?;
        dedup_client_hits(rows, limit)
    }

    /// Substring pass over client phone numbers. Both the stored number and
    /// the query are reduced to digits, so the match ignores how either
    /// side was spaced or punctuated. Filtered in Rust because SQL has no
    /// way to strip the punctuation from the stored value.
    fn phone_hits(&self, raw: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
        let Some(digits) = phone_query(raw) else {
            return Ok(Vec::new());
        };
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT c.id AS client_id, c.name AS client_name, p.value AS matched \
                 FROM client_phones p \
                 JOIN clients c ON c.id = p.client_id \
                 ORDER BY c.name",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map([], contact_row)
            .map_err(map_err)?;
        // Keep `Err` rows so the failure surfaces in `dedup_client_hits`.
        let matching = rows.filter(|r| match r {
            Ok((_, _, phone)) => digits_only(phone).contains(&digits),
            Err(_) => true,
        });
        dedup_client_hits(matching, limit)
    }
}

impl SearchRepository for SqliteSearchRepository {
    fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchHit>, RepoError> {
        // Full-text first (it is the ranked pass), then the email and phone
        // substring passes merged in — skipping any client already found.
        let mut hits = self.fts_hits(query, limit)?;
        for hit in self.email_hits(query, limit)? {
            push_unique(&mut hits, hit);
        }
        for hit in self.phone_hits(query, limit)? {
            push_unique(&mut hits, hit);
        }
        hits.truncate(limit as usize);
        Ok(hits)
    }
}

fn map_err(e: rusqlite::Error) -> RepoError {
    RepoError::Storage(e.to_string())
}

/// Appends `hit` unless an entity of the same kind and id is already
/// present — a client can match by name, email and phone at once.
fn push_unique(hits: &mut Vec<SearchHit>, hit: SearchHit) {
    let dup = hits
        .iter()
        .any(|h| h.kind == hit.kind && h.entity_id == hit.entity_id);
    if !dup {
        hits.push(hit);
    }
}

type ContactRow = (String, String, String);

/// Row shape shared by the email and phone scans: client id, client name,
/// and the matched value (used as the hit's snippet).
fn contact_row(row: &Row<'_>) -> rusqlite::Result<ContactRow> {
    Ok((
        row.get("client_id")?,
        row.get("client_name")?,
        row.get("matched")?,
    ))
}

/// Collapses `(client_id, name, value)` rows into client `SearchHit`s — one
/// per client, `snippet` set to the value that matched — capped at `limit`.
fn dedup_client_hits(
    rows: impl Iterator<Item = rusqlite::Result<ContactRow>>,
    limit: u32,
) -> Result<Vec<SearchHit>, RepoError> {
    let mut hits: Vec<SearchHit> = Vec::new();
    let mut seen: HashSet<Uuid> = HashSet::new();
    for row in rows {
        let (id_str, name, value) = row.map_err(map_err)?;
        let entity_id = Uuid::parse_str(&id_str)
            .map_err(|e| RepoError::Storage(format!("invalid client id: {e}")))?;
        if !seen.insert(entity_id) {
            continue;
        }
        hits.push(SearchHit {
            kind: SearchEntityKind::Client,
            entity_id,
            title: name,
            snippet: value,
        });
        if hits.len() as u32 >= limit {
            break;
        }
    }
    Ok(hits)
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

/// Wraps `needle` in `%…%` for a `LIKE` substring match, escaping the
/// wildcard characters so user input is matched literally (paired with
/// `ESCAPE '\'` in the SQL).
fn like_pattern(needle: &str) -> String {
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// Build a safe FTS5 `MATCH` expression from raw user input.
///
/// The input is split on every non-alphanumeric character into maximal
/// alphanumeric runs — not just on whitespace. This mirrors how the FTS5
/// `unicode61` tokenizer split the indexed text, so typing a full
/// "john@acme.com" produces `"john"* "acme"* "com"*`. Dropping the
/// punctuation also strips FTS5 operator characters, so raw input can never
/// trigger a query-syntax error, and leaves nothing to escape in the quotes.
///
/// Each run becomes a quoted prefix term; terms are joined with spaces,
/// which is an implicit AND in FTS5. Returns `None` when no usable term
/// remains (blank or punctuation-only input).
fn build_fts_query(raw: &str) -> Option<String> {
    let terms: Vec<String> = raw
        .split(|c: char| !c.is_alphanumeric())
        .filter(|run| !run.is_empty())
        .map(|run| format!("\"{run}\"*"))
        .collect();

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

/// Returns the digit string to run a phone search for — but only when
/// `raw` *looks* like a phone number: nothing but digits and the
/// punctuation phone numbers are written with, and at least
/// `MIN_PHONE_QUERY_DIGITS` digits. Any letter means it is a name query,
/// not a phone, and yields `None`.
fn phone_query(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let phone_shaped = !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '+' | '-' | '(' | ')' | '.' | '/'));
    if !phone_shaped {
        return None;
    }
    let digits = digits_only(trimmed);
    (digits.len() >= MIN_PHONE_QUERY_DIGITS).then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    // ── build_fts_query ──

    #[test]
    fn fts_query_single_word_becomes_a_quoted_prefix_term() {
        assert_eq!(build_fts_query("acme").as_deref(), Some("\"acme\"*"));
    }

    #[test]
    fn fts_query_joins_words_as_prefix_terms() {
        assert_eq!(
            build_fts_query("acme corp").as_deref(),
            Some("\"acme\"* \"corp\"*")
        );
    }

    #[test]
    fn fts_query_splits_punctuation_and_strips_operators() {
        // `@ . * " ( )` are all separators — none reach FTS5 as syntax.
        assert_eq!(
            build_fts_query("john@acme.com").as_deref(),
            Some("\"john\"* \"acme\"* \"com\"*")
        );
        assert_eq!(
            build_fts_query("acme* OR (\"corp\")").as_deref(),
            Some("\"acme\"* \"OR\"* \"corp\"*")
        );
    }

    #[test]
    fn fts_query_is_none_for_blank_or_punctuation_only() {
        assert_eq!(build_fts_query("   "), None);
        assert_eq!(build_fts_query("!@#$ %^&*"), None);
    }

    // ── phone_query ──

    #[test]
    fn phone_query_extracts_digits_from_a_phone_shaped_string() {
        assert_eq!(
            phone_query("+32 470 12 34 56").as_deref(),
            Some("32470123456")
        );
        assert_eq!(phone_query("(0470) 12-34-56").as_deref(), Some("0470123456"));
    }

    #[test]
    fn phone_query_is_none_for_names_and_short_numbers() {
        assert_eq!(phone_query("acme"), None);
        assert_eq!(phone_query("acme 470"), None); // has letters
        assert_eq!(phone_query("42"), None); // only 2 digits
    }

    // ── integration ──

    fn insert_client(db: &Db, id: Uuid, name: &str, contact: Option<&str>) {
        db.lock()
            .execute(
                "INSERT INTO clients (id, kind, name, contact_name, default_currency, created_at) \
                 VALUES (?1, 'Individual', ?2, ?3, 'EUR', '2026-01-01T00:00:00Z')",
                params![id.to_string(), name, contact],
            )
            .unwrap();
    }

    fn insert_client_email(db: &Db, client_id: Uuid, value: &str) {
        db.lock()
            .execute(
                "INSERT INTO client_emails (id, client_id, value) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4().to_string(), client_id.to_string(), value],
            )
            .unwrap();
    }

    fn insert_client_phone(db: &Db, client_id: Uuid, value: &str) {
        db.lock()
            .execute(
                "INSERT INTO client_phones (id, client_id, value) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4().to_string(), client_id.to_string(), value],
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

        let hits = repo(&db).search("acme", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, SearchEntityKind::Client);
        assert_eq!(hits[0].entity_id, id);
        assert_eq!(hits[0].title, "Acme Corporation");
    }

    #[test]
    fn finds_client_by_contact_name() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", Some("Wile E. Coyote"));

        let hits = repo(&db).search("coyote", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, id);
    }

    #[test]
    fn finds_client_by_email_address() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        insert_client_email(&db, id, "billing@acme-corp.example");

        let hits = repo(&db).search("billing", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, SearchEntityKind::Client);
        assert_eq!(hits[0].entity_id, id);
        assert_eq!(hits[0].snippet, "billing@acme-corp.example");
    }

    #[test]
    fn finds_client_by_a_mid_string_email_fragment() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme", None);
        insert_client_email(&db, id, "johnsmith@acme.example");

        // "smith" is in the middle of a token — FTS prefix could not match
        // it, but the email substring scan can.
        let hits = repo(&db).search("smith", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, id);
    }

    #[test]
    fn finds_client_by_phone_number_regardless_of_punctuation() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme", None);
        insert_client_phone(&db, id, "+32 470 12 34 56");

        // Stored "+32 470 12 34 56" → digits 32470123456; the query is the
        // local part, differently spaced — still a digit substring.
        let hits = repo(&db).search("470 12 34 56", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, SearchEntityKind::Client);
        assert_eq!(hits[0].entity_id, id);
        assert_eq!(hits[0].snippet, "+32 470 12 34 56");
    }

    #[test]
    fn finds_client_by_a_trailing_phone_fragment() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme", None);
        insert_client_phone(&db, id, "0470 12 34 56");

        let hits = repo(&db).search("3456", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, id);
    }

    #[test]
    fn a_client_matched_by_name_and_email_appears_once() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        insert_client_email(&db, id, "hello@acme.example");

        // "acme" matches the name (full-text) and the email (substring).
        let hits = repo(&db).search("acme", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, id);
    }

    #[test]
    fn finds_catalog_item_by_name_and_reference() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_catalog_item(&db, id, "Consulting hour", Some("CONS-01"));

        assert_eq!(repo(&db).search("consulting", 20).unwrap().len(), 1);
        let by_ref = repo(&db).search("CONS", 20).unwrap();
        assert_eq!(by_ref.len(), 1);
        assert_eq!(by_ref[0].kind, SearchEntityKind::CatalogItem);
        assert_eq!(by_ref[0].entity_id, id);
    }

    #[test]
    fn finds_invoice_by_number() {
        let db = open_memory();
        let client_id = Uuid::new_v4();
        insert_client(&db, client_id, "Acme", None);
        let invoice_id = Uuid::new_v4();
        insert_invoice(&db, invoice_id, client_id, Some(42));

        let hits = repo(&db).search("42", 20).unwrap();
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
        db.lock()
            .execute(
                "UPDATE invoices SET number = 99 WHERE id = ?1",
                params![invoice_id.to_string()],
            )
            .unwrap();

        let hits = repo(&db).search("99", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, invoice_id);
    }

    #[test]
    fn diacritics_are_folded() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Café Belge", None);
        // Query without the accent still matches the accented name.
        assert_eq!(repo(&db).search("cafe", 20).unwrap().len(), 1);
    }

    #[test]
    fn prefix_query_matches_partial_words() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Acme Corporation", None);
        assert_eq!(repo(&db).search("acm", 20).unwrap().len(), 1);
    }

    #[test]
    fn update_keeps_the_index_in_sync() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        db.lock()
            .execute(
                "UPDATE clients SET name = 'Globex Corporation' WHERE id = ?1",
                params![id.to_string()],
            )
            .unwrap();

        assert!(repo(&db).search("acme", 20).unwrap().is_empty());
        assert_eq!(repo(&db).search("globex", 20).unwrap().len(), 1);
    }

    #[test]
    fn delete_removes_from_the_index() {
        let db = open_memory();
        let id = Uuid::new_v4();
        insert_client(&db, id, "Acme Corporation", None);
        db.lock()
            .execute("DELETE FROM clients WHERE id = ?1", params![id.to_string()])
            .unwrap();

        assert!(repo(&db).search("acme", 20).unwrap().is_empty());
    }

    #[test]
    fn one_query_spans_multiple_entity_types() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Northwind Traders", None);
        insert_catalog_item(&db, Uuid::new_v4(), "Northwind delivery", None);

        let hits = repo(&db).search("northwind", 20).unwrap();
        assert_eq!(hits.len(), 2);
        let kinds: HashSet<_> = hits.iter().map(|h| h.kind).collect();
        assert!(kinds.contains(&SearchEntityKind::Client));
        assert!(kinds.contains(&SearchEntityKind::CatalogItem));
    }

    #[test]
    fn limit_caps_the_result_count() {
        let db = open_memory();
        for i in 0..10 {
            insert_client(&db, Uuid::new_v4(), &format!("Acme {i}"), None);
        }
        assert_eq!(repo(&db).search("acme", 3).unwrap().len(), 3);
    }

    #[test]
    fn no_match_returns_empty() {
        let db = open_memory();
        insert_client(&db, Uuid::new_v4(), "Acme Corporation", None);
        assert!(repo(&db).search("nonexistent", 20).unwrap().is_empty());
    }
}
