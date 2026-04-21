use crate::application::RepoError;
use crate::domain::email_template::{EmailTemplate, EmailTemplateId, EmailTemplateType};

pub trait EmailTemplateRepository: Send + Sync {
    fn insert(&self, template: &EmailTemplate) -> Result<(), RepoError>;
    fn update(&self, template: &EmailTemplate) -> Result<(), RepoError>;
    fn get(&self, id: EmailTemplateId) -> Result<Option<EmailTemplate>, RepoError>;
    fn list(&self) -> Result<Vec<EmailTemplate>, RepoError>;
    fn get_default_for_type(
        &self,
        t: EmailTemplateType,
    ) -> Result<Option<EmailTemplate>, RepoError>;
    fn set_default_for_type(
        &self,
        id: EmailTemplateId,
        t: EmailTemplateType,
    ) -> Result<(), RepoError>;
    fn delete(&self, id: EmailTemplateId) -> Result<(), RepoError>;
}
