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

/// Common interface for all FreeSynergy managers.
pub trait FsManager {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn is_healthy(&self) -> bool;
}

/// A containerized application entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: AppStatus,
}

impl Container {
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
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Installing | Self::Error(_))
    }
}

impl std::fmt::Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running         => write!(f, "Running"),
            Self::Stopped         => write!(f, "Stopped"),
            Self::Installing      => write!(f, "Installing"),
            Self::Error(msg)      => write!(f, "Error: {msg}"),
        }
    }
}

/// Manages containerized applications for the FreeSynergy ecosystem.
pub struct ContainerManager;

impl ContainerManager {
    pub fn new() -> Self {
        Self
    }

    /// Returns all installed apps and their current status.
    pub fn installed(&self) -> Vec<Container> {
        // TODO: read from Store / Inventory
        vec![]
    }

    /// Installs a container app by ID. Requires Store write permission.
    pub fn install(&self, app_id: &str) -> Result<(), ContainerError> {
        // TODO: pull from Store catalog, deploy via Podman Quadlets
        let _ = app_id;
        Ok(())
    }

    /// Removes an installed app. Requires Store write permission.
    pub fn remove(&self, app_id: &str) -> Result<(), ContainerError> {
        // TODO: stop container, remove Quadlet, update Store
        let _ = app_id;
        Ok(())
    }

    /// Starts a stopped app.
    pub fn start(&self, app_id: &str) -> Result<(), ContainerError> {
        let _ = app_id;
        Ok(())
    }

    /// Stops a running app.
    pub fn stop(&self, app_id: &str) -> Result<(), ContainerError> {
        let _ = app_id;
        Ok(())
    }
}

impl Default for ContainerManager {
    fn default() -> Self { Self::new() }
}

impl FsManager for ContainerManager {
    fn id(&self)          -> &str { "container" }
    fn name(&self)        -> &str { "Container App Manager" }
    fn is_healthy(&self)  -> bool { true }
}

#[derive(Debug)]
pub enum ContainerError {
    NotFound(String),
    AlreadyInstalled(String),
    PermissionDenied,
    StoreError(String),
    RuntimeError(String),
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "App not found: {id}"),
            Self::AlreadyInstalled(id) => write!(f, "App already installed: {id}"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::StoreError(msg) => write!(f, "Store error: {msg}"),
            Self::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
        }
    }
}

impl std::error::Error for ContainerError {}
