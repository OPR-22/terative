use std::sync::Arc;

use crate::application::ports::EmailLogRepository;
use crate::application::AppError;
use crate::domain::client::ClientId;
use crate::domain::email_log::EmailLog;

pub struct ListEmailLogsForClient {
    repo: Arc<dyn EmailLogRepository>,
}

impl ListEmailLogsForClient {
    pub fn new(repo: Arc<dyn EmailLogRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, client_id: ClientId) -> Result<Vec<EmailLog>, AppError> {
        Ok(self.repo.list_by_client(client_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::RepoError;
    use crate::domain::email_log::NewEmailLog;
    use crate::domain::email_template::EmailTemplateType;
    use chrono::Utc;
    use parking_lot::Mutex;

    #[derive(Default)]
    struct InMemoryEmailLogRepo(Mutex<Vec<EmailLog>>);
    impl EmailLogRepository for InMemoryEmailLogRepo {
        fn insert(&self, log: &EmailLog) -> Result<(), RepoError> {
            self.0.lock().push(log.clone());
            Ok(())
        }
        fn list_by_client(&self, client_id: ClientId) -> Result<Vec<EmailLog>, RepoError> {
            let mut v: Vec<EmailLog> = self
                .0
                .lock()
                .iter()
                .filter(|l| l.client_id == client_id)
                .cloned()
                .collect();
            v.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
            Ok(v)
        }
        fn list_by_invoices(
            &self,
            _: &[crate::domain::invoice::InvoiceId],
        ) -> Result<
            std::collections::HashMap<crate::domain::invoice::InvoiceId, Vec<EmailLog>>,
            RepoError,
        > {
            Ok(Default::default())
        }
    }

    fn log(client: ClientId, subject: &str) -> EmailLog {
        EmailLog::record(NewEmailLog {
            client_id: client,
            invoice_id: None,
            template_type: Some(EmailTemplateType::InitialContact),
            template_name: None,
            to_address: "x@y.z".into(),
            subject: subject.into(),
            sent_at: Utc::now(),
        })
        .unwrap()
    }

    #[test]
    fn returns_only_matching_client_logs() {
        let repo = Arc::new(InMemoryEmailLogRepo::default());
        let alice = ClientId::new();
        let bob = ClientId::new();
        repo.insert(&log(alice, "a1")).unwrap();
        repo.insert(&log(bob, "b1")).unwrap();
        repo.insert(&log(alice, "a2")).unwrap();

        let listed = ListEmailLogsForClient::new(repo).execute(alice).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|l| l.client_id == alice));
    }
}
