// config.rs — ForgejoConfig: output of the setup wizard.
//
// Serialized via fs-config (TOML) to /etc/freesynergy/forgejo/config.toml.
//
// OIDC client secret and S3 credentials are stored as *secret references*
// (env:<VAR> or file:<path>) — never in plain text.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── OidcConfig ────────────────────────────────────────────────────────────────

/// Kanidm OIDC integration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OidcConfig {
    /// OIDC issuer URL, e.g. `https://idm.example.com`.
    pub issuer_url: String,
    /// Client ID registered in Kanidm (e.g. `"forgejo"`).
    pub client_id: String,
    /// Secret reference: `env:<VAR>` or `file:<path>`.
    pub client_secret_ref: String,
}

impl OidcConfig {
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.issuer_url.is_empty()
            && !self.client_id.is_empty()
            && !self.client_secret_ref.is_empty()
    }
}

// ── S3Config ──────────────────────────────────────────────────────────────────

/// S3-compatible storage for Forgejo repository data and LFS (optional).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 endpoint URL.
    pub endpoint: String,
    /// Bucket name.
    pub bucket: String,
    /// Secret reference for the access key ID.
    pub access_key_ref: String,
    /// Secret reference for the secret access key.
    pub secret_key_ref: String,
}

impl S3Config {
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty()
            && !self.bucket.is_empty()
            && !self.access_key_ref.is_empty()
            && !self.secret_key_ref.is_empty()
    }
}

// ── ForgejoConfig ─────────────────────────────────────────────────────────────

/// Complete Forgejo configuration produced by the setup wizard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForgejoConfig {
    /// Public domain (e.g. `git.example.com`).
    pub domain: String,
    /// SSH port (default: 22).
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
    /// Kanidm OIDC integration.
    pub oidc: OidcConfig,
    /// Optional S3 storage for LFS / repository data.
    pub s3: Option<S3Config>,
}

fn default_ssh_port() -> u16 {
    22
}

impl ForgejoConfig {
    /// `true` when all required fields are set.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.domain.is_empty() && self.oidc.is_configured()
    }
}

// ── Default config path ───────────────────────────────────────────────────────

/// Default config path: `/etc/freesynergy/forgejo/config.toml`
#[must_use]
pub fn default_config_path() -> PathBuf {
    PathBuf::from("/etc/freesynergy/forgejo/config.toml")
}

// ── ForgejoConfigStore ────────────────────────────────────────────────────────

/// Loads and saves [`ForgejoConfig`] via `fs-config` helpers.
pub struct ForgejoConfigStore {
    path: PathBuf,
}

impl ForgejoConfigStore {
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
    pub fn load(&self) -> Result<ForgejoConfig, String> {
        if !self.path.exists() {
            return Ok(ForgejoConfig::default());
        }
        fs_config::load_toml::<ForgejoConfig>(&self.path).map_err(|e| e.to_string())
    }

    /// Save config to disk (creates parent directories as needed).
    ///
    /// # Errors
    ///
    /// Returns an error if the config cannot be written.
    pub fn save(&self, config: &ForgejoConfig) -> Result<(), String> {
        fs_config::save_toml(&self.path, config).map_err(|e| e.to_string())
    }
}
