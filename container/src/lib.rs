#![deny(clippy::all, clippy::pedantic, warnings)]
// FreeSynergy Container App Manager
//
// Responsibilities:
//   - List installed and available containerized apps
//   - Install, update, start, stop, remove container apps
//   - Write state changes to the Store (requires permission)
//   - Provide UI components for app management
//
// Formerly known as "Conductor". Renamed to reflect its actual role:
// managing containerized applications within the FreeSynergy ecosystem.

use std::sync::Arc;

use fs_core::{FsManager, ManagerStore, NoopStore};

/// A containerized application entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: AppStatus,
}

impl Container {
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.status == AppStatus::Running
    }
}

/// Runtime status of a container app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppStatus {
    Running,
    Stopped,
    Installing,
    Error(String),
}

impl AppStatus {
    #[must_use]
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Installing | Self::Error(_))
    }

    /// Short human-readable label for the status.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Installing => "Installing",
            Self::Error(_) => "Error",
        }
    }

    /// CSS color variable for the status badge.
    #[must_use]
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Running => "var(--fs-color-success, #22c55e)",
            Self::Stopped => "var(--fs-color-text-muted, #6b7280)",
            Self::Installing => "var(--fs-color-warning, #f59e0b)",
            Self::Error(_) => "var(--fs-color-danger, #ef4444)",
        }
    }
}

impl std::fmt::Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Manages containerized applications for the `FreeSynergy` ecosystem.
pub struct ContainerManager {
    store: Arc<dyn ManagerStore>,
}

impl ContainerManager {
    /// Create a manager backed by `store`.
    pub fn new(store: Arc<dyn ManagerStore>) -> Self {
        Self { store }
    }

    /// Create a manager with a no-op store (test / offline use).
    #[must_use]
    pub fn with_noop() -> Self {
        Self::new(Arc::new(NoopStore))
    }

    /// Returns all installed apps and their current status.
    ///
    /// Reads the serialized app list from `"container.installed"` in the store.
    #[must_use]
    pub fn installed(&self) -> Vec<Container> {
        // The store holds a newline-separated list of "id:name:version:status" entries.
        // A real implementation would use a proper DB or structured format.
        self.store
            .read_setting("container.installed")
            .map(|raw| parse_container_list(&raw))
            .unwrap_or_default()
    }

    /// Installs a container app by ID. Requires Store write permission.
    ///
    /// # Errors
    ///
    /// Returns an error if the installation fails.
    pub fn install(&self, app_id: &str) -> Result<(), ContainerError> {
        // TODO: pull from Store catalog, deploy via Podman Quadlets
        let _ = (app_id, &self.store);
        Ok(())
    }

    /// Removes an installed app. Requires Store write permission.
    ///
    /// # Errors
    ///
    /// Returns an error if removal fails.
    pub fn remove(&self, app_id: &str) -> Result<(), ContainerError> {
        // TODO: stop container, remove Quadlet, update Store
        let _ = (app_id, &self.store);
        Ok(())
    }

    /// Starts a stopped app.
    ///
    /// # Errors
    ///
    /// Returns an error if the app cannot be started.
    pub fn start(&self, app_id: &str) -> Result<(), ContainerError> {
        let _ = (app_id, &self.store);
        Ok(())
    }

    /// Stops a running app.
    ///
    /// # Errors
    ///
    /// Returns an error if the app cannot be stopped.
    pub fn stop(&self, app_id: &str) -> Result<(), ContainerError> {
        let _ = (app_id, &self.store);
        Ok(())
    }
}

impl Default for ContainerManager {
    fn default() -> Self {
        Self::with_noop()
    }
}

impl FsManager for ContainerManager {
    fn id(&self) -> &'static str {
        "container"
    }
    fn name(&self) -> &'static str {
        "Container App Manager"
    }
    fn is_healthy(&self) -> bool {
        true
    }
}

/// Error type for the Container App Manager — alias of the shared [`fs_core::ManagerError`].
pub type ContainerError = fs_core::ManagerError;

// ── Private helpers ───────────────────────────────────────────────────────────

fn parse_container_list(raw: &str) -> Vec<Container> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, ':');
            let id = parts.next()?.to_owned();
            let name = parts.next()?.to_owned();
            let version = parts.next()?.to_owned();
            let status = match parts.next().unwrap_or("stopped") {
                "running" => AppStatus::Running,
                "installing" => AppStatus::Installing,
                other if other.starts_with("error:") => AppStatus::Error(other[6..].into()),
                _ => AppStatus::Stopped,
            };
            Some(Container {
                id,
                name,
                version,
                status,
            })
        })
        .collect()
}
