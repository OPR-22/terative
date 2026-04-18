use std::sync::Arc;

use crate::application::ports::CatalogItemRepository;
use crate::application::AppError;
use crate::domain::catalog_item::{
    CatalogItem, CatalogItemError, CatalogItemId, CatalogItemKind, NewCatalogItem,
};
use crate::domain::money::Money;

pub struct CreateCatalogItem {
    repo: Arc<dyn CatalogItemRepository>,
}

impl CreateCatalogItem {
    pub fn new(repo: Arc<dyn CatalogItemRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: NewCatalogItem) -> Result<CatalogItem, AppError> {
        let item = CatalogItem::create(input)?;
        self.repo.insert(&item)?;
        Ok(item)
    }
}

pub struct UpdateCatalogItem {
    repo: Arc<dyn CatalogItemRepository>,
}

#[derive(Debug, Clone)]
pub struct UpdateCatalogItemInput {
    pub id: CatalogItemId,
    pub name: String,
    pub kind: CatalogItemKind,
    pub default_price: Money,
    pub unit: Option<String>,
    pub reference: Option<String>,
}

impl UpdateCatalogItem {
    pub fn new(repo: Arc<dyn CatalogItemRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: UpdateCatalogItemInput) -> Result<CatalogItem, AppError> {
        let mut item = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(CatalogItemError::EmptyName.into());
        }
        if input.default_price.is_negative() {
            return Err(CatalogItemError::NegativePrice.into());
        }
        item.name = name;
        item.kind = input.kind;
        item.default_price = input.default_price;
        item.unit = input.unit.and_then(normalize);
        item.reference = input.reference.and_then(normalize);
        self.repo.update(&item)?;
        Ok(item)
    }
}

pub struct ArchiveCatalogItem {
    repo: Arc<dyn CatalogItemRepository>,
}

impl ArchiveCatalogItem {
    pub fn new(repo: Arc<dyn CatalogItemRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: CatalogItemId) -> Result<(), AppError> {
        let mut item = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        item.deactivate();
        self.repo.update(&item)?;
        Ok(())
    }
}

pub struct UnarchiveCatalogItem {
    repo: Arc<dyn CatalogItemRepository>,
}

impl UnarchiveCatalogItem {
    pub fn new(repo: Arc<dyn CatalogItemRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: CatalogItemId) -> Result<(), AppError> {
        let mut item = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        item.reactivate();
        self.repo.update(&item)?;
        Ok(())
    }
}

pub struct ListCatalogItems {
    repo: Arc<dyn CatalogItemRepository>,
}

impl ListCatalogItems {
    pub fn new(repo: Arc<dyn CatalogItemRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, include_inactive: bool) -> Result<Vec<CatalogItem>, AppError> {
        Ok(self.repo.list(include_inactive)?)
    }
}

fn normalize(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::money::Currency;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    struct InMemoryRepo {
        inner: Mutex<HashMap<CatalogItemId, CatalogItem>>,
    }

    impl CatalogItemRepository for InMemoryRepo {
        fn insert(&self, s: &CatalogItem) -> Result<(), RepoError> {
            self.inner.lock().insert(s.id, s.clone());
            Ok(())
        }
        fn update(&self, s: &CatalogItem) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&s.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(s.id, s.clone());
            Ok(())
        }
        fn get(&self, id: CatalogItemId) -> Result<Option<CatalogItem>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, include_inactive: bool) -> Result<Vec<CatalogItem>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<CatalogItem> = g
                .values()
                .filter(|s| include_inactive || s.active)
                .cloned()
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(v)
        }
        fn delete(&self, id: CatalogItemId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    fn new_service(name: &str, cents: i64) -> NewCatalogItem {
        NewCatalogItem {
            name: name.into(),
            kind: CatalogItemKind::Service,
            default_price: Money::new(cents, eur()),
            unit: Some("hour".into()),
            reference: None,
        }
    }

    #[test]
    fn create_persists_entity_with_all_fields() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(NewCatalogItem {
                name: "Consulting".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(10000, eur()),
                unit: Some("hour".into()),
                reference: Some("CONS-01".into()),
            })
            .unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(s.kind, CatalogItemKind::Service);
        assert_eq!(s.unit.as_deref(), Some("hour"));
        assert_eq!(s.reference.as_deref(), Some("CONS-01"));
        assert!(s.active);
        assert_eq!(repo.inner.lock().len(), 1);

        // Confirm the stored copy matches (the repo receives the fully
        // constructed domain entity, not the input).
        let stored = repo.inner.lock().get(&s.id).cloned().unwrap();
        assert_eq!(stored, s);
    }

    #[test]
    fn create_product_kind_persists() {
        let repo = Arc::new(InMemoryRepo::default());
        let p = CreateCatalogItem::new(repo.clone())
            .execute(NewCatalogItem {
                name: "Book".into(),
                kind: CatalogItemKind::Product,
                default_price: Money::new(2500, eur()),
                unit: Some("piece".into()),
                reference: Some("SKU-042".into()),
            })
            .unwrap();
        assert_eq!(p.kind, CatalogItemKind::Product);
        let stored = repo.inner.lock().get(&p.id).cloned().unwrap();
        assert_eq!(stored.kind, CatalogItemKind::Product);
        assert_eq!(stored.reference.as_deref(), Some("SKU-042"));
    }

    #[test]
    fn update_changes_all_fields() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Consulting", 10000))
            .unwrap();
        let updated = UpdateCatalogItem::new(repo.clone())
            .execute(UpdateCatalogItemInput {
                id: s.id,
                name: "Consulting (senior)".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(20000, eur()),
                unit: Some("day".into()),
                reference: Some("SR-001".into()),
            })
            .unwrap();
        assert_eq!(updated.default_price.minor_units(), 20000);
        assert_eq!(updated.name, "Consulting (senior)");
        assert_eq!(updated.unit.as_deref(), Some("day"));
        assert_eq!(updated.reference.as_deref(), Some("SR-001"));
    }

    #[test]
    fn update_rejects_missing() {
        let repo = Arc::new(InMemoryRepo::default());
        let err = UpdateCatalogItem::new(repo)
            .execute(UpdateCatalogItemInput {
                id: CatalogItemId::new(),
                name: "X".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::zero(eur()),
                unit: None,
                reference: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn update_can_change_kind_from_service_to_product() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Online course", 5000))
            .unwrap();
        assert_eq!(s.kind, CatalogItemKind::Service);
        let updated = UpdateCatalogItem::new(repo.clone())
            .execute(UpdateCatalogItemInput {
                id: s.id,
                name: "Online course".into(),
                kind: CatalogItemKind::Product,
                default_price: s.default_price,
                unit: Some("license".into()),
                reference: Some("COURSE-01".into()),
            })
            .unwrap();
        assert_eq!(updated.kind, CatalogItemKind::Product);
        assert_eq!(updated.unit.as_deref(), Some("license"));
    }

    #[test]
    fn update_can_clear_optional_fields() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(NewCatalogItem {
                name: "Consulting".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(15000, eur()),
                unit: Some("hour".into()),
                reference: Some("CONS-01".into()),
            })
            .unwrap();
        let updated = UpdateCatalogItem::new(repo)
            .execute(UpdateCatalogItemInput {
                id: s.id,
                name: "Consulting".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(15000, eur()),
                unit: None,
                reference: None,
            })
            .unwrap();
        assert_eq!(updated.unit, None);
        assert_eq!(updated.reference, None);
    }

    #[test]
    fn update_trims_blank_optional_fields_to_none() {
        // A "  " string coming from an over-eager frontend must be normalized
        // to None rather than stored as whitespace.
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Consulting", 10000))
            .unwrap();
        let updated = UpdateCatalogItem::new(repo)
            .execute(UpdateCatalogItemInput {
                id: s.id,
                name: "Consulting".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(10000, eur()),
                unit: Some("   ".into()),
                reference: Some("".into()),
            })
            .unwrap();
        assert_eq!(updated.unit, None);
        assert_eq!(updated.reference, None);
    }

    #[test]
    fn update_rejects_negative_price() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Consulting", 10000))
            .unwrap();
        let err = UpdateCatalogItem::new(repo)
            .execute(UpdateCatalogItemInput {
                id: s.id,
                name: "Consulting".into(),
                kind: CatalogItemKind::Service,
                default_price: Money::new(-1, eur()),
                unit: None,
                reference: None,
            })
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::CatalogItem(CatalogItemError::NegativePrice)
        ));
    }

    #[test]
    fn archive_deactivates_entity() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Consulting", 0))
            .unwrap();
        ArchiveCatalogItem::new(repo.clone()).execute(s.id).unwrap();
        let stored = repo.inner.lock().get(&s.id).cloned().unwrap();
        assert!(!stored.active);
        assert_eq!(
            ListCatalogItems::new(repo.clone()).execute(false).unwrap().len(),
            0
        );
        assert_eq!(
            ListCatalogItems::new(repo).execute(true).unwrap().len(),
            1
        );
    }

    #[test]
    fn unarchive_reactivates_entity() {
        let repo = Arc::new(InMemoryRepo::default());
        let s = CreateCatalogItem::new(repo.clone())
            .execute(new_service("Consulting", 0))
            .unwrap();
        ArchiveCatalogItem::new(repo.clone()).execute(s.id).unwrap();
        UnarchiveCatalogItem::new(repo.clone()).execute(s.id).unwrap();
        let stored = repo.inner.lock().get(&s.id).cloned().unwrap();
        assert!(stored.active);
        assert_eq!(
            ListCatalogItems::new(repo).execute(false).unwrap().len(),
            1
        );
    }

    #[test]
    fn list_sorted_by_name() {
        let repo = Arc::new(InMemoryRepo::default());
        let create = CreateCatalogItem::new(repo.clone());
        create.execute(new_service("Zeta", 0)).unwrap();
        create.execute(new_service("Alpha", 0)).unwrap();
        let list = ListCatalogItems::new(repo).execute(false).unwrap();
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zeta");
    }
}
