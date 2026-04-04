// wizard.rs — TuwunelSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by the UI (view.rs) or the CLI.
//
// Steps:
//   ServerName   → admin enters the Matrix server name + contact address
//   TlsCerts     → choose ACME or manual TLS cert
//   OidcIntegration → configure Kanidm OIDC (REQUIRED — no local accounts)
//   Federation   → enable/disable Matrix federation
//   Confirm      → review + confirm before writing config
//   Done         → produces TuwunelConfig
//
// IAM integration note:
//   Tuwunel delegates authentication to Kanidm via OIDC.
//   All Matrix user accounts are backed by Kanidm identities.
//   The `skip_oidc` flag is only available for offline/standalone testing.
//   In production the wizard enforces OIDC configuration.

use crate::{
    config::{OidcConfig, TlsConfig, TuwunelConfig},
    error::MatrixManagerError,
};

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the Tuwunel setup wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Enter the Matrix server name and admin contact.
    ServerName,
    /// Configure TLS (ACME or manual certificate paths).
    TlsCerts,
    /// Configure Kanidm OIDC integration (required for production).
    OidcIntegration,
    /// Enable or disable Matrix federation.
    Federation,
    /// Review all inputs before applying.
    Confirm,
    /// Wizard complete — `TuwunelConfig` is available.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::ServerName => "matrix-wizard-step-server-title",
            Self::TlsCerts => "matrix-wizard-step-tls-title",
            Self::OidcIntegration => "matrix-wizard-step-oidc-title",
            Self::Federation => "matrix-wizard-step-federation-title",
            Self::Confirm => "matrix-wizard-step-confirm-title",
            Self::Done => "matrix-wizard-step-done-title",
        }
    }

    /// FTL key for the step description.
    #[must_use]
    pub fn description_key(&self) -> &'static str {
        match self {
            Self::ServerName => "matrix-wizard-step-server-desc",
            Self::TlsCerts => "matrix-wizard-step-tls-desc",
            Self::OidcIntegration => "matrix-wizard-step-oidc-desc",
            Self::Federation => "matrix-wizard-step-federation-desc",
            Self::Confirm => "matrix-wizard-step-confirm-desc",
            Self::Done => "matrix-wizard-step-done-desc",
        }
    }
}

// ── WizardOutcome ─────────────────────────────────────────────────────────────

/// Result of a completed wizard run.
#[derive(Debug, Clone)]
pub struct WizardOutcome {
    /// The complete, validated Tuwunel configuration.
    pub config: TuwunelConfig,
}

// ── TuwunelSetupWizard ────────────────────────────────────────────────────────

/// State machine that guides an admin through initial Tuwunel configuration.
///
/// # IAM
///
/// Kanidm OIDC is **required** for production. `skip_oidc` is only intended
/// for offline/standalone testing and emits a warning in the UI.
///
/// # Usage
///
/// ```
/// use fs_manager_matrix::wizard::{TuwunelSetupWizard, WizardStep};
///
/// let mut w = TuwunelSetupWizard::new();
/// assert_eq!(w.step(), &WizardStep::ServerName);
///
/// w.set_server_name("example.org", "admin@example.org").unwrap();
/// w.advance().unwrap();
/// assert_eq!(w.step(), &WizardStep::TlsCerts);
/// ```
#[derive(Debug)]
pub struct TuwunelSetupWizard {
    step: WizardStep,
    config: TuwunelConfig,
    outcome: Option<WizardOutcome>,
}

impl TuwunelSetupWizard {
    /// Create a new wizard at the `ServerName` step.
    #[must_use]
    pub fn new() -> Self {
        Self {
            step: WizardStep::ServerName,
            config: TuwunelConfig::default(),
            outcome: None,
        }
    }

    /// Current wizard step.
    #[must_use]
    pub fn step(&self) -> &WizardStep {
        &self.step
    }

    /// Read-only access to the configuration being built.
    #[must_use]
    pub fn config(&self) -> &TuwunelConfig {
        &self.config
    }

    /// Outcome (only `Some` after `Done`).
    #[must_use]
    pub fn outcome(&self) -> Option<&WizardOutcome> {
        self.outcome.as_ref()
    }

    // ── Step setters ──────────────────────────────────────────────────────────

    /// Set the Matrix server name and admin contact.
    ///
    /// Valid at: `ServerName`.
    pub fn set_server_name(
        &mut self,
        server_name: &str,
        admin_email: &str,
    ) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::ServerName {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-wrong-step").to_string(),
            ));
        }
        let server_name = server_name.trim();
        let admin_email = admin_email.trim();
        if server_name.is_empty() || admin_email.is_empty() {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-server-required").to_string(),
            ));
        }
        if server_name.contains(' ') {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-server-spaces").to_string(),
            ));
        }
        if !admin_email.contains('@') {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-admin-email-invalid").to_string(),
            ));
        }
        server_name.clone_into(&mut self.config.server_name);
        admin_email.clone_into(&mut self.config.admin_email);
        Ok(())
    }

    /// Configure ACME TLS.
    ///
    /// Valid at: `TlsCerts`.
    pub fn set_tls_acme(&mut self) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::TlsCerts {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-wrong-step").to_string(),
            ));
        }
        self.config.tls = TlsConfig {
            use_acme: true,
            cert_path: String::new(),
            key_path: String::new(),
        };
        Ok(())
    }

    /// Configure manual TLS with certificate and key paths.
    ///
    /// Valid at: `TlsCerts`.
    pub fn set_tls_manual(
        &mut self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::TlsCerts {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-wrong-step").to_string(),
            ));
        }
        let cert_path = cert_path.trim();
        let key_path = key_path.trim();
        if cert_path.is_empty() || key_path.is_empty() {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-tls-paths-empty").to_string(),
            ));
        }
        self.config.tls = TlsConfig {
            use_acme: false,
            cert_path: cert_path.to_owned(),
            key_path: key_path.to_owned(),
        };
        Ok(())
    }

    /// Configure Kanidm OIDC integration.
    ///
    /// Valid at: `OidcIntegration`.
    pub fn set_oidc(
        &mut self,
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::OidcIntegration {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-wrong-step").to_string(),
            ));
        }
        let issuer_url = issuer_url.trim();
        let client_id = client_id.trim();
        let client_secret = client_secret.trim();
        if issuer_url.is_empty() || client_id.is_empty() || client_secret.is_empty() {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-oidc-fields-required").to_string(),
            ));
        }
        self.config.oidc = OidcConfig {
            issuer_url: issuer_url.to_owned(),
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
        };
        self.config.skip_oidc = false;
        Ok(())
    }

    /// Skip OIDC — only for offline/standalone testing.
    ///
    /// Valid at: `OidcIntegration`.
    pub fn skip_oidc_for_testing(&mut self) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::OidcIntegration {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-skip-invalid-step").to_string(),
            ));
        }
        self.config.skip_oidc = true;
        Ok(())
    }

    /// Set federation preference.
    ///
    /// Valid at: `Federation`.
    pub fn set_federation(&mut self, enabled: bool) -> Result<(), MatrixManagerError> {
        if self.step != WizardStep::Federation {
            return Err(MatrixManagerError::Validation(
                fs_i18n::t("matrix-wizard-error-wrong-step").to_string(),
            ));
        }
        self.config.federation_enabled = enabled;
        Ok(())
    }

    /// Advance to the next step.
    ///
    /// Returns an error if the current step's required data is missing.
    pub fn advance(&mut self) -> Result<(), MatrixManagerError> {
        match self.step {
            WizardStep::ServerName => {
                if self.config.server_name.is_empty() || self.config.admin_email.is_empty() {
                    return Err(MatrixManagerError::Validation(
                        fs_i18n::t("matrix-wizard-error-server-required").to_string(),
                    ));
                }
                self.step = WizardStep::TlsCerts;
            }
            WizardStep::TlsCerts => {
                if !self.config.tls.is_configured() {
                    return Err(MatrixManagerError::Validation(
                        fs_i18n::t("matrix-wizard-error-tls-required").to_string(),
                    ));
                }
                self.step = WizardStep::OidcIntegration;
            }
            WizardStep::OidcIntegration => {
                if !self.config.skip_oidc && !self.config.oidc.is_configured() {
                    return Err(MatrixManagerError::Validation(
                        fs_i18n::t("matrix-wizard-error-oidc-required").to_string(),
                    ));
                }
                self.step = WizardStep::Federation;
            }
            WizardStep::Federation => {
                self.step = WizardStep::Confirm;
            }
            WizardStep::Confirm => {
                if !self.config.is_valid() {
                    return Err(MatrixManagerError::Validation(
                        fs_i18n::t("matrix-wizard-error-config-incomplete").to_string(),
                    ));
                }
                self.outcome = Some(WizardOutcome {
                    config: self.config.clone(),
                });
                self.step = WizardStep::Done;
            }
            WizardStep::Done => {
                return Err(MatrixManagerError::AlreadyCompleted);
            }
        }
        Ok(())
    }
}

impl Default for TuwunelSetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
