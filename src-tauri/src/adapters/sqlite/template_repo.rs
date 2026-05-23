use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::TemplateRepository;
use crate::application::RepoError;
use crate::domain::template::{FontChoice, InvoiceTemplate, TemplateId, TemplateLayout};

pub struct SqliteTemplateRepository {
    db: Db,
}

impl SqliteTemplateRepository {
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

fn row_to_template(row: &Row<'_>) -> rusqlite::Result<InvoiceTemplate> {
    let id_str: String = row.get("id")?;
    let id = TemplateId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let layout_str: String = row.get("base_layout")?;
    let base_layout = TemplateLayout::parse(&layout_str).unwrap_or(TemplateLayout::Classic);
    let font_str: String = row.get("font_family")?;
    let font_family = FontChoice::parse(&font_str).unwrap_or(FontChoice::SansSerif);
    let logo_image: Option<Vec<u8>> = row.get("logo_image")?;
    Ok(InvoiceTemplate {
        id,
        name: row.get("name")?,
        base_layout,
        logo_image,
        accent_color: row.get("accent_color")?,
        font_family,
        show_seller_phone: row.get::<_, i64>("show_seller_phone")? != 0,
        show_seller_email: row.get::<_, i64>("show_seller_email")? != 0,
        show_registration_id: row.get::<_, i64>("show_registration_id")? != 0,
        show_tax_id_numbers: row.get::<_, i64>("show_tax_id_numbers")? != 0,
        show_signature: row.get::<_, i64>("show_signature")? != 0,
        show_due_date: row.get::<_, i64>("show_due_date")? != 0,
        show_total_in_words: row.get::<_, i64>("show_total_in_words")? != 0,
        header_text: row.get("header_text")?,
        footer_text: row.get("footer_text")?,
        is_default: row.get::<_, i64>("is_default")? != 0,
    })
}

const SELECT_COLS: &str = "id, name, base_layout, logo_image, accent_color, font_family, \
    show_seller_phone, show_seller_email, show_registration_id, show_tax_id_numbers, \
    show_signature, show_due_date, show_total_in_words, header_text, footer_text, is_default";

impl TemplateRepository for SqliteTemplateRepository {
    fn insert(&self, t: &InvoiceTemplate) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO invoice_templates (id, name, base_layout, logo_image, accent_color, font_family,
                show_seller_phone, show_seller_email, show_registration_id, show_tax_id_numbers,
                show_signature, show_due_date, show_total_in_words, header_text, footer_text, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                t.id.to_string(),
                t.name,
                t.base_layout.as_str(),
                t.logo_image,
                t.accent_color,
                t.font_family.as_str(),
                t.show_seller_phone as i64,
                t.show_seller_email as i64,
                t.show_registration_id as i64,
                t.show_tax_id_numbers as i64,
                t.show_signature as i64,
                t.show_due_date as i64,
                t.show_total_in_words as i64,
                t.header_text,
                t.footer_text,
                t.is_default as i64,
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, t: &InvoiceTemplate) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE invoice_templates SET
                    name = ?2, base_layout = ?3, logo_image = ?4, accent_color = ?5, font_family = ?6,
                    show_seller_phone = ?7, show_seller_email = ?8, show_registration_id = ?9,
                    show_tax_id_numbers = ?10, show_signature = ?11, show_due_date = ?12,
                    show_total_in_words = ?13, header_text = ?14, footer_text = ?15
                 WHERE id = ?1",
                params![
                    t.id.to_string(),
                    t.name,
                    t.base_layout.as_str(),
                    t.logo_image,
                    t.accent_color,
                    t.font_family.as_str(),
                    t.show_seller_phone as i64,
                    t.show_seller_email as i64,
                    t.show_registration_id as i64,
                    t.show_tax_id_numbers as i64,
                    t.show_signature as i64,
                    t.show_due_date as i64,
                    t.show_total_in_words as i64,
                    t.header_text,
                    t.footer_text,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: TemplateId) -> Result<Option<InvoiceTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_COLS} FROM invoice_templates WHERE id = ?1");
        conn.query_row(&sql, params![id.to_string()], row_to_template)
            .optional()
            .map_err(map_err)
    }

    fn list(&self) -> Result<Vec<InvoiceTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM invoice_templates ORDER BY is_default DESC, name COLLATE NOCASE ASC"
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt.query_map([], row_to_template).map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn get_default(&self) -> Result<Option<InvoiceTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_COLS} FROM invoice_templates WHERE is_default = 1 LIMIT 1");
        conn.query_row(&sql, [], row_to_template)
            .optional()
            .map_err(map_err)
    }

    fn set_default(&self, id: TemplateId) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        tx.execute("UPDATE invoice_templates SET is_default = 0", [])
            .map_err(map_err)?;
        let affected = tx
            .execute(
                "UPDATE invoice_templates SET is_default = 1 WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn is_used_by_invoice(&self, id: TemplateId) -> Result<bool, RepoError> {
        let conn = self.db.lock();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM invoices WHERE template_id = ?1",
                params![id.to_string()],
                |r| r.get(0),
            )
            .map_err(map_err)?;
        Ok(count > 0)
    }

    fn delete(&self, id: TemplateId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM invoice_templates WHERE id = ?1",
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
    use crate::domain::template::NewInvoiceTemplate;

    fn make(name: &str) -> InvoiceTemplate {
        InvoiceTemplate::create(NewInvoiceTemplate {
            name: name.into(),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteTemplateRepository::new(db);
        let t = make("Classic Blue");
        repo.insert(&t).unwrap();
        let loaded = repo.get(t.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Classic Blue");
        assert_eq!(loaded.base_layout, TemplateLayout::Classic);
    }

    #[test]
    fn set_default_is_exclusive() {
        let db = open_memory();
        let repo = SqliteTemplateRepository::new(db);
        let a = make("A");
        let b = make("B");
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        repo.set_default(a.id).unwrap();
        repo.set_default(b.id).unwrap();
        assert_eq!(repo.get_default().unwrap().unwrap().id, b.id);
        assert!(!repo.get(a.id).unwrap().unwrap().is_default);
    }
}
