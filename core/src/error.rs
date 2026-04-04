// error.rs — ManagerCoreError

/// Errors produced by the manager core library.
#[derive(Debug, thiserror::Error)]
pub enum ManagerCoreError {
    /// The service command failed.
    #[error("service command failed for '{service}': {reason}")]
    CommandFailed { service: String, reason: String },

    /// The service is not installed.
    #[error("service '{0}' is not installed")]
    NotInstalled(String),

    /// Systemd unit interaction failed.
    #[error("systemd error: {0}")]
    Systemd(String),

    /// Podman / container interaction failed.
    #[error("container error: {0}")]
    Container(String),

    /// I/O error (e.g. when reading service status file).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
