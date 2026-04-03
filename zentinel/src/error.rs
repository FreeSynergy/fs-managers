// error.rs — ZentinelManagerError

#[derive(Debug, thiserror::Error)]
pub enum ZentinelManagerError {
    #[error("Route not found: {0}")]
    NotFound(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Zentinel API error: {0}")]
    Api(String),
    #[error("Duplicate route ID: {0}")]
    Duplicate(String),
}
