use std::sync::Arc;

use rust_decimal::Decimal;

use crate::application::ports::TaxRepository;
use crate::application::AppError;
#[cfg(test)] use crate::application::ErrorCode;
use crate::domain::tax::{NewTaxDefinition, TaxDefinition, TaxId};

#[derive(Clone)]
pub struct CreateTax {
    repo: Arc<dyn TaxRepository>,
}

impl CreateTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: NewTaxDefinition) -> Result<TaxDefinition, AppError> {
        let tax = TaxDefinition::create(input)?;
        self.repo.insert(&tax)?;
        Ok(tax)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateTaxInput {
    pub id: TaxId,
    pub name: String,
    pub percentage: Decimal,
    pub tax_id_number: Option<String>,
}

pub struct UpdateTax {
    repo: Arc<dyn TaxRepository>,
}

impl UpdateTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: UpdateTaxInput) -> Result<TaxDefinition, AppError> {
        let mut tax = self.repo.get(input.id)?.ok_or(AppError::resource_not_found())?;
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(crate::domain::tax::TaxError::EmptyName.into());
        }
        if input.percentage.is_sign_negative() {
            return Err(crate::domain::tax::TaxError::NegativePercentage.into());
        }
        tax.name = name;
        tax.percentage = input.percentage;
        tax.tax_id_number = input.tax_id_number.and_then(|s| {
            let t = s.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        });
        self.repo.update(&tax)?;
        Ok(tax)
    }
}

pub struct ArchiveTax {
    repo: Arc<dyn TaxRepository>,
}

impl ArchiveTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: TaxId) -> Result<(), AppError> {
        let mut tax = self.repo.get(id)?.ok_or(AppError::resource_not_found())?;
        tax.archive(chrono::Utc::now());
        self.repo.update(&tax)?;
        Ok(())
    }
}

pub struct UnarchiveTax {
    repo: Arc<dyn TaxRepository>,
}

impl UnarchiveTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: TaxId) -> Result<(), AppError> {
        let mut tax = self.repo.get(id)?.ok_or(AppError::resource_not_found())?;
        tax.unarchive();
        self.repo.update(&tax)?;
        Ok(())
    }
}

pub struct ListTaxes {
    repo: Arc<dyn TaxRepository>,
}

impl ListTaxes {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, include_archived: bool) -> Result<Vec<TaxDefinition>, AppError> {
        Ok(self.repo.list(include_archived)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use parking_lot::Mutex;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryTaxRepo {
        inner: Mutex<HashMap<TaxId, TaxDefinition>>,
    }

    impl TaxRepository for InMemoryTaxRepo {
        fn insert(&self, t: &TaxDefinition) -> Result<(), RepoError> {
            self.inner.lock().insert(t.id, t.clone());
            Ok(())
        }
        fn update(&self, t: &TaxDefinition) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&t.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(t.id, t.clone());
            Ok(())
        }
        fn get(&self, id: TaxId) -> Result<Option<TaxDefinition>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, include_archived: bool) -> Result<Vec<TaxDefinition>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<TaxDefinition> = g
                .values()
                .filter(|t| include_archived || !t.is_archived())
                .cloned()
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(v)
        }
        fn get_many(&self, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError> {
            let g = self.inner.lock();
            Ok(ids.iter().filter_map(|id| g.get(id).cloned()).collect())
        }
        fn delete(&self, id: TaxId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    fn repo() -> Arc<InMemoryTaxRepo> {
        Arc::new(InMemoryTaxRepo::default())
    }

    #[test]
    fn create_tax_persists_entity() {
        let r = repo();
        let tax = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap();
        assert_eq!(tax.name, "TVA");
        assert_eq!(r.inner.lock().len(), 1);
    }

    #[test]
    fn update_tax_changes_percentage() {
        let r = repo();
        let tax = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap();
        let updated = UpdateTax::new(r.clone())
            .execute(UpdateTaxInput {
                id: tax.id,
                name: "TVA Reduced".into(),
                percentage: dec!(6),
                tax_id_number: Some("BE123".into()),
            })
            .unwrap();
        assert_eq!(updated.name, "TVA Reduced");
        assert_eq!(updated.percentage, dec!(6));
        assert_eq!(updated.tax_id_number.as_deref(), Some("BE123"));
    }

    #[test]
    fn update_tax_rejects_missing_id() {
        let r = repo();
        let err = UpdateTax::new(r)
            .execute(UpdateTaxInput {
                id: TaxId::new(),
                name: "X".into(),
                percentage: dec!(1),
                tax_id_number: None,
            })
            .unwrap_err();
        assert!(err.is(ErrorCode::ResourceNotFound));
    }

    #[test]
    fn update_tax_rejects_empty_name() {
        let r = repo();
        let tax = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap();
        let err = UpdateTax::new(r)
            .execute(UpdateTaxInput {
                id: tax.id,
                name: "  ".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap_err();
        assert!(err.is(ErrorCode::TaxEmptyName));
    }

    #[test]
    fn archive_tax_soft_deletes() {
        let r = repo();
        let tax = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap();
        ArchiveTax::new(r.clone()).execute(tax.id).unwrap();
        let stored = r.inner.lock().get(&tax.id).cloned().unwrap();
        assert!(stored.is_archived());
    }

    #[test]
    fn unarchive_tax_reactivates() {
        let r = repo();
        let tax = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: None,
            })
            .unwrap();
        ArchiveTax::new(r.clone()).execute(tax.id).unwrap();
        UnarchiveTax::new(r.clone()).execute(tax.id).unwrap();
        let stored = r.inner.lock().get(&tax.id).cloned().unwrap();
        assert!(!stored.is_archived());
    }

    #[test]
    fn list_excludes_inactive_by_default() {
        let r = repo();
        let uc = CreateTax::new(r.clone());
        uc.execute(NewTaxDefinition {
            name: "A".into(),
            percentage: dec!(10),
            tax_id_number: None,
        })
        .unwrap();
        let b = uc
            .execute(NewTaxDefinition {
                name: "B".into(),
                percentage: dec!(20),
                tax_id_number: None,
            })
            .unwrap();
        ArchiveTax::new(r.clone()).execute(b.id).unwrap();
        assert_eq!(ListTaxes::new(r.clone()).execute(false).unwrap().len(), 1);
        assert_eq!(ListTaxes::new(r).execute(true).unwrap().len(), 2);
    }
}
