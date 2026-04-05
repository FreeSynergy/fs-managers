// service_controller.rs — ForgejoServiceController + ForgejoCategoryController.
//
// Design Pattern: Adapter (ForgejoServiceController wraps ContainerServiceController)
//               + Composite (ForgejoCategoryController implements CategoryManager)

use fs_manager_core::{
    CategoryManager, ContainerServiceController, ManagerCoreError, ServiceCategory,
    ServiceController, ServiceInfo, ServiceStatus,
};

// ── ForgejoServiceController ──────────────────────────────────────────────────

/// Wraps [`ContainerServiceController`] for the Forgejo pod.
pub struct ForgejoServiceController {
    inner: ContainerServiceController,
}

impl ForgejoServiceController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ContainerServiceController::new("pod-forgejo-pod.service"),
        }
    }
}

impl Default for ForgejoServiceController {
    fn default() -> Self {
        Self::new()
    }
}

#[fs_manager_core::async_trait]
impl ServiceController for ForgejoServiceController {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn start(&self) -> Result<(), ManagerCoreError> {
        self.inner.start().await
    }

    async fn stop(&self) -> Result<(), ManagerCoreError> {
        self.inner.stop().await
    }

    async fn restart(&self) -> Result<(), ManagerCoreError> {
        self.inner.restart().await
    }

    async fn enable(&self) -> Result<(), ManagerCoreError> {
        self.inner.enable().await
    }

    async fn disable(&self) -> Result<(), ManagerCoreError> {
        self.inner.disable().await
    }

    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError> {
        self.inner.status().await
    }
}

// ── ForgejoCategoryController ─────────────────────────────────────────────────

/// Implements [`CategoryManager`] for the `Git` service category.
///
/// Currently manages only Forgejo; additional Git forges can be added here
/// as adapters following the same Strategy pattern as the Wiki manager.
pub struct ForgejoCategoryController {
    ctrl: ForgejoServiceController,
}

impl ForgejoCategoryController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ctrl: ForgejoServiceController::new(),
        }
    }
}

impl Default for ForgejoCategoryController {
    fn default() -> Self {
        Self::new()
    }
}

#[fs_manager_core::async_trait]
impl ServiceController for ForgejoCategoryController {
    fn name(&self) -> &str {
        self.ctrl.name()
    }

    async fn start(&self) -> Result<(), ManagerCoreError> {
        self.ctrl.start().await
    }

    async fn stop(&self) -> Result<(), ManagerCoreError> {
        self.ctrl.stop().await
    }

    async fn restart(&self) -> Result<(), ManagerCoreError> {
        self.ctrl.restart().await
    }

    async fn enable(&self) -> Result<(), ManagerCoreError> {
        self.ctrl.enable().await
    }

    async fn disable(&self) -> Result<(), ManagerCoreError> {
        self.ctrl.disable().await
    }

    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError> {
        self.ctrl.status().await
    }
}

#[fs_manager_core::async_trait]
impl CategoryManager for ForgejoCategoryController {
    fn category(&self) -> ServiceCategory {
        ServiceCategory::Git
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>, ManagerCoreError> {
        let status = self.ctrl.status().await.unwrap_or(ServiceStatus::Unknown);
        let installed = !matches!(status, ServiceStatus::Unknown);
        Ok(vec![ServiceInfo {
            id: "forgejo".into(),
            display_name: "Forgejo".into(),
            installed,
            is_primary: true,
            status,
            version: None,
        }])
    }

    async fn list_running(&self) -> Result<Vec<ServiceInfo>, ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|s| s.status.is_running()).collect())
    }

    async fn get_active(&self) -> Result<Option<ServiceInfo>, ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|s| s.is_primary))
    }

    async fn set_active(&self, service_id: &str) -> Result<(), ManagerCoreError> {
        if service_id == "forgejo" {
            Ok(())
        } else {
            Err(ManagerCoreError::NotInstalled(service_id.into()))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_manager_core::ServiceCategory;

    #[test]
    fn category_is_git() {
        let c = ForgejoCategoryController::new();
        assert_eq!(c.category(), ServiceCategory::Git);
    }

    #[test]
    fn name_is_pod_service() {
        let c = ForgejoCategoryController::new();
        assert_eq!(c.name(), "pod-forgejo-pod.service");
    }

    #[tokio::test]
    async fn set_active_forgejo_ok() {
        let c = ForgejoCategoryController::new();
        assert!(c.set_active("forgejo").await.is_ok());
    }

    #[tokio::test]
    async fn set_active_unknown_err() {
        let c = ForgejoCategoryController::new();
        assert!(c.set_active("gitlab").await.is_err());
    }
}
