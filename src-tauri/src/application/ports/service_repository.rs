use crate::application::RepoError;
use crate::domain::service::{Service, ServiceId};

pub trait ServiceRepository: Send + Sync {
    fn insert(&self, service: &Service) -> Result<(), RepoError>;
    fn update(&self, service: &Service) -> Result<(), RepoError>;
    fn get(&self, id: ServiceId) -> Result<Option<Service>, RepoError>;
    fn list(&self, include_inactive: bool) -> Result<Vec<Service>, RepoError>;
    fn delete(&self, id: ServiceId) -> Result<(), RepoError>;
}
