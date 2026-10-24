use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod infrai_queue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatterJob {
    Intake { matter_id: String, client_email: String },
    SignedDelivery { matter_id: String, document_id: String },
    DeadlineFollowUp { matter_id: String, deadline: String },
}

impl MatterJob {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Intake { .. } => "matter-intake",
            Self::SignedDelivery { .. } => "signed-document-delivery",
            Self::DeadlineFollowUp { .. } => "deadline-follow-up",
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JobError {
    #[error("matter intake needs a client email")]
    MissingClientEmail,
    #[error("signed delivery needs a document id")]
    MissingDocument,
    #[error("deadline follow-up needs a deadline")]
    MissingDeadline,
}

pub fn should_dead_letter(job: &MatterJob) -> Result<(), JobError> {
    match job {
        MatterJob::Intake { client_email, .. } if client_email.trim().is_empty() => {
            Err(JobError::MissingClientEmail)
        }
        MatterJob::SignedDelivery { document_id, .. } if document_id.trim().is_empty() => {
            Err(JobError::MissingDocument)
        }
        MatterJob::DeadlineFollowUp { deadline, .. } if deadline.trim().is_empty() => {
            Err(JobError::MissingDeadline)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_signed_delivery_is_sent_to_dead_letter_queue() {
        let job = MatterJob::SignedDelivery {
            matter_id: "matter-42".into(),
            document_id: "".into(),
        };
        assert_eq!(should_dead_letter(&job), Err(JobError::MissingDocument));
    }
}
