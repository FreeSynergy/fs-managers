// adapter.rs — ForgejoAdapter: GitProvider trait implementation.
//
// Design Pattern: Adapter
//   Translates the generic GitProvider interface to Forgejo-specific
//   behavior.  The caller receives a &dyn GitProvider without knowing
//   the underlying implementation.

// ── GitProvider ───────────────────────────────────────────────────────────────

/// Common interface for Git forge adapters.
///
/// Provides platform-specific metadata and defaults used by the wizard
/// and service controller without making any network calls.
pub trait GitProvider: Send + Sync {
    /// Stable service identifier (matches the Store package ID).
    fn provider_id(&self) -> &'static str;

    /// Human-readable display name.
    fn display_name(&self) -> &'static str;

    /// HTTP path used for readiness / health checks.
    fn health_path(&self) -> &'static str;

    /// Default TCP port for the HTTP API.
    fn default_http_port(&self) -> u16;

    /// Default TCP port for SSH access.
    fn default_ssh_port(&self) -> u16 {
        22
    }

    /// Suggested default OIDC client ID for this platform.
    fn default_oidc_client_id(&self) -> &'static str;

    /// Service capability registered in `fs-registry`.
    fn capability(&self) -> &'static str {
        "git-hosting"
    }
}

// ── ForgejoAdapter ────────────────────────────────────────────────────────────

/// [`GitProvider`] implementation for Forgejo.
pub struct ForgejoAdapter;

impl GitProvider for ForgejoAdapter {
    fn provider_id(&self) -> &'static str {
        "forgejo"
    }

    fn display_name(&self) -> &'static str {
        "Forgejo"
    }

    fn health_path(&self) -> &'static str {
        "/-/health"
    }

    fn default_http_port(&self) -> u16 {
        3000
    }

    fn default_oidc_client_id(&self) -> &'static str {
        "forgejo"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_is_forgejo() {
        assert_eq!(ForgejoAdapter.provider_id(), "forgejo");
    }

    #[test]
    fn health_path_is_set() {
        assert!(!ForgejoAdapter.health_path().is_empty());
    }

    #[test]
    fn default_http_port_is_3000() {
        assert_eq!(ForgejoAdapter.default_http_port(), 3000);
    }

    #[test]
    fn default_ssh_port_is_22() {
        assert_eq!(ForgejoAdapter.default_ssh_port(), 22);
    }

    #[test]
    fn default_oidc_client_id_is_forgejo() {
        assert_eq!(ForgejoAdapter.default_oidc_client_id(), "forgejo");
    }

    #[test]
    fn capability_is_git_hosting() {
        assert_eq!(ForgejoAdapter.capability(), "git-hosting");
    }
}
