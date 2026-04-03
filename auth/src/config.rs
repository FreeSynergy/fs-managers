// config.rs — KanidmConfig: output of the setup wizard.
//
// This struct is the single source of truth for what the wizard collected.
// A real implementation would serialize this via fs-config (TOML) and write it
// to /etc/freesynergy/kanidm/config.toml before starting the container.

/// A registered OIDC client (service that authenticates against Kanidm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcClient {
    /// Logical name, e.g. `"forgejo"` or `"outline"`.
    pub id: String,
    /// Human-readable display name shown in the Kanidm UI consent screen.
    pub display_name: String,
    /// Redirect URI the service accepts OIDC callbacks on.
    pub redirect_uri: String,
}

/// Complete Kanidm configuration produced by the setup wizard.
#[derive(Debug, Clone, Default)]
pub struct KanidmConfig {
    /// Primary domain for this Kanidm instance, e.g. `"idm.example.com"`.
    pub domain: String,
    /// Initial admin account username (default: `"admin"`).
    pub admin_username: String,
    /// Initial admin account password (never stored after wizard completes).
    pub admin_password: String,
    /// OIDC clients to register on first start.
    pub oidc_clients: Vec<OidcClient>,
}

impl KanidmConfig {
    /// Returns `true` if all required fields are non-empty.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.domain.is_empty()
            && !self.admin_username.is_empty()
            && !self.admin_password.is_empty()
    }
}
