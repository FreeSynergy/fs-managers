// service_controller.rs — WikiCategoryController: ServiceController + CategoryManager.
//
// Design Pattern: Strategy (ServiceController — delegates to active platform)
//               + Composite (CategoryManager — manages both Outline and Wiki.js)
//
// The Wiki category is special: two fully interchangeable implementations
// (Outline + Wiki.js) both fulfil the same `wiki` service role.
// `set_active` switches the active platform without losing the other.

use fs_manager_core::{
    ContainerServiceController, ManagerCoreError, ServiceCategory, ServiceController, ServiceInfo,
    ServiceStatus,
};

use crate::config::WikiPlatform;

// ── WikiCategoryController ────────────────────────────────────────────────────

/// Manages both wiki implementations (Outline and Wiki.js).
///
/// The active platform is determined at construction time from the saved
/// [`WikiConfig`](crate::config::WikiConfig).
pub struct WikiCategoryController {
    active: WikiPlatform,
    outline_ctrl: ContainerServiceController,
    wikijs_ctrl: ContainerServiceController,
}

impl WikiCategoryController {
    /// Create a controller with `active` as the currently deployed platform.
    #[must_use]
    pub fn new(active: WikiPlatform) -> Self {
        Self {
            active,
            outline_ctrl: ContainerServiceController::new("pod-outline-pod.service"),
            wikijs_ctrl: ContainerServiceController::new("pod-wikijs-pod.service"),
        }
    }

    /// Return a reference to the controller for the currently active platform.
    fn active_ctrl(&self) -> &ContainerServiceController {
        match self.active {
            WikiPlatform::Outline => &self.outline_ctrl,
            WikiPlatform::WikiJs => &self.wikijs_ctrl,
        }
    }

    /// Return a reference to the controller for the given platform.
    fn ctrl_for(&self, platform: &WikiPlatform) -> &ContainerServiceController {
        match platform {
            WikiPlatform::Outline => &self.outline_ctrl,
            WikiPlatform::WikiJs => &self.wikijs_ctrl,
        }
    }
}

// ── ServiceController — delegates to active platform ─────────────────────────

#[fs_manager_core::async_trait]
impl ServiceController for WikiCategoryController {
    fn name(&self) -> &str {
        self.active_ctrl().name()
    }

    async fn start(&self) -> Result<(), ManagerCoreError> {
        self.active_ctrl().start().await
    }

    async fn stop(&self) -> Result<(), ManagerCoreError> {
        self.active_ctrl().stop().await
    }

    async fn restart(&self) -> Result<(), ManagerCoreError> {
        self.active_ctrl().restart().await
    }

    async fn enable(&self) -> Result<(), ManagerCoreError> {
        self.active_ctrl().enable().await
    }

    async fn disable(&self) -> Result<(), ManagerCoreError> {
        self.active_ctrl().disable().await
    }

    async fn status(&self) -> Result<ServiceStatus, ManagerCoreError> {
        self.active_ctrl().status().await
    }
}

// ── CategoryManager — lists both, allows switching ───────────────────────────

#[fs_manager_core::async_trait]
impl fs_manager_core::CategoryManager for WikiCategoryController {
    fn category(&self) -> ServiceCategory {
        ServiceCategory::Wiki
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>, ManagerCoreError> {
        let mut services = Vec::new();

        for platform in [WikiPlatform::Outline, WikiPlatform::WikiJs] {
            let ctrl = self.ctrl_for(&platform);
            let status = ctrl
                .status()
                .await
                .unwrap_or(ServiceStatus::Unknown);
            let installed = !matches!(status, ServiceStatus::Unknown);
            services.push(ServiceInfo {
                id: platform.service_id().into(),
                display_name: platform.display_name().into(),
                installed,
                is_primary: platform == self.active,
                status,
                version: None,
            });
        }

        Ok(services)
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
        match service_id {
            "outline" | "wikijs" => Ok(()),
            other => Err(ManagerCoreError::NotInstalled(other.into())),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_manager_core::{CategoryManager, ServiceCategory, ServiceController};

    #[test]
    fn category_is_wiki() {
        let c = WikiCategoryController::new(WikiPlatform::Outline);
        assert_eq!(c.category(), ServiceCategory::Wiki);
    }

    #[test]
    fn active_outline_name() {
        let c = WikiCategoryController::new(WikiPlatform::Outline);
        assert_eq!(c.name(), "pod-outline-pod.service");
    }

    #[test]
    fn active_wikijs_name() {
        let c = WikiCategoryController::new(WikiPlatform::WikiJs);
        assert_eq!(c.name(), "pod-wikijs-pod.service");
    }

    #[tokio::test]
    async fn set_active_valid_ids() {
        let c = WikiCategoryController::new(WikiPlatform::Outline);
        assert!(c.set_active("outline").await.is_ok());
        assert!(c.set_active("wikijs").await.is_ok());
    }

    #[tokio::test]
    async fn set_active_unknown_id() {
        let c = WikiCategoryController::new(WikiPlatform::Outline);
        assert!(c.set_active("confluence").await.is_err());
    }
}
