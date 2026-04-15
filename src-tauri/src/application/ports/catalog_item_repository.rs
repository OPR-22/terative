use crate::application::RepoError;
use crate::domain::catalog_item::{CatalogItem, CatalogItemId};

pub trait CatalogItemRepository: Send + Sync {
    fn insert(&self, item: &CatalogItem) -> Result<(), RepoError>;
    fn update(&self, item: &CatalogItem) -> Result<(), RepoError>;
    fn get(&self, id: CatalogItemId) -> Result<Option<CatalogItem>, RepoError>;
    fn list(&self, include_inactive: bool) -> Result<Vec<CatalogItem>, RepoError>;
    fn delete(&self, id: CatalogItemId) -> Result<(), RepoError>;
}
