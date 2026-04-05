// error.rs — ForgejoManagerError.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgejoManagerError {
    #[error("wizard called on wrong step")]
    WrongStep,

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("could not write config: {0}")]
    ConfigWrite(String),
}
