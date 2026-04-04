// service.rs — ServiceController trait + ServiceStatus + supporting types.
//
// Design Pattern: Command (ServiceCommand variants) + Strategy (ServiceController impl)

use crate::error::ManagerCoreError;

// ── ServiceStatus ─────────────────────────────────────────────────────────────

/// The current operational status of a managed service.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    /// Service is running normally.
    Running,
    /// Service is stopped (not started).
    Stopped,
    /// Service has failed and is not recovering.
    Failed,
    /// Service is starting up.
    Starting,
    /// Service is shutting down.
    Stopping,
    /// Status cannot be determined (e.g. not installed).
    Unknown,
}

impl ServiceStatus {
    /// FTL key for displaying the status in the UI.
    #[must_use]
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::Running => "manager-service-status-running",
            Self::Stopped => "manager-service-status-stopped",
            Self::Failed => "manager-service-status-failed",
            Self::Starting => "manager-service-status-starting",
            Self::Stopping => "manager-service-status-stopping",
            Self::Unknown => "manager-service-status-unknown",
        }
    }

    /// Returns `true` when the service is operational.
    #[must_use]
    pub fn is_running(&self) -> bool {
        *self == Self::Running
    }
}

// ── ServiceCommand ────────────────────────────────────────────────────────────

/// Commands that can be issued to a `ServiceController`.
///
/// Design Pattern: Command — each variant encapsulates a discrete operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceCommand {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl ServiceCommand {
    /// FTL key for button labels etc.
    #[must_use]
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::Start => "manager-service-cmd-start",
            Self::Stop => "manager-service-cmd-stop",
            Self::Restart => "manager-service-cmd-restart",
            Self::Enable => "manager-service-cmd-enable",
            Self::Disable => "manager-service-cmd-disable",
        }
    }
}

// ── ServiceController ─────────────────────────────────────────────────────────

/// Strategy trait — controls the lifecycle of a single service.
///
/// Two built-in implementations:
/// - [`SystemdServiceController`] — wraps `systemctl`
/// - [`ContainerServiceController`] — wraps `podman` via fs-container gRPC
///
/// Managers implement or compose these to control their services.
#[async_trait::async_trait]
pub trait ServiceController: Send + Sync {
    /// The service name (e.g. `"kanidm"` or `"pod-kanidm-pod.service"`).
    fn name(&self) -> &str;

    /// Start the service.
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn start(&self) -> Result<(), ManagerCoreError>;

    /// Stop the service.
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn stop(&self) -> Result<(), ManagerCoreError>;

    /// Restart the service (stop + start).
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn restart(&self) -> Result<(), ManagerCoreError>;

    /// Enable the service to start on boot.
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn enable(&self) -> Result<(), ManagerCoreError>;

    /// Disable the service from starting on boot.
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn disable(&self) -> Result<(), ManagerCoreError>;

    /// Query the current status.
    ///
    /// # Errors
    /// Returns `ManagerCoreError::CommandFailed` on failure.
    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError>;

    /// Execute a `ServiceCommand`.
    ///
    /// Default implementation dispatches to the individual methods.
    ///
    /// # Errors
    /// Propagates errors from the individual command methods.
    async fn execute(&self, cmd: ServiceCommand) -> Result<(), ManagerCoreError> {
        match cmd {
            ServiceCommand::Start => self.start().await,
            ServiceCommand::Stop => self.stop().await,
            ServiceCommand::Restart => self.restart().await,
            ServiceCommand::Enable => self.enable().await,
            ServiceCommand::Disable => self.disable().await,
        }
    }
}
