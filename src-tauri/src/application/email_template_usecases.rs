use std::sync::Arc;

use crate::application::ports::EmailTemplateRepository;
use crate::application::AppError;
use crate::domain::email_template::{EmailTemplate, EmailTemplateId, NewEmailTemplate};

pub struct CreateEmailTemplate {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl CreateEmailTemplate {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, input: NewEmailTemplate) -> Result<EmailTemplate, AppError> {
        let template = EmailTemplate::create(input)?;
        self.repo.insert(&template)?;
        Ok(template)
    }
}

pub struct UpdateEmailTemplate {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl UpdateEmailTemplate {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(
        &self,
        id: EmailTemplateId,
        name: String,
        subject_template: String,
        body_template: String,
    ) -> Result<EmailTemplate, AppError> {
        let mut template = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        template.update(name, subject_template, body_template)?;
        self.repo.update(&template)?;
        Ok(template)
    }
}

pub struct DeleteEmailTemplate {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl DeleteEmailTemplate {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: EmailTemplateId) -> Result<(), AppError> {
        let template = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        if template.is_default {
            return Err(AppError::EmailTemplateIsDefault);
        }
        self.repo.delete(id)?;
        Ok(())
    }
}

pub struct SetDefaultEmailTemplate {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl SetDefaultEmailTemplate {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self, id: EmailTemplateId) -> Result<(), AppError> {
        let template = self.repo.get(id)?.ok_or(AppError::NotFound)?;
        self.repo
            .set_default_for_type(id, template.template_type)?;
        Ok(())
    }
}

pub struct ListEmailTemplates {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl ListEmailTemplates {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
    pub fn execute(&self) -> Result<Vec<EmailTemplate>, AppError> {
        Ok(self.repo.list()?)
    }
}
