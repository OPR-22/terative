use crate::application::RepoError;
use crate::domain::tax::{TaxDefinition, TaxId};

pub trait TaxRepository: Send + Sync {
    fn insert(&self, tax: &TaxDefinition) -> Result<(), RepoError>;
    fn update(&self, tax: &TaxDefinition) -> Result<(), RepoError>;
    fn get(&self, id: TaxId) -> Result<Option<TaxDefinition>, RepoError>;
    fn list(&self, include_inactive: bool) -> Result<Vec<TaxDefinition>, RepoError>;
    fn get_many(&self, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError>;
    fn delete(&self, id: TaxId) -> Result<(), RepoError>;
}
