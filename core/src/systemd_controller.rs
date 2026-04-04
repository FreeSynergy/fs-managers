// systemd_controller.rs — SystemdServiceController.
//
// Wraps `systemctl` via tokio::process::Command.
// No direct systemd-rs dependency — only shell commands.

use crate::{error::ManagerCoreError, service::ServiceStatus};

/// Controls a systemd service via `systemctl`.
pub struct SystemdServiceController {
    unit_name: String,
}

impl SystemdServiceController {
    /// Create a controller for the given systemd unit name.
    ///
    /// The `unit_name` should include the `.service` suffix
    /// (e.g. `"pod-kanidm-pod.service"`).
    #[must_use]
    pub fn new(unit_name: impl Into<String>) -> Self {
        Self {
            unit_name: unit_name.into(),
        }
    }

    async fn systemctl(&self, args: &[&str]) -> Result<(), ManagerCoreError> {
        let output = tokio::process::Command::new("systemctl")
            .args(args)
            .output()
            .await
            .map_err(ManagerCoreError::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ManagerCoreError::Systemd(stderr))
        }
    }
}

#[async_trait::async_trait]
impl crate::service::ServiceController for SystemdServiceController {
    fn name(&self) -> &str {
        &self.unit_name
    }

    async fn start(&self) -> Result<(), ManagerCoreError> {
        self.systemctl(&["start", &self.unit_name]).await
    }

    async fn stop(&self) -> Result<(), ManagerCoreError> {
        self.systemctl(&["stop", &self.unit_name]).await
    }

    async fn restart(&self) -> Result<(), ManagerCoreError> {
        self.systemctl(&["restart", &self.unit_name]).await
    }

    async fn enable(&self) -> Result<(), ManagerCoreError> {
        self.systemctl(&["enable", &self.unit_name]).await
    }

    async fn disable(&self) -> Result<(), ManagerCoreError> {
        self.systemctl(&["disable", &self.unit_name]).await
    }

    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError> {
        let output = tokio::process::Command::new("systemctl")
            .args(["is-active", &self.unit_name])
            .output()
            .await
            .map_err(ManagerCoreError::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let status = match stdout.trim() {
            "active" => ServiceStatus::Running,
            "activating" => ServiceStatus::Starting,
            "deactivating" => ServiceStatus::Stopping,
            "failed" => ServiceStatus::Failed,
            "inactive" => ServiceStatus::Stopped,
            _ => ServiceStatus::Unknown,
        };
        Ok(status)
    }
}
