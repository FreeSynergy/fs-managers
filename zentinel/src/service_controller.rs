// service_controller.rs — ZentinelProxyController: ServiceController + CategoryManager.
//
// Design Pattern: Strategy (ServiceController) + Composite (CategoryManager)

use fs_manager_core::ServiceController as _;

// ── ZentinelProxyController ───────────────────────────────────────────────────

/// Service controller and category manager for the Zentinel reverse proxy.
pub struct ZentinelProxyController {
    controller: fs_manager_core::SystemdServiceController,
}

impl ZentinelProxyController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            controller: fs_manager_core::SystemdServiceController::new("pod-zentinel-pod.service"),
        }
    }
}

impl Default for ZentinelProxyController {
    fn default() -> Self {
        Self::new()
    }
}

#[fs_manager_core::async_trait]
impl fs_manager_core::ServiceController for ZentinelProxyController {
    fn name(&self) -> &str {
        self.controller.name()
    }
    async fn start(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.start().await
    }
    async fn stop(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.stop().await
    }
    async fn restart(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.restart().await
    }
    async fn enable(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.enable().await
    }
    async fn disable(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.disable().await
    }
    async fn status(
        &self,
    ) -> Result<fs_manager_core::ServiceStatus, fs_manager_core::ManagerCoreError> {
        self.controller.status().await
    }
}

#[fs_manager_core::async_trait]
impl fs_manager_core::CategoryManager for ZentinelProxyController {
    fn category(&self) -> fs_manager_core::ServiceCategory {
        fs_manager_core::ServiceCategory::Proxy
    }

    async fn list_all(
        &self,
    ) -> Result<Vec<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let zentinel_status = self
            .controller
            .status()
            .await
            .unwrap_or(fs_manager_core::ServiceStatus::Unknown);
        let zentinel_installed =
            !matches!(zentinel_status, fs_manager_core::ServiceStatus::Unknown);

        Ok(vec![fs_manager_core::ServiceInfo {
            id: "zentinel".into(),
            display_name: "Zentinel".into(),
            installed: zentinel_installed,
            is_primary: true,
            status: zentinel_status,
            version: None,
        }])
    }

    async fn list_running(
        &self,
    ) -> Result<Vec<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|s| s.status.is_running()).collect())
    }

    async fn get_active(
        &self,
    ) -> Result<Option<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|s| s.is_primary))
    }

    async fn set_active(&self, service_id: &str) -> Result<(), fs_manager_core::ManagerCoreError> {
        if service_id == "zentinel" {
            Ok(())
        } else {
            Err(fs_manager_core::ManagerCoreError::NotInstalled(
                service_id.into(),
            ))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_manager_core::{CategoryManager, ServiceCategory, ServiceController};

    #[test]
    fn controller_name() {
        let c = ZentinelProxyController::new();
        assert_eq!(c.name(), "pod-zentinel-pod.service");
    }

    #[test]
    fn category_is_proxy() {
        let c = ZentinelProxyController::new();
        assert_eq!(c.category(), ServiceCategory::Proxy);
    }
}
