// error.rs — MailManagerError

#[derive(Debug, thiserror::Error)]
pub enum MailManagerError {
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Config write failed: {0}")]
    ConfigWrite(String),
    #[error("Wizard already completed")]
    AlreadyCompleted,
    #[error("API error: {0}")]
    Api(String),
}
