use std::sync::Arc;

use crate::application::ports::ServiceRepository;
use crate::application::AppError;
use crate::domain::money::Money;
use crate::domain::service::{NewService, Service, ServiceError, ServiceId};

pub struct CreateService {
    repo: Arc<dyn ServiceRepository>,
}

impl CreateService {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: NewService) -> Result<Service, AppError> {
        let service = Service::create(input)?;
        self.repo.insert(&service)?;
        Ok(service)
    }
}

pub struct UpdateService {
    repo: Arc<dyn ServiceRepository>,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceInput {
    pub id: ServiceId,
    pub name: String,
    pub default_price: Money,
}

impl UpdateService {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, input: UpdateServiceInput) -> Result<Service, AppError> {
        let mut s = self.repo.get(input.id)?.ok_or(AppError::NotFound)?;
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ServiceError::EmptyName.into());
        }
        if input.default_price.is_negative() {
            return Err(ServiceError::NegativePrice.into());
        }
        s.name = name;
        s.default_price = input.default_price;
        self.repo.update(&s)?;
        Ok(s)
    }
}

pub struct ArchiveService {
    repo: Arc<dyn ServiceRepository>,
}

impl ArchiveService {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: ServiceId) -> Result<(), AppError> {
        let mut service = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        service.deactivate();
        self.repo.update(&service)?;
        Ok(())
    }
}

pub struct UnarchiveService {
    repo: Arc<dyn ServiceRepository>,
}

impl UnarchiveService {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: ServiceId) -> Result<(), AppError> {
        let mut service = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        service.reactivate();
        self.repo.update(&service)?;
        Ok(())
    }
}

pub struct ListServices {
    repo: Arc<dyn ServiceRepository>,
}

impl ListServices {
    pub fn new(repo: Arc<dyn ServiceRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, include_inactive: bool) -> Result<Vec<Service>, AppError> {
        Ok(self.repo.list(include_inactive)?)
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
    struct InMemoryServiceRepo {
        inner: Mutex<HashMap<ServiceId, Service>>,
    }

    impl ServiceRepository for InMemoryServiceRepo {
        fn insert(&self, s: &Service) -> Result<(), RepoError> {
            self.inner.lock().insert(s.id, s.clone());
            Ok(())
        }
        fn update(&self, s: &Service) -> Result<(), RepoError> {
            let mut g = self.inner.lock();
            if !g.contains_key(&s.id) {
                return Err(RepoError::NotFound);
            }
            g.insert(s.id, s.clone());
            Ok(())
        }
        fn get(&self, id: ServiceId) -> Result<Option<Service>, RepoError> {
            Ok(self.inner.lock().get(&id).cloned())
        }
        fn list(&self, include_inactive: bool) -> Result<Vec<Service>, RepoError> {
            let g = self.inner.lock();
            let mut v: Vec<Service> = g
                .values()
                .filter(|s| include_inactive || s.active)
                .cloned()
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(v)
        }
        fn delete(&self, id: ServiceId) -> Result<(), RepoError> {
            self.inner.lock().remove(&id);
            Ok(())
        }
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    #[test]
    fn create_service_persists_entity() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let s = CreateService::new(repo.clone())
            .execute(NewService {
                name: "Consulting".into(),
                default_price: Money::new(10000, eur()),
            })
            .unwrap();
        assert_eq!(s.name, "Consulting");
        assert_eq!(repo.inner.lock().len(), 1);
    }

    #[test]
    fn update_service_changes_price() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let s = CreateService::new(repo.clone())
            .execute(NewService {
                name: "Consulting".into(),
                default_price: Money::new(10000, eur()),
            })
            .unwrap();
        let updated = UpdateService::new(repo.clone())
            .execute(UpdateServiceInput {
                id: s.id,
                name: "Consulting (senior)".into(),
                default_price: Money::new(20000, eur()),
            })
            .unwrap();
        assert_eq!(updated.default_price.amount_cents, 20000);
        assert_eq!(updated.name, "Consulting (senior)");
    }

    #[test]
    fn update_service_rejects_missing() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let err = UpdateService::new(repo)
            .execute(UpdateServiceInput {
                id: ServiceId::new(),
                name: "X".into(),
                default_price: Money::zero(eur()),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn update_service_rejects_negative_price() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let s = CreateService::new(repo.clone())
            .execute(NewService {
                name: "Consulting".into(),
                default_price: Money::new(10000, eur()),
            })
            .unwrap();
        let err = UpdateService::new(repo)
            .execute(UpdateServiceInput {
                id: s.id,
                name: "Consulting".into(),
                default_price: Money::new(-1, eur()),
            })
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Service(ServiceError::NegativePrice)
        ));
    }

    #[test]
    fn archive_service_deactivates_entity() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let s = CreateService::new(repo.clone())
            .execute(NewService {
                name: "Consulting".into(),
                default_price: Money::zero(eur()),
            })
            .unwrap();
        ArchiveService::new(repo.clone()).execute(s.id).unwrap();
        let stored = repo.inner.lock().get(&s.id).cloned().unwrap();
        assert!(!stored.active);
        assert_eq!(
            ListServices::new(repo.clone()).execute(false).unwrap().len(),
            0
        );
        assert_eq!(
            ListServices::new(repo).execute(true).unwrap().len(),
            1
        );
    }

    #[test]
    fn unarchive_service_reactivates_entity() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let s = CreateService::new(repo.clone())
            .execute(NewService {
                name: "Consulting".into(),
                default_price: Money::zero(eur()),
            })
            .unwrap();
        ArchiveService::new(repo.clone()).execute(s.id).unwrap();
        UnarchiveService::new(repo.clone()).execute(s.id).unwrap();
        let stored = repo.inner.lock().get(&s.id).cloned().unwrap();
        assert!(stored.active);
        assert_eq!(
            ListServices::new(repo).execute(false).unwrap().len(),
            1
        );
    }

    #[test]
    fn list_services_sorted_by_name() {
        let repo = Arc::new(InMemoryServiceRepo::default());
        let create = CreateService::new(repo.clone());
        create
            .execute(NewService {
                name: "Zeta".into(),
                default_price: Money::zero(eur()),
            })
            .unwrap();
        create
            .execute(NewService {
                name: "Alpha".into(),
                default_price: Money::zero(eur()),
            })
            .unwrap();
        let list = ListServices::new(repo).execute(false).unwrap();
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zeta");
    }
}
