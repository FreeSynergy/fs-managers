// error.rs — AuthManagerError

#[derive(Debug, thiserror::Error)]
pub enum AuthManagerError {
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Config write failed: {0}")]
    ConfigWrite(String),
    #[error("Wizard already completed")]
    AlreadyCompleted,
}
