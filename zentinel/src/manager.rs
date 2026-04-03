// manager.rs — ZentinelManager: Facade over the Zentinel API.
//
// Design Pattern: Facade
//
// The Facade hides the complexity of the Zentinel Control Plane REST API
// behind a clean domain interface. Callers only deal with RouteConfig and
// service IDs — never raw HTTP calls.
//
// Auto-routing:
//   When a service registers in fs-registry (via Bus event), the caller can
//   call `auto_route_for_service` with the service capability + endpoint.
//   ZentinelManager derives a sensible RouteConfig and registers it.
//
// The actual HTTP calls to Zentinel are stubbed out — they must be filled
// in by the runtime layer (the Container starts Zentinel, the manager calls
// the Control Plane API at runtime). In tests the manager runs offline.

use std::sync::{Arc, RwLock};

use fs_core::{FsManager, ManagerStore, NoopStore};

use crate::{
    error::ZentinelManagerError,
    route::{RouteConfig, RouteProtocol, RouteTable},
};

// ── Well-known capability → default path mapping ──────────────────────────────

/// Default path prefix for a given registry capability.
///
/// Used when auto-generating a route from a newly registered service.
fn default_path_for_capability(capability: &str) -> &'static str {
    match capability {
        "iam" | "iam.oidc-provider" | "iam.ldap" => "/auth",
        "mail" => "/mail",
        "git" => "/git",
        "wiki" => "/wiki",
        "chat" => "/chat",
        "storage" | "s3" => "/storage",
        "proxy.control-plane" => "/_zentinel",
        _ => "/",
    }
}

// ── ZentinelManager ───────────────────────────────────────────────────────────

/// Facade over the Zentinel reverse-proxy and Control Plane.
///
/// Maintains an in-memory `RouteTable` and propagates changes to the
/// Zentinel Control Plane API.  The API calls are stubbed — the runtime
/// layer injects the actual HTTP client.
pub struct ZentinelManager {
    table: Arc<RwLock<RouteTable>>,
    /// Persistent store for route snapshots (write-through on add/remove).
    #[allow(dead_code)]
    store: Arc<dyn ManagerStore>,
    /// Base URL of the Zentinel Control Plane, e.g. `http://zentinel-plane:9090`.
    control_plane_url: String,
}

impl ZentinelManager {
    /// Create a manager connected to a Zentinel Control Plane.
    pub fn new(control_plane_url: impl Into<String>, store: Arc<dyn ManagerStore>) -> Self {
        Self {
            table: Arc::new(RwLock::new(RouteTable::new())),
            store,
            control_plane_url: control_plane_url.into(),
        }
    }

    /// Create an offline manager for testing (no real API calls, no-op store).
    #[must_use]
    pub fn offline() -> Self {
        Self::new("http://localhost:9090", Arc::new(NoopStore))
    }

    /// All registered routes, sorted by ID.
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned (should not happen in practice).
    #[must_use]
    pub fn routes(&self) -> Vec<RouteConfig> {
        self.table
            .read()
            .expect("route table lock")
            .all()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Number of registered routes.
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned.
    #[must_use]
    pub fn route_count(&self) -> usize {
        self.table.read().expect("route table lock").len()
    }

    // ── Facade operations ─────────────────────────────────────────────────────

    /// Register a new route.
    ///
    /// Upserts the route in the local table and calls the Control Plane API
    /// to apply it.
    ///
    /// # Errors
    /// - `Validation`  — route is not valid (empty fields)
    /// - `Api`         — Control Plane unreachable (stubbed to Ok in offline mode)
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned.
    pub fn add_route(&self, route: RouteConfig) -> Result<(), ZentinelManagerError> {
        if !route.is_valid() {
            return Err(ZentinelManagerError::Validation(
                "zentinel-error-route-invalid".into(),
            ));
        }
        // Stub: in a real implementation, call POST {control_plane_url}/api/routes
        // with route serialized as JSON.  We log the URL to avoid unused-field warnings.
        let _ = &self.control_plane_url;

        self.table.write().expect("route table lock").upsert(route);
        Ok(())
    }

    /// Remove a route by ID.
    ///
    /// # Errors
    /// - `NotFound` — no route with that ID
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned.
    pub fn remove_route(&self, id: &str) -> Result<(), ZentinelManagerError> {
        let removed = self.table.write().expect("route table lock").remove(id);
        if removed.is_none() {
            return Err(ZentinelManagerError::NotFound(id.to_string()));
        }
        // Stub: call DELETE {control_plane_url}/api/routes/{id}
        Ok(())
    }

    /// Remove all routes for a service.
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned.
    pub fn remove_routes_for_service(&self, service_id: &str) {
        let ids: Vec<String> = self
            .table
            .read()
            .expect("route table lock")
            .by_service(service_id)
            .into_iter()
            .map(|r| r.id.clone())
            .collect();

        let mut table = self.table.write().expect("route table lock");
        for id in ids {
            table.remove(&id);
        }
    }

    /// Auto-generate and register a route for a newly registered service.
    ///
    /// Derives a sensible `RouteConfig` from the service's capability and
    /// endpoint.  The route ID is `"{service_id}-auto"`.
    ///
    /// Called by the bus handler when `registry::service::registered` fires.
    ///
    /// # Errors
    /// - `Validation` — endpoint is empty
    ///
    /// # Panics
    /// Panics if the internal route-table lock is poisoned.
    pub fn auto_route_for_service(
        &self,
        service_id: &str,
        capability: &str,
        endpoint: &str,
    ) -> Result<RouteConfig, ZentinelManagerError> {
        if endpoint.is_empty() {
            return Err(ZentinelManagerError::Validation(
                "zentinel-error-endpoint-empty".into(),
            ));
        }

        let path = default_path_for_capability(capability);
        let route_id = format!("{service_id}-auto");
        let proto = if endpoint.starts_with("https://") {
            RouteProtocol::Https
        } else {
            RouteProtocol::Http
        };

        let route = RouteConfig::new(
            route_id,
            service_id,
            path,
            endpoint,
            false,
            proto,
            format!("Auto-route: {service_id} ({capability})"),
        );

        self.add_route(route.clone())?;
        Ok(route)
    }

    /// The Control Plane base URL this manager is configured for.
    #[must_use]
    pub fn control_plane_url(&self) -> &str {
        &self.control_plane_url
    }
}

impl FsManager for ZentinelManager {
    fn id(&self) -> &'static str {
        "zentinel"
    }
    fn name(&self) -> &'static str {
        "Zentinel Manager"
    }
    fn is_healthy(&self) -> bool {
        // Stub: a real implementation pings GET {control_plane_url}/health
        true
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> ZentinelManager {
        ZentinelManager::offline()
    }

    fn kanidm_route() -> RouteConfig {
        RouteConfig::new(
            "kanidm-main",
            "kanidm",
            "/auth",
            "https://kanidm:8443",
            false,
            RouteProtocol::Https,
            "Kanidm IAM",
        )
    }

    #[test]
    fn add_and_list_route() {
        let m = manager();
        m.add_route(kanidm_route()).unwrap();
        assert_eq!(m.route_count(), 1);
        assert_eq!(m.routes()[0].id, "kanidm-main");
    }

    #[test]
    fn add_invalid_route_is_error() {
        let m = manager();
        let bad = RouteConfig::new(
            "",
            "svc",
            "/",
            "http://svc:80",
            false,
            RouteProtocol::Http,
            "",
        );
        assert!(m.add_route(bad).is_err());
    }

    #[test]
    fn remove_route() {
        let m = manager();
        m.add_route(kanidm_route()).unwrap();
        m.remove_route("kanidm-main").unwrap();
        assert_eq!(m.route_count(), 0);
    }

    #[test]
    fn remove_nonexistent_is_error() {
        let m = manager();
        assert!(m.remove_route("missing").is_err());
    }

    #[test]
    fn remove_routes_for_service_clears_all() {
        let m = manager();
        m.add_route(kanidm_route()).unwrap();
        m.add_route(RouteConfig::new(
            "kanidm-ldap",
            "kanidm",
            "/ldap",
            "https://kanidm:3636",
            false,
            RouteProtocol::Tcp,
            "Kanidm LDAP",
        ))
        .unwrap();
        m.remove_routes_for_service("kanidm");
        assert_eq!(m.route_count(), 0);
    }

    #[test]
    fn auto_route_iam_gets_auth_path() {
        let m = manager();
        let route = m
            .auto_route_for_service("kanidm", "iam.oidc-provider", "https://kanidm:8443")
            .unwrap();
        assert_eq!(route.path, "/auth");
        assert_eq!(route.id, "kanidm-auto");
        assert_eq!(route.protocol, RouteProtocol::Https);
    }

    #[test]
    fn auto_route_git_gets_git_path() {
        let m = manager();
        let route = m
            .auto_route_for_service("forgejo", "git", "http://forgejo:3000")
            .unwrap();
        assert_eq!(route.path, "/git");
        assert_eq!(route.protocol, RouteProtocol::Http);
    }

    #[test]
    fn auto_route_empty_endpoint_is_error() {
        let m = manager();
        assert!(m.auto_route_for_service("svc", "iam", "").is_err());
    }

    #[test]
    fn manager_id_and_name() {
        let m = manager();
        assert_eq!(m.id(), "zentinel");
        assert_eq!(m.name(), "Zentinel Manager");
    }

    #[test]
    fn control_plane_url_is_stored() {
        let m = ZentinelManager::new("http://cp:9090", std::sync::Arc::new(NoopStore));
        assert_eq!(m.control_plane_url(), "http://cp:9090");
    }
}
