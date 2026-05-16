use std::sync::Arc;

use rust_decimal::Decimal;

use crate::application::ports::{CommitEvents, EventBus, NoopEventBus, TaxRepository};
use crate::application::AppError;
#[cfg(test)] use crate::application::ErrorCode;
use crate::domain::aggregate_root::AggregateRoot;
use crate::domain::events::tax_events::TaxUpdated;
use crate::domain::tax::{NewTaxDefinition, TaxDefinition, TaxId};

#[derive(Clone)]
pub struct CreateTax {
    repo: Arc<dyn TaxRepository>,
    events: Arc<dyn EventBus>,
}

impl CreateTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventBus),
        }
    }
    /// Inject the real event bus. Production wiring (`OrgServices::new`) calls
    /// this; tests that don't assert on events keep the no-op default.
    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
    }
    pub fn execute(&self, input: NewTaxDefinition) -> Result<TaxDefinition, AppError> {
        let mut tax = TaxDefinition::create(input)?;
        self.repo.insert(&tax)?;
        tax.commit(self.events.as_ref());
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
    events: Arc<dyn EventBus>,
}

impl UpdateTax {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventBus),
        }
    }
    pub fn with_events(mut self, events: Arc<dyn EventBus>) -> Self {
        self.events = events;
        self
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
        // Snapshot the prior state before mutation so the audit row can show
        // exactly what changed. `Clone` is cheap on a tax (no Vec fields).
        let before = tax.clone();
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
        // No domain `update` method — the use case records the event itself.
        let changes = tax.diff_against(&before);
        tax.apply(TaxUpdated {
            id: tax.id,
            changes,
            at: chrono::Utc::now(),
        });
        tax.commit(self.events.as_ref());
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
        fn labels_for(
            &self,
            ids: &[TaxId],
        ) -> Result<HashMap<TaxId, String>, RepoError> {
            let g = self.inner.lock();
            Ok(ids
                .iter()
                .filter_map(|id| g.get(id).map(|t| (*id, t.name.clone())))
                .collect())
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

    // === Audit diff ===

    /// Captures `TaxUpdated` payloads via downcast — the shared
    /// `CollectingEventBus` only records names, but we need `changes` here.
    #[derive(Default)]
    struct CapturingTaxUpdatedBus {
        updates: Mutex<Vec<crate::domain::events::tax_events::TaxUpdated>>,
    }
    impl crate::application::ports::EventBus for CapturingTaxUpdatedBus {
        fn dispatch(&self, event: &dyn crate::domain::events::DomainEvent) {
            if let Some(u) = event
                .as_any()
                .downcast_ref::<crate::domain::events::tax_events::TaxUpdated>()
            {
                self.updates.lock().push(u.clone());
            }
        }
    }

    #[test]
    fn update_tax_publishes_event_with_field_diff() {
        use crate::domain::field_change::FieldChange;

        let r = repo();
        let created = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: Some("BE0123".into()),
            })
            .unwrap();

        let bus = Arc::new(CapturingTaxUpdatedBus::default());
        UpdateTax::new(r)
            .with_events(bus.clone())
            .execute(UpdateTaxInput {
                id: created.id,
                name: "VAT".into(),         // changed
                percentage: dec!(21),       // unchanged
                tax_id_number: None,        // changed (Some → None)
            })
            .unwrap();

        let updates = bus.updates.lock();
        assert_eq!(updates.len(), 1);
        let evt = &updates[0];
        // Two fields changed; `percentage` is omitted because it equals before.
        assert_eq!(evt.changes.len(), 2);
        let field_names: Vec<&str> = evt.changes.iter().map(FieldChange::field).collect();
        assert!(field_names.contains(&"name"));
        assert!(field_names.contains(&"tax_id_number"));
        assert!(!field_names.contains(&"percentage"));
    }

    #[test]
    fn update_tax_publishes_empty_changes_when_nothing_actually_changed() {
        let r = repo();
        let created = CreateTax::new(r.clone())
            .execute(NewTaxDefinition {
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: Some("BE0123".into()),
            })
            .unwrap();
        let bus = Arc::new(CapturingTaxUpdatedBus::default());

        // Same input as the prior state — diff should be empty.
        UpdateTax::new(r)
            .with_events(bus.clone())
            .execute(UpdateTaxInput {
                id: created.id,
                name: "TVA".into(),
                percentage: dec!(21),
                tax_id_number: Some("BE0123".into()),
            })
            .unwrap();

        let updates = bus.updates.lock();
        assert_eq!(updates.len(), 1, "the event always fires, even on no-op");
        assert!(updates[0].changes.is_empty());
    }
}
