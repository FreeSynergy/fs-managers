// route.rs — RouteConfig + RouteTable.
//
// A RouteConfig maps one service to a Zentinel proxy route.
// RouteTable is the in-memory store of all active routes.
//
// Zentinel Control Plane receives these as JSON via its REST API:
//   POST /api/routes          → add_route
//   PUT  /api/routes/{id}     → update_route
//   DELETE /api/routes/{id}   → remove_route

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Protocol ──────────────────────────────────────────────────────────────────

/// Transport protocol for a route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RouteProtocol {
    #[default]
    Http,
    Https,
    Tcp,
}

impl RouteProtocol {
    /// FTL key for the protocol label.
    #[must_use]
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::Http => "zentinel-proto-http",
            Self::Https => "zentinel-proto-https",
            Self::Tcp => "zentinel-proto-tcp",
        }
    }
}

// ── RouteConfig ───────────────────────────────────────────────────────────────

/// A single Zentinel proxy route.
///
/// Maps an incoming path (or TCP port) to an upstream service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Unique route ID — must be unique within the Zentinel instance.
    /// Typically `"{service_id}-{port}"`, e.g. `"kanidm-8443"`.
    pub id: String,
    /// The service this route leads to (maps to a registry `service_id`).
    pub service_id: String,
    /// Path prefix Zentinel matches on (e.g. `"/auth"` or `"/"`).
    pub path: String,
    /// Upstream URL incl. port (e.g. `"http://kanidm:8443"`).
    pub upstream: String,
    /// Whether to strip the `path` prefix before forwarding.
    pub strip_path: bool,
    /// Transport protocol.
    pub protocol: RouteProtocol,
    /// Human-readable description (shown in Zentinel dashboard).
    pub description: String,
}

impl RouteConfig {
    /// Create a simple HTTP route with `strip_path = false`.
    #[must_use]
    pub fn http(
        id: impl Into<String>,
        service_id: impl Into<String>,
        path: impl Into<String>,
        upstream: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let service_id = service_id.into();
        let description = format!("{service_id} → {}", upstream.into());
        Self {
            id,
            service_id,
            path: path.into(),
            upstream: description
                .split(" → ")
                .nth(1)
                .unwrap_or_default()
                .to_string(),
            strip_path: false,
            protocol: RouteProtocol::Http,
            description,
        }
    }

    /// Returns `true` if all required fields are non-empty.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
            && !self.service_id.is_empty()
            && !self.path.is_empty()
            && !self.upstream.is_empty()
    }
}

// Convenience: build with all fields explicit.
impl RouteConfig {
    /// Full constructor.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        service_id: impl Into<String>,
        path: impl Into<String>,
        upstream: impl Into<String>,
        strip_path: bool,
        protocol: RouteProtocol,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            service_id: service_id.into(),
            path: path.into(),
            upstream: upstream.into(),
            strip_path,
            protocol,
            description: description.into(),
        }
    }
}

// ── RouteTable ────────────────────────────────────────────────────────────────

/// In-memory collection of all active Zentinel routes, keyed by route ID.
#[derive(Debug, Default)]
pub struct RouteTable {
    routes: HashMap<String, RouteConfig>,
}

impl RouteTable {
    /// Create an empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a route.
    pub fn upsert(&mut self, route: RouteConfig) {
        self.routes.insert(route.id.clone(), route);
    }

    /// Remove a route by ID. Returns `None` if not found.
    pub fn remove(&mut self, id: &str) -> Option<RouteConfig> {
        self.routes.remove(id)
    }

    /// Look up a route by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&RouteConfig> {
        self.routes.get(id)
    }

    /// All routes for a given service ID.
    #[must_use]
    pub fn by_service(&self, service_id: &str) -> Vec<&RouteConfig> {
        self.routes
            .values()
            .filter(|r| r.service_id == service_id)
            .collect()
    }

    /// All routes, sorted by ID for stable output.
    #[must_use]
    pub fn all(&self) -> Vec<&RouteConfig> {
        let mut v: Vec<&RouteConfig> = self.routes.values().collect();
        v.sort_by_key(|r| r.id.as_str());
        v
    }

    /// Number of routes in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// `true` if the table contains no routes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn kanidm_route() -> RouteConfig {
        RouteConfig::new(
            "kanidm-main",
            "kanidm",
            "/",
            "https://kanidm:8443",
            false,
            RouteProtocol::Https,
            "Kanidm IAM",
        )
    }

    #[test]
    fn upsert_and_get() {
        let mut t = RouteTable::new();
        t.upsert(kanidm_route());
        assert!(t.get("kanidm-main").is_some());
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn remove_returns_route() {
        let mut t = RouteTable::new();
        t.upsert(kanidm_route());
        let r = t.remove("kanidm-main");
        assert!(r.is_some());
        assert!(t.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut t = RouteTable::new();
        assert!(t.remove("missing").is_none());
    }

    #[test]
    fn by_service_filters_correctly() {
        let mut t = RouteTable::new();
        t.upsert(kanidm_route());
        t.upsert(RouteConfig::new(
            "forgejo-main",
            "forgejo",
            "/git",
            "http://forgejo:3000",
            false,
            RouteProtocol::Http,
            "Forgejo Git",
        ));
        assert_eq!(t.by_service("kanidm").len(), 1);
        assert_eq!(t.by_service("forgejo").len(), 1);
        assert_eq!(t.by_service("other").len(), 0);
    }

    #[test]
    fn all_returns_sorted() {
        let mut t = RouteTable::new();
        t.upsert(RouteConfig::new(
            "z-route",
            "svc",
            "/z",
            "http://z:80",
            false,
            RouteProtocol::Http,
            "",
        ));
        t.upsert(RouteConfig::new(
            "a-route",
            "svc",
            "/a",
            "http://a:80",
            false,
            RouteProtocol::Http,
            "",
        ));
        let all = t.all();
        assert_eq!(all[0].id, "a-route");
        assert_eq!(all[1].id, "z-route");
    }

    #[test]
    fn route_is_valid() {
        assert!(kanidm_route().is_valid());
    }

    #[test]
    fn invalid_route_empty_upstream() {
        let r = RouteConfig::new("id", "svc", "/", "", false, RouteProtocol::Http, "");
        assert!(!r.is_valid());
    }

    #[test]
    fn upsert_replaces_existing() {
        let mut t = RouteTable::new();
        t.upsert(kanidm_route());
        let mut updated = kanidm_route();
        updated.upstream = "https://kanidm-new:9443".to_string();
        t.upsert(updated);
        assert_eq!(t.len(), 1);
        assert_eq!(
            t.get("kanidm-main").unwrap().upstream,
            "https://kanidm-new:9443"
        );
    }
}
