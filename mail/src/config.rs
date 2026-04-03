// config.rs — StalwartConfig: output of the setup wizard.
//
// Serialized via fs-config (TOML) to /etc/freesynergy/stalwart/config.toml
// before the container starts.

/// TLS certificate configuration for the mail server.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TlsConfig {
    /// PEM certificate path (or ACME domain for auto-cert).
    pub cert_path: String,
    /// PEM private key path (empty when using ACME).
    pub key_path: String,
    /// Use ACME (Let's Encrypt) to obtain certificates automatically.
    pub use_acme: bool,
}

impl TlsConfig {
    /// Returns `true` if TLS is configured (either manual cert or ACME).
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.use_acme || (!self.cert_path.is_empty() && !self.key_path.is_empty())
    }
}

/// OIDC integration with Kanidm (or another provider).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OidcConfig {
    /// OIDC issuer URL, e.g. `https://idm.example.com`.
    pub issuer_url: String,
    /// OIDC client ID registered in Kanidm.
    pub client_id: String,
    /// OIDC client secret.
    pub client_secret: String,
}

impl OidcConfig {
    /// Returns `true` if all OIDC fields are set.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.issuer_url.is_empty() && !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

/// Complete Stalwart configuration produced by the setup wizard.
#[derive(Debug, Clone, Default)]
pub struct StalwartConfig {
    /// Primary mail domain (e.g. `"mail.example.com"`).
    pub domain: String,
    /// Admin email address (also used as postmaster).
    pub admin_email: String,
    /// TLS certificate configuration.
    pub tls: TlsConfig,
    /// OIDC integration (Kanidm).
    pub oidc: OidcConfig,
    /// Whether to skip OIDC integration (standalone / offline setup).
    pub skip_oidc: bool,
}

impl StalwartConfig {
    /// Returns `true` if all required fields are set.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.domain.is_empty()
            && !self.admin_email.is_empty()
            && self.tls.is_configured()
            && (self.skip_oidc || self.oidc.is_configured())
    }
}
