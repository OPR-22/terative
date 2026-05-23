use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::settings::render_placeholders;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmailTemplateId(pub Uuid);

impl EmailTemplateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EmailTemplateId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EmailTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailTemplateType {
    InitialContact,
    FollowUp,
}

impl EmailTemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InitialContact => "InitialContact",
            Self::FollowUp => "FollowUp",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "InitialContact" => Some(Self::InitialContact),
            "FollowUp" => Some(Self::FollowUp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailTemplate {
    pub id: EmailTemplateId,
    pub name: String,
    pub template_type: EmailTemplateType,
    pub subject_template: String,
    pub body_template: String,
    pub is_default: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmailTemplateError {
    #[error("email template name cannot be empty")]
    EmptyName,
    #[error("email template subject cannot be empty")]
    EmptySubject,
    #[error("email template body cannot be empty")]
    EmptyBody,
}

#[derive(Debug, Clone)]
pub struct NewEmailTemplate {
    pub name: String,
    pub template_type: EmailTemplateType,
    pub subject_template: String,
    pub body_template: String,
}

impl EmailTemplate {
    pub fn create(input: NewEmailTemplate) -> Result<Self, EmailTemplateError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(EmailTemplateError::EmptyName);
        }
        let subject = input.subject_template.trim().to_string();
        if subject.is_empty() {
            return Err(EmailTemplateError::EmptySubject);
        }
        let body = input.body_template.trim().to_string();
        if body.is_empty() {
            return Err(EmailTemplateError::EmptyBody);
        }
        Ok(Self {
            id: EmailTemplateId::new(),
            name,
            template_type: input.template_type,
            subject_template: subject,
            body_template: body,
            is_default: false,
        })
    }

    pub fn update(
        &mut self,
        name: String,
        subject_template: String,
        body_template: String,
    ) -> Result<(), EmailTemplateError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(EmailTemplateError::EmptyName);
        }
        let subject = subject_template.trim().to_string();
        if subject.is_empty() {
            return Err(EmailTemplateError::EmptySubject);
        }
        let body = body_template.trim().to_string();
        if body.is_empty() {
            return Err(EmailTemplateError::EmptyBody);
        }
        self.name = name;
        self.subject_template = subject;
        self.body_template = body;
        Ok(())
    }

    pub fn render_subject(&self, vars: &HashMap<&str, String>) -> String {
        render_placeholders(&self.subject_template, vars)
    }

    pub fn render_body(&self, vars: &HashMap<&str, String>) -> String {
        render_placeholders(&self.body_template, vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> NewEmailTemplate {
        NewEmailTemplate {
            name: "Default".into(),
            template_type: EmailTemplateType::InitialContact,
            subject_template: "Invoice {{number}}".into(),
            body_template: "Hi {{client_name}}, total: {{total}}".into(),
        }
    }

    #[test]
    fn create_valid() {
        let t = EmailTemplate::create(valid_input()).unwrap();
        assert_eq!(t.name, "Default");
        assert_eq!(t.template_type, EmailTemplateType::InitialContact);
        assert!(!t.is_default);
    }

    #[test]
    fn create_rejects_empty_name() {
        let mut input = valid_input();
        input.name = "  ".into();
        assert_eq!(
            EmailTemplate::create(input).unwrap_err(),
            EmailTemplateError::EmptyName,
        );
    }

    #[test]
    fn create_rejects_empty_subject() {
        let mut input = valid_input();
        input.subject_template = "  ".into();
        assert_eq!(
            EmailTemplate::create(input).unwrap_err(),
            EmailTemplateError::EmptySubject,
        );
    }

    #[test]
    fn create_rejects_empty_body() {
        let mut input = valid_input();
        input.body_template = "  ".into();
        assert_eq!(
            EmailTemplate::create(input).unwrap_err(),
            EmailTemplateError::EmptyBody,
        );
    }

    #[test]
    fn update_valid() {
        let mut t = EmailTemplate::create(valid_input()).unwrap();
        t.update(
            "Updated".into(),
            "New subject".into(),
            "New body".into(),
        )
        .unwrap();
        assert_eq!(t.name, "Updated");
        assert_eq!(t.subject_template, "New subject");
    }

    #[test]
    fn update_rejects_empty_name() {
        let mut t = EmailTemplate::create(valid_input()).unwrap();
        assert_eq!(
            t.update("  ".into(), "s".into(), "b".into()).unwrap_err(),
            EmailTemplateError::EmptyName,
        );
    }

    #[test]
    fn render_substitutes_placeholders() {
        let t = EmailTemplate::create(valid_input()).unwrap();
        let mut vars = HashMap::new();
        vars.insert("number", "42".into());
        vars.insert("client_name", "Acme".into());
        vars.insert("total", "100.00 EUR".into());
        assert_eq!(t.render_subject(&vars), "Invoice 42");
        assert_eq!(t.render_body(&vars), "Hi Acme, total: 100.00 EUR");
    }

    #[test]
    fn parse_template_type_roundtrip() {
        assert_eq!(
            EmailTemplateType::parse(EmailTemplateType::InitialContact.as_str()),
            Some(EmailTemplateType::InitialContact),
        );
        assert_eq!(
            EmailTemplateType::parse(EmailTemplateType::FollowUp.as_str()),
            Some(EmailTemplateType::FollowUp),
        );
        assert_eq!(EmailTemplateType::parse("Unknown"), None);
    }
}
