use crate::adapters::sqlite::connection::Db;
use crate::application::ports::InvoiceNumberGenerator;
use crate::application::RepoError;
use crate::domain::invoice::InvoiceNumber;

pub struct SqliteInvoiceNumberGenerator {
    db: Db,
}

impl SqliteInvoiceNumberGenerator {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl InvoiceNumberGenerator for SqliteInvoiceNumberGenerator {
    fn next(&self) -> Result<InvoiceNumber, RepoError> {
        let mut conn = self.db.lock();
        let tx = conn
            .transaction()
            .map_err(|e| RepoError::Storage(e.to_string()))?;
        let current: i64 = tx
            .query_row(
                "SELECT next_number FROM invoice_number_seq WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| RepoError::Storage(e.to_string()))?;
        tx.execute(
            "UPDATE invoice_number_seq SET next_number = ?1 WHERE id = 1",
            [current + 1],
        )
        .map_err(|e| RepoError::Storage(e.to_string()))?;
        tx.commit()
            .map_err(|e| RepoError::Storage(e.to_string()))?;
        Ok(InvoiceNumber(current as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::open_memory;

    #[test]
    fn next_increments() {
        let db = open_memory();
        let gen = SqliteInvoiceNumberGenerator::new(db);
        assert_eq!(gen.next().unwrap(), InvoiceNumber(1));
        assert_eq!(gen.next().unwrap(), InvoiceNumber(2));
        assert_eq!(gen.next().unwrap(), InvoiceNumber(3));
    }
}
