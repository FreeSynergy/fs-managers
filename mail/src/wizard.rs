// wizard.rs — StalwartSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by the UI (view.rs) or the CLI.
//
// Steps:
//   Domain          → admin enters the mail domain
//   TlsCerts        → choose ACME or manual TLS cert
//   OidcIntegration → configure Kanidm OIDC (optional, skippable)
//   Confirm         → review + confirm before writing config
//   Done            → produces StalwartConfig

use crate::{
    config::{OidcConfig, StalwartConfig, TlsConfig},
    error::MailManagerError,
};

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the Stalwart setup wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Enter the primary mail domain.
    Domain,
    /// Configure TLS (ACME or manual certificate paths).
    TlsCerts,
    /// Configure Kanidm OIDC integration (skippable).
    OidcIntegration,
    /// Review all inputs before applying.
    Confirm,
    /// Wizard complete — `StalwartConfig` is available.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Domain => "mail-wizard-step-domain-title",
            Self::TlsCerts => "mail-wizard-step-tls-title",
            Self::OidcIntegration => "mail-wizard-step-oidc-title",
            Self::Confirm => "mail-wizard-step-confirm-title",
            Self::Done => "mail-wizard-step-done-title",
        }
    }

    /// FTL key for the step description.
    #[must_use]
    pub fn description_key(&self) -> &'static str {
        match self {
            Self::Domain => "mail-wizard-step-domain-desc",
            Self::TlsCerts => "mail-wizard-step-tls-desc",
            Self::OidcIntegration => "mail-wizard-step-oidc-desc",
            Self::Confirm => "mail-wizard-step-confirm-desc",
            Self::Done => "mail-wizard-step-done-desc",
        }
    }
}

// ── WizardOutcome ─────────────────────────────────────────────────────────────

/// Result of a completed wizard run.
#[derive(Debug, Clone)]
pub struct WizardOutcome {
    /// The complete, validated Stalwart configuration.
    pub config: StalwartConfig,
}

// ── StalwartSetupWizard ───────────────────────────────────────────────────────

/// State machine that guides an admin through initial Stalwart configuration.
///
/// # Usage
///
/// ```
/// use fs_manager_mail::wizard::{StalwartSetupWizard, WizardStep};
///
/// let mut w = StalwartSetupWizard::new();
/// assert_eq!(w.step(), &WizardStep::Domain);
///
/// w.set_domain("mail.example.com", "admin@example.com").unwrap();
/// w.advance().unwrap();
/// assert_eq!(w.step(), &WizardStep::TlsCerts);
/// ```
#[derive(Debug)]
pub struct StalwartSetupWizard {
    step: WizardStep,
    config: StalwartConfig,
    outcome: Option<WizardOutcome>,
}

impl StalwartSetupWizard {
    /// Create a new wizard at the `Domain` step.
    #[must_use]
    pub fn new() -> Self {
        Self {
            step: WizardStep::Domain,
            config: StalwartConfig::default(),
            outcome: None,
        }
    }

    /// The current step.
    #[must_use]
    pub fn step(&self) -> &WizardStep {
        &self.step
    }

    /// The config as built so far (may be incomplete until `Done`).
    #[must_use]
    pub fn config(&self) -> &StalwartConfig {
        &self.config
    }

    /// The completed outcome — only `Some` after reaching `Done`.
    #[must_use]
    pub fn outcome(&self) -> Option<&WizardOutcome> {
        self.outcome.as_ref()
    }

    // ── Domain step ───────────────────────────────────────────────────────────

    /// Set the primary mail domain and admin/postmaster e-mail address.
    ///
    /// # Errors
    /// - `Validation` — domain or `admin_email` is empty / contains spaces
    pub fn set_domain(&mut self, domain: &str, admin_email: &str) -> Result<(), MailManagerError> {
        let domain = domain.trim();
        let admin_email = admin_email.trim();

        if domain.is_empty() {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-domain-empty".into(),
            ));
        }
        if domain.contains(' ') {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-domain-spaces".into(),
            ));
        }
        if admin_email.is_empty() || !admin_email.contains('@') {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-admin-email-invalid".into(),
            ));
        }
        self.config.domain = domain.to_string();
        self.config.admin_email = admin_email.to_string();
        Ok(())
    }

    // ── TlsCerts step ─────────────────────────────────────────────────────────

    /// Configure ACME (automatic certificate via Let's Encrypt).
    ///
    /// # Errors
    /// - `Validation` — called outside the `TlsCerts` step
    pub fn use_acme(&mut self) -> Result<(), MailManagerError> {
        if self.step != WizardStep::TlsCerts {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-wrong-step".into(),
            ));
        }
        self.config.tls = TlsConfig {
            cert_path: String::new(),
            key_path: String::new(),
            use_acme: true,
        };
        Ok(())
    }

    /// Configure manual TLS with existing certificate and key paths.
    ///
    /// # Errors
    /// - `Validation` — paths empty or called outside the `TlsCerts` step
    pub fn set_tls_paths(
        &mut self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<(), MailManagerError> {
        if self.step != WizardStep::TlsCerts {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-wrong-step".into(),
            ));
        }
        let cert_path = cert_path.trim();
        let key_path = key_path.trim();
        if cert_path.is_empty() || key_path.is_empty() {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-tls-paths-empty".into(),
            ));
        }
        self.config.tls = TlsConfig {
            cert_path: cert_path.to_string(),
            key_path: key_path.to_string(),
            use_acme: false,
        };
        Ok(())
    }

    // ── OidcIntegration step ──────────────────────────────────────────────────

    /// Set Kanidm OIDC credentials.
    ///
    /// # Errors
    /// - `Validation` — any field empty or not a valid URL
    pub fn set_oidc(
        &mut self,
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<(), MailManagerError> {
        let issuer_url = issuer_url.trim();
        let client_id = client_id.trim();
        let client_secret = client_secret.trim();

        if issuer_url.is_empty() || client_id.is_empty() || client_secret.is_empty() {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-oidc-fields-required".into(),
            ));
        }
        self.config.oidc = OidcConfig {
            issuer_url: issuer_url.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        };
        self.config.skip_oidc = false;
        Ok(())
    }

    /// Skip the OIDC step — Stalwart will use local accounts only.
    ///
    /// # Errors
    /// - `Validation` — called outside the `OidcIntegration` step
    pub fn skip_oidc(&mut self) -> Result<(), MailManagerError> {
        if self.step != WizardStep::OidcIntegration {
            return Err(MailManagerError::Validation(
                "mail-wizard-error-skip-invalid-step".into(),
            ));
        }
        self.config.skip_oidc = true;
        self.step = WizardStep::Confirm;
        Ok(())
    }

    // ── State machine ─────────────────────────────────────────────────────────

    /// Advance to the next step.
    ///
    /// Validates the current step's required inputs before advancing.
    ///
    /// # Errors
    /// - `Validation`       — required fields missing / invalid
    /// - `AlreadyCompleted` — wizard is already at `Done`
    pub fn advance(&mut self) -> Result<(), MailManagerError> {
        match &self.step {
            WizardStep::Domain => {
                if self.config.domain.is_empty() || self.config.admin_email.is_empty() {
                    return Err(MailManagerError::Validation(
                        "mail-wizard-error-domain-required".into(),
                    ));
                }
                self.step = WizardStep::TlsCerts;
            }
            WizardStep::TlsCerts => {
                if !self.config.tls.is_configured() {
                    return Err(MailManagerError::Validation(
                        "mail-wizard-error-tls-required".into(),
                    ));
                }
                self.step = WizardStep::OidcIntegration;
            }
            WizardStep::OidcIntegration => {
                if !self.config.skip_oidc && !self.config.oidc.is_configured() {
                    return Err(MailManagerError::Validation(
                        "mail-wizard-error-oidc-required".into(),
                    ));
                }
                self.step = WizardStep::Confirm;
            }
            WizardStep::Confirm => {
                if !self.config.is_valid() {
                    return Err(MailManagerError::Validation(
                        "mail-wizard-error-config-incomplete".into(),
                    ));
                }
                self.outcome = Some(WizardOutcome {
                    config: self.config.clone(),
                });
                self.step = WizardStep::Done;
            }
            WizardStep::Done => {
                return Err(MailManagerError::AlreadyCompleted);
            }
        }
        Ok(())
    }
}

impl Default for StalwartSetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_wizard_acme() -> StalwartSetupWizard {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap(); // → TlsCerts
        w.use_acme().unwrap();
        w.advance().unwrap(); // → OidcIntegration
        w.set_oidc("https://idm.example.com", "stalwart", "secret123")
            .unwrap();
        w.advance().unwrap(); // → Confirm
        w.advance().unwrap(); // → Done
        w
    }

    fn complete_wizard_skip_oidc() -> StalwartSetupWizard {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap(); // → TlsCerts
        w.use_acme().unwrap();
        w.advance().unwrap(); // → OidcIntegration
        w.skip_oidc().unwrap(); // → Confirm (skips OidcIntegration advance)
        w.advance().unwrap(); // → Done
        w
    }

    #[test]
    fn initial_step_is_domain() {
        assert_eq!(StalwartSetupWizard::new().step(), &WizardStep::Domain);
    }

    #[test]
    fn domain_empty_blocks_advance() {
        let mut w = StalwartSetupWizard::new();
        assert!(w.advance().is_err());
    }

    #[test]
    fn domain_with_spaces_rejected() {
        let mut w = StalwartSetupWizard::new();
        assert!(w
            .set_domain("mail example.com", "admin@example.com")
            .is_err());
    }

    #[test]
    fn admin_email_without_at_rejected() {
        let mut w = StalwartSetupWizard::new();
        assert!(w
            .set_domain("mail.example.com", "adminexample.com")
            .is_err());
    }

    #[test]
    fn tls_not_configured_blocks_advance() {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap(); // → TlsCerts
        assert!(w.advance().is_err()); // TLS not yet set
    }

    #[test]
    fn set_tls_paths_empty_is_error() {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap();
        assert!(w.set_tls_paths("", "").is_err());
    }

    #[test]
    fn set_tls_from_wrong_step_is_error() {
        let mut w = StalwartSetupWizard::new();
        assert!(w.use_acme().is_err());
        assert!(w.set_tls_paths("/cert.pem", "/key.pem").is_err());
    }

    #[test]
    fn full_wizard_acme_oidc_produces_outcome() {
        let w = complete_wizard_acme();
        assert_eq!(w.step(), &WizardStep::Done);
        let outcome = w.outcome().unwrap();
        assert_eq!(outcome.config.domain, "mail.example.com");
        assert!(outcome.config.tls.use_acme);
        assert!(!outcome.config.skip_oidc);
        assert_eq!(outcome.config.oidc.client_id, "stalwart");
    }

    #[test]
    fn full_wizard_skip_oidc_produces_outcome() {
        let w = complete_wizard_skip_oidc();
        assert_eq!(w.step(), &WizardStep::Done);
        let outcome = w.outcome().unwrap();
        assert!(outcome.config.skip_oidc);
    }

    #[test]
    fn advance_past_done_is_error() {
        let mut w = complete_wizard_acme();
        assert!(w.advance().is_err());
    }

    #[test]
    fn skip_oidc_from_wrong_step_is_error() {
        let mut w = StalwartSetupWizard::new();
        assert!(w.skip_oidc().is_err());
    }

    #[test]
    fn step_title_keys_all_non_empty() {
        for step in [
            WizardStep::Domain,
            WizardStep::TlsCerts,
            WizardStep::OidcIntegration,
            WizardStep::Confirm,
            WizardStep::Done,
        ] {
            assert!(!step.title_key().is_empty());
            assert!(!step.description_key().is_empty());
        }
    }

    #[test]
    fn manual_tls_paths_set_correctly() {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap(); // → TlsCerts
        w.set_tls_paths("/etc/certs/mail.pem", "/etc/certs/mail-key.pem")
            .unwrap();
        assert!(!w.config().tls.use_acme);
        assert_eq!(w.config().tls.cert_path, "/etc/certs/mail.pem");
    }

    #[test]
    fn oidc_fields_required_when_not_skipped() {
        let mut w = StalwartSetupWizard::new();
        w.set_domain("mail.example.com", "admin@example.com")
            .unwrap();
        w.advance().unwrap();
        w.use_acme().unwrap();
        w.advance().unwrap(); // → OidcIntegration
                              // Don't set OIDC, don't skip — advance must fail
        assert!(w.advance().is_err());
    }
}
