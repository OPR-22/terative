use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::adapters::sqlite::connection::Db;
use crate::application::ports::EmailTemplateRepository;
use crate::application::RepoError;
use crate::domain::email_template::{EmailTemplate, EmailTemplateId, EmailTemplateType};

pub struct SqliteEmailTemplateRepository {
    db: Db,
}

impl SqliteEmailTemplateRepository {
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

fn row_to_email_template(row: &Row<'_>) -> rusqlite::Result<EmailTemplate> {
    let id_str: String = row.get("id")?;
    let id = EmailTemplateId(Uuid::parse_str(&id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?);
    let type_str: String = row.get("template_type")?;
    let template_type = EmailTemplateType::parse(&type_str).unwrap_or(EmailTemplateType::InitialContact);
    Ok(EmailTemplate {
        id,
        name: row.get("name")?,
        template_type,
        subject_template: row.get("subject_template")?,
        body_template: row.get("body_template")?,
        is_default: row.get::<_, i64>("is_default")? != 0,
    })
}

const SELECT_COLS: &str = "id, name, template_type, subject_template, body_template, is_default";

impl EmailTemplateRepository for SqliteEmailTemplateRepository {
    fn insert(&self, t: &EmailTemplate) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO email_templates (id, name, template_type, subject_template, body_template, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                t.id.to_string(),
                t.name,
                t.template_type.as_str(),
                t.subject_template,
                t.body_template,
                t.is_default as i64,
            ],
        )
        .map_err(map_err)?;
        Ok(())
    }

    fn update(&self, t: &EmailTemplate) -> Result<(), RepoError> {
        let conn = self.db.lock();
        let affected = conn
            .execute(
                "UPDATE email_templates SET name = ?2, subject_template = ?3, body_template = ?4
                 WHERE id = ?1",
                params![
                    t.id.to_string(),
                    t.name,
                    t.subject_template,
                    t.body_template,
                ],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn get(&self, id: EmailTemplateId) -> Result<Option<EmailTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!("SELECT {SELECT_COLS} FROM email_templates WHERE id = ?1");
        conn.query_row(&sql, params![id.to_string()], row_to_email_template)
            .optional()
            .map_err(map_err)
    }

    fn list(&self) -> Result<Vec<EmailTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM email_templates ORDER BY template_type ASC, is_default DESC, name COLLATE NOCASE ASC"
        );
        let mut stmt = conn.prepare(&sql).map_err(map_err)?;
        let rows = stmt
            .query_map([], row_to_email_template)
            .map_err(map_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(map_err)?);
        }
        Ok(out)
    }

    fn get_default_for_type(
        &self,
        t: EmailTemplateType,
    ) -> Result<Option<EmailTemplate>, RepoError> {
        let conn = self.db.lock();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM email_templates WHERE template_type = ?1 AND is_default = 1 LIMIT 1"
        );
        conn.query_row(&sql, params![t.as_str()], row_to_email_template)
            .optional()
            .map_err(map_err)
    }

    fn set_default_for_type(
        &self,
        id: EmailTemplateId,
        t: EmailTemplateType,
    ) -> Result<(), RepoError> {
        let mut conn = self.db.lock();
        let tx = conn.transaction().map_err(map_err)?;
        tx.execute(
            "UPDATE email_templates SET is_default = 0 WHERE template_type = ?1",
            params![t.as_str()],
        )
        .map_err(map_err)?;
        let affected = tx
            .execute(
                "UPDATE email_templates SET is_default = 1 WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(map_err)?;
        if affected == 0 {
            return Err(RepoError::NotFound);
        }
        tx.commit().map_err(map_err)?;
        Ok(())
    }

    fn delete(&self, id: EmailTemplateId) -> Result<(), RepoError> {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM email_templates WHERE id = ?1",
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
    use crate::domain::email_template::NewEmailTemplate;

    fn make(name: &str, t: EmailTemplateType) -> EmailTemplate {
        EmailTemplate::create(NewEmailTemplate {
            name: name.into(),
            template_type: t,
            subject_template: "Subject".into(),
            body_template: "Body".into(),
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let db = open_memory();
        let repo = SqliteEmailTemplateRepository::new(db);
        let t = make("Initial", EmailTemplateType::InitialContact);
        repo.insert(&t).unwrap();
        let loaded = repo.get(t.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Initial");
        assert_eq!(loaded.template_type, EmailTemplateType::InitialContact);
    }

    #[test]
    fn set_default_is_exclusive_within_type() {
        let db = open_memory();
        let repo = SqliteEmailTemplateRepository::new(db);
        let a = make("A", EmailTemplateType::InitialContact);
        let b = make("B", EmailTemplateType::InitialContact);
        let c = make("C", EmailTemplateType::FollowUp);
        repo.insert(&a).unwrap();
        repo.insert(&b).unwrap();
        repo.insert(&c).unwrap();

        repo.set_default_for_type(a.id, EmailTemplateType::InitialContact)
            .unwrap();
        repo.set_default_for_type(c.id, EmailTemplateType::FollowUp)
            .unwrap();

        // Now set b as default for InitialContact — a should lose default.
        repo.set_default_for_type(b.id, EmailTemplateType::InitialContact)
            .unwrap();

        let default_ic = repo
            .get_default_for_type(EmailTemplateType::InitialContact)
            .unwrap()
            .unwrap();
        assert_eq!(default_ic.id, b.id);
        assert!(!repo.get(a.id).unwrap().unwrap().is_default);

        // FollowUp default should be unaffected.
        let default_fu = repo
            .get_default_for_type(EmailTemplateType::FollowUp)
            .unwrap()
            .unwrap();
        assert_eq!(default_fu.id, c.id);
    }

    #[test]
    fn list_returns_all_templates() {
        let db = open_memory();
        let repo = SqliteEmailTemplateRepository::new(db);
        repo.insert(&make("A", EmailTemplateType::InitialContact))
            .unwrap();
        repo.insert(&make("B", EmailTemplateType::FollowUp))
            .unwrap();
        assert_eq!(repo.list().unwrap().len(), 2);
    }

    #[test]
    fn delete_removes_template() {
        let db = open_memory();
        let repo = SqliteEmailTemplateRepository::new(db);
        let t = make("X", EmailTemplateType::InitialContact);
        repo.insert(&t).unwrap();
        repo.delete(t.id).unwrap();
        assert!(repo.get(t.id).unwrap().is_none());
    }
}
