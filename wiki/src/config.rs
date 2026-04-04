// config.rs — WikiConfig: output of the setup wizard.
//
// Serialized via fs-config (TOML) to /etc/freesynergy/wiki/config.toml
// before the container starts.
//
// OIDC client secret and S3 credentials are stored as *secret references*
// (env:<VAR> or file:<path>) — never in plain text.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── WikiPlatform ──────────────────────────────────────────────────────────────

/// Which wiki platform is deployed on this node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WikiPlatform {
    /// Outline (default recommendation).
    #[default]
    Outline,
    /// Wiki.js (alternative implementation).
    WikiJs,
}

impl WikiPlatform {
    /// Stable service identifier used in pod names and registry.
    #[must_use]
    pub fn service_id(&self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::WikiJs => "wikijs",
        }
    }

    /// Human-readable display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Outline => "Outline",
            Self::WikiJs => "Wiki.js",
        }
    }
}

// ── OidcConfig ────────────────────────────────────────────────────────────────

/// OIDC integration with Kanidm.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OidcConfig {
    /// OIDC issuer URL, e.g. `https://idm.example.com`.
    pub issuer_url: String,
    /// OIDC client ID registered in Kanidm (e.g. `"outline"`).
    pub client_id: String,
    /// Secret reference for the client secret: `env:<VAR>` or `file:<path>`.
    pub client_secret_ref: String,
}

impl OidcConfig {
    /// Returns `true` if all required fields are set.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.issuer_url.is_empty()
            && !self.client_id.is_empty()
            && !self.client_secret_ref.is_empty()
    }
}

// ── S3Config ──────────────────────────────────────────────────────────────────

/// S3-compatible storage for attachments / assets (optional).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 endpoint URL, e.g. `https://s3.example.com`.
    pub endpoint: String,
    /// Bucket name.
    pub bucket: String,
    /// Secret reference for the access key ID: `env:<VAR>` or `file:<path>`.
    pub access_key_ref: String,
    /// Secret reference for the secret access key: `env:<VAR>` or `file:<path>`.
    pub secret_key_ref: String,
}

impl S3Config {
    /// Returns `true` if all required fields are set.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty()
            && !self.bucket.is_empty()
            && !self.access_key_ref.is_empty()
            && !self.secret_key_ref.is_empty()
    }
}

// ── WikiConfig ────────────────────────────────────────────────────────────────

/// Complete wiki configuration produced by the setup wizard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WikiConfig {
    /// Which wiki platform to deploy.
    pub platform: WikiPlatform,
    /// Public domain for the wiki (e.g. `wiki.example.com`).
    pub domain: String,
    /// OIDC integration (Kanidm).
    pub oidc: OidcConfig,
    /// Optional S3 storage for attachments.
    pub s3: Option<S3Config>,
}

impl WikiConfig {
    /// Returns `true` if all required fields are set.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.domain.is_empty() && self.oidc.is_configured()
    }
}

// ── Default config path ───────────────────────────────────────────────────────

/// Default config path: `/etc/freesynergy/wiki/config.toml`
#[must_use]
pub fn default_config_path() -> PathBuf {
    PathBuf::from("/etc/freesynergy/wiki/config.toml")
}

// ── WikiConfigStore ───────────────────────────────────────────────────────────

/// Loads and saves [`WikiConfig`] via `fs-config` standalone helpers.
pub struct WikiConfigStore {
    path: PathBuf,
}

impl WikiConfigStore {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Load config from disk; returns default if file does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load(&self) -> Result<WikiConfig, String> {
        if !self.path.exists() {
            return Ok(WikiConfig::default());
        }
        fs_config::load_toml::<WikiConfig>(&self.path).map_err(|e| e.to_string())
    }

    /// Save config to disk (creates parent directories as needed).
    ///
    /// # Errors
    ///
    /// Returns an error if the config cannot be written.
    pub fn save(&self, config: &WikiConfig) -> Result<(), String> {
        fs_config::save_toml(&self.path, config).map_err(|e| e.to_string())
    }
}
