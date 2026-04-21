use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::email_template::{EmailTemplate, EmailTemplateType, NewEmailTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum EmailTemplateTypeDto {
    InitialContact,
    FollowUp,
}

impl From<EmailTemplateType> for EmailTemplateTypeDto {
    fn from(t: EmailTemplateType) -> Self {
        match t {
            EmailTemplateType::InitialContact => Self::InitialContact,
            EmailTemplateType::FollowUp => Self::FollowUp,
        }
    }
}

impl From<EmailTemplateTypeDto> for EmailTemplateType {
    fn from(dto: EmailTemplateTypeDto) -> Self {
        match dto {
            EmailTemplateTypeDto::InitialContact => Self::InitialContact,
            EmailTemplateTypeDto::FollowUp => Self::FollowUp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct EmailTemplateDto {
    pub id: Uuid,
    pub name: String,
    pub template_type: EmailTemplateTypeDto,
    pub subject_template: String,
    pub body_template: String,
    pub is_default: bool,
}

impl From<&EmailTemplate> for EmailTemplateDto {
    fn from(t: &EmailTemplate) -> Self {
        Self {
            id: t.id.0,
            name: t.name.clone(),
            template_type: t.template_type.into(),
            subject_template: t.subject_template.clone(),
            body_template: t.body_template.clone(),
            is_default: t.is_default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewEmailTemplateDto {
    pub name: String,
    pub template_type: EmailTemplateTypeDto,
    pub subject_template: String,
    pub body_template: String,
}

impl From<NewEmailTemplateDto> for NewEmailTemplate {
    fn from(dto: NewEmailTemplateDto) -> Self {
        NewEmailTemplate {
            name: dto.name,
            template_type: dto.template_type.into(),
            subject_template: dto.subject_template,
            body_template: dto.body_template,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateEmailTemplateDto {
    pub id: Uuid,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
}
