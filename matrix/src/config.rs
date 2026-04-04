// config.rs — TuwunelConfig: output of the setup wizard.
//
// Serialized via fs-config (TOML) to /etc/freesynergy/tuwunel/config.toml
// before the container starts.

use serde::{Deserialize, Serialize};

// ── OidcConfig ────────────────────────────────────────────────────────────────

/// Kanidm OIDC integration — required for Tuwunel (IAM-first: no local accounts).
///
/// Tuwunel enforces OIDC: all Matrix accounts must be backed by Kanidm.
/// Skipping is only supported for standalone testing (offline mode).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OidcConfig {
    /// Kanidm issuer URL, e.g. `"https://idm.example.com"`.
    pub issuer_url: String,
    /// OIDC client ID registered in Kanidm (e.g. `"tuwunel"`).
    pub client_id: String,
    /// OIDC client secret.
    pub client_secret: String,
}

impl OidcConfig {
    /// Returns `true` if all fields are filled.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.issuer_url.is_empty() && !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

// ── TlsConfig ─────────────────────────────────────────────────────────────────

/// TLS configuration for the Matrix homeserver.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    /// PEM certificate path (or ACME domain for auto-cert).
    pub cert_path: String,
    /// PEM private key path (empty when using ACME).
    pub key_path: String,
    /// Use ACME (Let's Encrypt) to obtain certificates automatically.
    pub use_acme: bool,
}

impl TlsConfig {
    /// Returns `true` if TLS is configured.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.use_acme || (!self.cert_path.is_empty() && !self.key_path.is_empty())
    }
}

// ── TuwunelConfig ─────────────────────────────────────────────────────────────

/// Complete Tuwunel configuration produced by the setup wizard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TuwunelConfig {
    /// Matrix server name (e.g. `"example.org"`).
    pub server_name: String,
    /// Admin contact address (e.g. `"admin@example.org"`).
    pub admin_email: String,
    /// TLS configuration.
    pub tls: TlsConfig,
    /// Kanidm OIDC integration (required for production).
    pub oidc: OidcConfig,
    /// Whether OIDC is skipped — only allowed for offline/standalone testing.
    pub skip_oidc: bool,
    /// Enable Matrix federation with other homeservers.
    pub federation_enabled: bool,
}

impl TuwunelConfig {
    /// Returns `true` if all required fields are set.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.server_name.is_empty()
            && !self.admin_email.is_empty()
            && self.tls.is_configured()
            && (self.skip_oidc || self.oidc.is_configured())
    }
}
