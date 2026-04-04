// error.rs — WikiManagerError

#[derive(Debug, thiserror::Error)]
pub enum WikiManagerError {
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Config write failed: {0}")]
    ConfigWrite(String),
    #[error("Wizard already completed")]
    AlreadyCompleted,
    #[error("Step not allowed here")]
    WrongStep,
    #[error("Service not known: {0}")]
    UnknownService(String),
}
