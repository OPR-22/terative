use std::collections::HashMap;

use crate::application::RepoError;
use crate::domain::catalog_item::{CatalogItem, CatalogItemId};

pub trait CatalogItemRepository: Send + Sync {
    fn insert(&self, item: &CatalogItem) -> Result<(), RepoError>;
    fn update(&self, item: &CatalogItem) -> Result<(), RepoError>;
    fn get(&self, id: CatalogItemId) -> Result<Option<CatalogItem>, RepoError>;
    fn list(&self, include_archived: bool) -> Result<Vec<CatalogItem>, RepoError>;
    fn delete(&self, id: CatalogItemId) -> Result<(), RepoError>;

    /// Batch fetch of catalog item display labels (their `name`). Missing
    /// entries mean the item doesn't exist. Used by audit handlers to render
    /// `entity_label` without an N+1 lookup.
    fn labels_for(
        &self,
        ids: &[CatalogItemId],
    ) -> Result<HashMap<CatalogItemId, String>, RepoError>;
}
