use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::application::ports::{EmailError, EmailSender, OutboundEmail};

pub struct LettreEmailSender;

impl LettreEmailSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LettreEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailSender for LettreEmailSender {
    fn send(&self, m: OutboundEmail<'_>) -> Result<(), EmailError> {
        let from = m
            .from_address
            .parse()
            .map_err(|e| EmailError::NotConfigured(format!("from: {e}")))?;
        let to = m
            .to_address
            .parse()
            .map_err(|e| EmailError::NotConfigured(format!("to: {e}")))?;

        let builder = Message::builder()
            .from(from)
            .to(to)
            .subject(m.subject);

        let text = SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body(m.body.to_string());

        let message = if let Some(att) = m.attachment {
            let content_type = att
                .content_type
                .parse::<ContentType>()
                .unwrap_or(ContentType::parse("application/octet-stream").unwrap());
            let attachment =
                Attachment::new(att.file_name.to_string()).body(att.bytes.to_vec(), content_type);
            builder
                .multipart(MultiPart::mixed().singlepart(text).singlepart(attachment))
                .map_err(|e| EmailError::Transport(format!("build: {e}")))?
        } else {
            builder
                .singlepart(text)
                .map_err(|e| EmailError::Transport(format!("build: {e}")))?
        };

        let transport = build_transport(m.smtp_host, m.smtp_port, m.smtp_user, m.smtp_password)?;
        transport
            .send(&message)
            .map_err(|e| EmailError::Transport(e.to_string()))?;
        Ok(())
    }

    fn test_connection(
        &self,
        host: &str,
        port: u16,
        user: &str,
        password: &str,
    ) -> Result<(), EmailError> {
        let transport = build_transport(host, port, user, password)?;
        transport
            .test_connection()
            .map_err(|e| EmailError::Transport(e.to_string()))?;
        Ok(())
    }
}

fn build_transport(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
) -> Result<SmtpTransport, EmailError> {
    // STARTTLS on 587 is the common default; implicit TLS on 465.
    let builder = if port == 465 {
        SmtpTransport::relay(host)
            .map_err(|e| EmailError::Transport(format!("relay: {e}")))?
    } else {
        SmtpTransport::starttls_relay(host)
            .map_err(|e| EmailError::Transport(format!("starttls: {e}")))?
    };
    let transport = builder
        .port(port)
        .credentials(Credentials::new(user.to_string(), password.to_string()))
        .build();
    Ok(transport)
}
