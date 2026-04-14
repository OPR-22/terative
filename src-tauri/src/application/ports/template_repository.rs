use crate::application::RepoError;
use crate::domain::template::{InvoiceTemplate, TemplateId};

pub trait TemplateRepository: Send + Sync {
    fn insert(&self, template: &InvoiceTemplate) -> Result<(), RepoError>;
    fn update(&self, template: &InvoiceTemplate) -> Result<(), RepoError>;
    fn get(&self, id: TemplateId) -> Result<Option<InvoiceTemplate>, RepoError>;
    fn list(&self) -> Result<Vec<InvoiceTemplate>, RepoError>;
    fn get_default(&self) -> Result<Option<InvoiceTemplate>, RepoError>;
    fn set_default(&self, id: TemplateId) -> Result<(), RepoError>;
    fn is_used_by_invoice(&self, id: TemplateId) -> Result<bool, RepoError>;
    fn delete(&self, id: TemplateId) -> Result<(), RepoError>;
}
