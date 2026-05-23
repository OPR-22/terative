use std::collections::HashMap;

use crate::application::RepoError;
use crate::domain::tax::{TaxDefinition, TaxId};

pub trait TaxRepository: Send + Sync {
    fn insert(&self, tax: &TaxDefinition) -> Result<(), RepoError>;
    fn update(&self, tax: &TaxDefinition) -> Result<(), RepoError>;
    fn get(&self, id: TaxId) -> Result<Option<TaxDefinition>, RepoError>;
    fn list(&self, include_archived: bool) -> Result<Vec<TaxDefinition>, RepoError>;
    fn get_many(&self, ids: &[TaxId]) -> Result<Vec<TaxDefinition>, RepoError>;
    fn delete(&self, id: TaxId) -> Result<(), RepoError>;

    /// Batch fetch of tax display labels (their `name`). Missing entries
    /// mean the tax doesn't exist. Used by audit handlers to render
    /// `entity_label` without an N+1 lookup.
    fn labels_for(
        &self,
        ids: &[TaxId],
    ) -> Result<HashMap<TaxId, String>, RepoError>;
}
