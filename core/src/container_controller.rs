// container_controller.rs — ContainerServiceController using podman CLI.
//
// Controls a Podman pod directly via `podman pod` commands.
// No direct fs-container gRPC dependency — only shell commands,
// consistent with SystemdServiceController's approach.

use crate::{error::ManagerCoreError, service::ServiceStatus};

/// Controls a Podman pod via `podman pod start/stop/restart/inspect`.
///
/// Used when the pod is NOT managed via a systemd unit (e.g. during
/// initial setup or on systems without systemd). For production deployments
/// the pod.yml is installed via `podman play kube` which generates a
/// systemd quadlet — use `SystemdServiceController` in that case.
pub struct ContainerServiceController {
    pod_name: String,
}

impl ContainerServiceController {
    /// Create a controller for the given Podman pod name (e.g. `"kanidm-pod"`).
    #[must_use]
    pub fn new(pod_name: impl Into<String>) -> Self {
        Self {
            pod_name: pod_name.into(),
        }
    }

    async fn podman(&self, args: &[&str]) -> Result<(), ManagerCoreError> {
        let output = tokio::process::Command::new("podman")
            .args(args)
            .output()
            .await
            .map_err(ManagerCoreError::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ManagerCoreError::Container(stderr))
        }
    }
}

#[async_trait::async_trait]
impl crate::service::ServiceController for ContainerServiceController {
    fn name(&self) -> &str {
        &self.pod_name
    }

    async fn start(&self) -> Result<(), ManagerCoreError> {
        self.podman(&["pod", "start", &self.pod_name]).await
    }

    async fn stop(&self) -> Result<(), ManagerCoreError> {
        self.podman(&["pod", "stop", &self.pod_name]).await
    }

    async fn restart(&self) -> Result<(), ManagerCoreError> {
        self.podman(&["pod", "restart", &self.pod_name]).await
    }

    /// Enable: runs `podman generate systemd` to create a unit file,
    /// then enables it via systemctl.
    async fn enable(&self) -> Result<(), ManagerCoreError> {
        // generate-systemd writes to stdout; we pipe it to systemd-run or
        // write the unit file manually. For now we delegate to systemctl
        // because the pod.yml installation path (podman play kube) already
        // creates the quadlet unit. This method is a no-op for podman-only
        // controller — use SystemdServiceController for full enable/disable.
        Err(ManagerCoreError::Container(
            "enable/disable requires systemd integration — use SystemdServiceController".into(),
        ))
    }

    async fn disable(&self) -> Result<(), ManagerCoreError> {
        Err(ManagerCoreError::Container(
            "enable/disable requires systemd integration — use SystemdServiceController".into(),
        ))
    }

    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError> {
        let output = tokio::process::Command::new("podman")
            .args(["pod", "inspect", "--format", "{{.State}}", &self.pod_name])
            .output()
            .await
            .map_err(ManagerCoreError::Io)?;

        if !output.status.success() {
            // Pod does not exist → not installed / unknown.
            return Ok(ServiceStatus::Unknown);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let status = match stdout.trim().to_lowercase().as_str() {
            "running" => ServiceStatus::Running,
            "stopped" | "exited" => ServiceStatus::Stopped,
            "degraded" => ServiceStatus::Failed,
            _ => ServiceStatus::Unknown,
        };
        Ok(status)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ServiceController;

    #[test]
    fn name_returns_pod_name() {
        let c = ContainerServiceController::new("my-pod");
        assert_eq!(c.name(), "my-pod");
    }
}
