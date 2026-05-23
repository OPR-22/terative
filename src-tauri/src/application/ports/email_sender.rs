use crate::application::RepoError;

/// A prepared outbound email message, independent of transport.
pub struct OutboundEmail<'a> {
    pub smtp_host: &'a str,
    pub smtp_port: u16,
    pub smtp_user: &'a str,
    pub smtp_password: &'a str,
    pub from_address: &'a str,
    pub to_address: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
    pub attachment: Option<EmailAttachment<'a>>,
}

pub struct EmailAttachment<'a> {
    pub file_name: &'a str,
    pub content_type: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("email not configured: {0}")]
    NotConfigured(String),
    #[error("smtp transport error: {0}")]
    Transport(String),
    #[error(transparent)]
    Repo(#[from] RepoError),
}

pub trait EmailSender: Send + Sync {
    fn send(&self, message: OutboundEmail<'_>) -> Result<(), EmailError>;
    fn test_connection(
        &self,
        smtp_host: &str,
        smtp_port: u16,
        smtp_user: &str,
        smtp_password: &str,
    ) -> Result<(), EmailError>;
}
