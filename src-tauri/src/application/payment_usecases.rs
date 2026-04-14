use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::application::ports::{ListPaymentsQuery, PaymentRepository};
use crate::application::AppError;
use crate::domain::money::Money;
use crate::domain::payment::{
    NewPayment, NewPaymentAllocation, Payment, PaymentId, PaymentMethod,
};

pub struct RecordPayment {
    repo: Arc<dyn PaymentRepository>,
}

impl RecordPayment {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: NewPayment) -> Result<Payment, AppError> {
        let payment = Payment::create(input, Utc::now())?;
        self.repo.insert(&payment)?;
        Ok(payment)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentInput {
    pub id: PaymentId,
    pub date: NaiveDate,
    pub amount: Money,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub allocations: Vec<NewPaymentAllocation>,
    pub notes: Option<String>,
}

pub struct UpdatePayment {
    repo: Arc<dyn PaymentRepository>,
}

impl UpdatePayment {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: UpdatePaymentInput) -> Result<Payment, AppError> {
        let mut payment = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        payment.replace_fields(
            input.date,
            input.amount,
            input.method,
            input.reference,
            input.allocations,
            input.notes,
        )?;
        self.repo.update(&payment)?;
        Ok(payment)
    }
}

pub struct DeletePayment {
    repo: Arc<dyn PaymentRepository>,
}

impl DeletePayment {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: PaymentId) -> Result<(), AppError> {
        if self.repo.get(id)?.is_none() {
            return Err(AppError::NotFound);
        }
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct ListPayments {
    repo: Arc<dyn PaymentRepository>,
}

impl ListPayments {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, AppError> {
        Ok(self.repo.list(query)?)
    }
}

pub struct GetPayment {
    repo: Arc<dyn PaymentRepository>,
}

impl GetPayment {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: PaymentId) -> Result<Payment, AppError> {
        self.repo.get(id)?.ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::client::ClientId;
    use crate::domain::invoice::InvoiceId;
    use crate::domain::money::Currency;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryPaymentRepo {
        inner: Mutex<HashMap<PaymentId, Payment>>,
    }

    impl PaymentRepository for InMemoryPaymentRepo {
        fn insert(&self, p: &Payment) -> Result<(), RepoError> {
            self.inner.lock().insert(p.id, p.clone());
            Ok(())
        }
        fn update(&self, p: &Payment) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&p.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(p.id, p.clone());
            Ok(())
        }
        fn get(&self, id: PaymentId) -> Result<Option<Payment>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, query: ListPaymentsQuery) -> Result<Vec<Payment>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Payment> = g
                .values()
                .filter(|p| query.client_id.map(|c| c == p.client_id).unwrap_or(true))
                .cloned()
                .collect();
            v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(v)
        }
        fn delete(&self, id: PaymentId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
        fn allocated_for_invoice(&self, id: InvoiceId) -> Result<Money, RepoError> {
            let g = self.inner.lock();
            let sum: i64 = g
                .values()
                .flat_map(|p| p.allocations.iter())
                .filter(|a| a.invoice_id == id)
                .map(|a| a.amount.amount_cents)
                .sum();
            Ok(Money::new(sum, Currency::new("EUR").unwrap()))
        }
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn new_input(amount: i64, allocations: Vec<NewPaymentAllocation>) -> NewPayment {
        NewPayment {
            client_id: ClientId::new(),
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            amount: Money::new(amount, eur()),
            method: PaymentMethod::BankTransfer,
            reference: None,
            allocations,
            notes: None,
        }
    }

    #[test]
    fn record_payment_persists_entity() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let payment = RecordPayment::new(repo.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        assert_eq!(payment.amount.amount_cents, 1000);
        assert_eq!(repo.inner.lock().len(), 1);
    }

    #[test]
    fn record_payment_rejects_negative_amount() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let err = RecordPayment::new(repo)
            .execute(new_input(-1, vec![]))
            .unwrap_err();
        assert!(matches!(err, AppError::Payment(_)));
    }

    #[test]
    fn update_payment_rejects_missing_id() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let err = UpdatePayment::new(repo)
            .execute(UpdatePaymentInput {
                id: PaymentId::new(),
                date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
                amount: Money::new(500, eur()),
                method: PaymentMethod::Cash,
                reference: None,
                allocations: vec![],
                notes: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn update_payment_replaces_fields() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let payment = RecordPayment::new(repo.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        let updated = UpdatePayment::new(repo)
            .execute(UpdatePaymentInput {
                id: payment.id,
                date: payment.date,
                amount: Money::new(1500, eur()),
                method: PaymentMethod::Cash,
                reference: Some("REF".into()),
                allocations: vec![],
                notes: None,
            })
            .unwrap();
        assert_eq!(updated.amount.amount_cents, 1500);
        assert_eq!(updated.method, PaymentMethod::Cash);
        assert_eq!(updated.reference.as_deref(), Some("REF"));
    }

    #[test]
    fn delete_payment_removes_entity() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let payment = RecordPayment::new(repo.clone())
            .execute(new_input(1000, vec![]))
            .unwrap();
        DeletePayment::new(repo.clone()).execute(payment.id).unwrap();
        assert!(repo.inner.lock().is_empty());
    }

    #[test]
    fn delete_payment_rejects_missing_id() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let err = DeletePayment::new(repo)
            .execute(PaymentId::new())
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn list_payments_filters_by_client() {
        let repo = Arc::new(InMemoryPaymentRepo::default());
        let record = RecordPayment::new(repo.clone());
        let mut a = new_input(1000, vec![]);
        let client_a = a.client_id;
        record.execute(a.clone()).unwrap();
        a = new_input(500, vec![]);
        record.execute(a).unwrap();
        let filtered = ListPayments::new(repo)
            .execute(ListPaymentsQuery {
                client_id: Some(client_a),
                search: None,
            })
            .unwrap();
        assert_eq!(filtered.len(), 1);
    }
}
