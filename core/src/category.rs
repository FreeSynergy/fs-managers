// category.rs — CategoryManager trait + ServiceInfo + ServiceCategory.
//
// Design Pattern: Composite — a CategoryManager manages all services of
// one category (e.g. all IAM providers, all mail servers).

use crate::{error::ManagerCoreError, service::ServiceStatus};

// ── ServiceCategory ───────────────────────────────────────────────────────────

/// The functional category of a service.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCategory {
    /// Identity and Access Management (Kanidm, Keycloak, …)
    Iam,
    /// Mail server (Stalwart, Postfix, …)
    Mail,
    /// Messenger / Matrix (Tuwunel, …)
    Messenger,
    /// Reverse proxy (Zentinel, Nginx, …)
    Proxy,
    /// Git forge (Forgejo, Gitea, …)
    Git,
    /// Wiki / documentation (Outline, Wiki.js, …)
    Wiki,
    /// Other / catch-all.
    Other(String),
}

impl ServiceCategory {
    /// FTL key for the category label.
    #[must_use]
    pub fn i18n_key(&self) -> String {
        match self {
            Self::Iam => "manager-category-iam".into(),
            Self::Mail => "manager-category-mail".into(),
            Self::Messenger => "manager-category-messenger".into(),
            Self::Proxy => "manager-category-proxy".into(),
            Self::Git => "manager-category-git".into(),
            Self::Wiki => "manager-category-wiki".into(),
            Self::Other(name) => format!("manager-category-other-{name}"),
        }
    }
}

// ── ServiceInfo ───────────────────────────────────────────────────────────────

/// Metadata about a single service known to a `CategoryManager`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    /// Stable identifier (e.g. `"kanidm"`).
    pub id: String,
    /// Human-readable name (e.g. `"Kanidm"`).
    pub display_name: String,
    /// Whether this service is currently installed on the system.
    pub installed: bool,
    /// Whether this service is the active primary for the category.
    pub is_primary: bool,
    /// Current operational status.
    pub status: ServiceStatus,
    /// Version string if installed (e.g. `"1.4.2"`).
    pub version: Option<String>,
}

// ── CategoryManager ───────────────────────────────────────────────────────────

/// Composite trait — manages all services of a single category.
///
/// A single manager can list all known services in its category
/// (installed + available in the Store), know which is active, and
/// support role-switching between them.
#[async_trait::async_trait]
pub trait CategoryManager: Send + Sync {
    /// The category this manager is responsible for.
    fn category(&self) -> ServiceCategory;

    /// List all services in the category (installed + available in Store).
    ///
    /// # Errors
    /// Returns an error when the underlying service registry cannot be queried.
    async fn list_all(&self) -> Result<Vec<ServiceInfo>, ManagerCoreError>;

    /// List only services that are currently running.
    ///
    /// # Errors
    /// Returns an error when status queries fail.
    async fn list_running(&self) -> Result<Vec<ServiceInfo>, ManagerCoreError>;

    /// Return the current primary service for this category.
    ///
    /// The primary is the service that answers the category's service role
    /// (e.g. the IAM provider that all OIDC clients point to).
    ///
    /// Returns `None` when no primary is configured.
    ///
    /// # Errors
    /// Returns an error when the registry cannot be queried.
    async fn get_active(&self) -> Result<Option<ServiceInfo>, ManagerCoreError>;

    /// Switch the primary to the service identified by `service_id`.
    ///
    /// Updates the `fs-registry` capability entry to point to the new
    /// primary service.
    ///
    /// # Errors
    /// - `ManagerCoreError::NotInstalled` — target service not installed
    /// - `ManagerCoreError::CommandFailed` — registry update failed
    async fn set_active(&self, service_id: &str) -> Result<(), ManagerCoreError>;

    /// Check whether a newer version of `service_id` is available in the Store.
    ///
    /// Returns `Some(version_string)` when an update exists, `None` otherwise.
    /// The default implementation always returns `None` (no Store connection).
    /// Managers with Store access override this to query the catalog.
    ///
    /// # Errors
    /// Returns an error when the Store cannot be reached.
    async fn update_available(
        &self,
        _service_id: &str,
    ) -> Result<Option<String>, ManagerCoreError> {
        Ok(None)
    }
}
