// wizard.rs — ForgejoSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by view.rs or the CLI.
//
// Steps:
//   Domain   → enter the public git domain (e.g. git.example.com)
//   Ssh      → configure SSH port
//   Oidc     → configure Kanidm OIDC
//   S3       → configure S3 storage for LFS (optional, skippable)
//   Confirm  → review all inputs before saving
//   Done     → produces ForgejoConfig

use crate::{
    config::{ForgejoConfig, ForgejoConfigStore, OidcConfig, S3Config},
    error::ForgejoManagerError,
};
use std::path::PathBuf;

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the Forgejo setup wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Enter the public git domain.
    Domain,
    /// Configure SSH port.
    Ssh,
    /// Configure Kanidm OIDC integration.
    Oidc,
    /// Configure S3 storage for LFS (optional).
    S3,
    /// Review all inputs before saving.
    Confirm,
    /// Wizard complete — [`ForgejoConfig`] is ready.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Domain => crate::keys::WIZARD_STEP_DOMAIN_TITLE,
            Self::Ssh => crate::keys::WIZARD_STEP_SSH_TITLE,
            Self::Oidc => crate::keys::WIZARD_STEP_OIDC_TITLE,
            Self::S3 => crate::keys::WIZARD_STEP_S3_TITLE,
            Self::Confirm => crate::keys::WIZARD_STEP_CONFIRM_TITLE,
            Self::Done => crate::keys::WIZARD_STEP_DONE_TITLE,
        }
    }
}

// ── WizardOutcome ─────────────────────────────────────────────────────────────

/// Result of a completed wizard run.
#[derive(Debug, Clone)]
pub enum WizardOutcome {
    /// Config saved successfully.
    Saved(ForgejoConfig),
    /// User cancelled the wizard.
    Cancelled,
}

// ── ForgejoSetupWizard ────────────────────────────────────────────────────────

/// State machine that guides an admin through initial Forgejo configuration.
///
/// # Example
///
/// ```
/// use fs_manager_forgejo::wizard::{ForgejoSetupWizard, WizardStep};
/// use std::path::PathBuf;
///
/// let w = ForgejoSetupWizard::new(PathBuf::from("/tmp/forgejo/config.toml"));
/// assert_eq!(w.step(), &WizardStep::Domain);
/// ```
pub struct ForgejoSetupWizard {
    step: WizardStep,
    config: ForgejoConfig,
    config_path: PathBuf,
}

impl ForgejoSetupWizard {
    /// Create a new wizard writing to `config_path`.
    #[must_use]
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            step: WizardStep::Domain,
            config: ForgejoConfig::default(),
            config_path,
        }
    }

    /// Return the current wizard step.
    #[must_use]
    pub fn step(&self) -> &WizardStep {
        &self.step
    }

    /// Return the config being built (may be incomplete until `Done`).
    #[must_use]
    pub fn config(&self) -> &ForgejoConfig {
        &self.config
    }

    // ── Step: Domain ──────────────────────────────────────────────────────────

    /// Set the public domain and advance to `Ssh`.
    ///
    /// # Errors
    ///
    /// Returns an error if `domain` is empty or on the wrong step.
    pub fn set_domain(&mut self, domain: impl Into<String>) -> Result<(), ForgejoManagerError> {
        if self.step != WizardStep::Domain {
            return Err(ForgejoManagerError::WrongStep);
        }
        let domain = domain.into();
        if domain.is_empty() {
            return Err(ForgejoManagerError::Validation(
                "Domain must not be empty".into(),
            ));
        }
        self.config.domain = domain;
        self.step = WizardStep::Ssh;
        Ok(())
    }

    // ── Step: Ssh ─────────────────────────────────────────────────────────────

    /// Set the SSH port and advance to `Oidc`.
    ///
    /// # Errors
    ///
    /// Returns an error if `port` is 0 or on the wrong step.
    pub fn set_ssh_port(&mut self, port: u16) -> Result<(), ForgejoManagerError> {
        if self.step != WizardStep::Ssh {
            return Err(ForgejoManagerError::WrongStep);
        }
        if port == 0 {
            return Err(ForgejoManagerError::Validation(
                "SSH port must not be 0".into(),
            ));
        }
        self.config.ssh_port = port;
        self.step = WizardStep::Oidc;
        Ok(())
    }

    // ── Step: Oidc ────────────────────────────────────────────────────────────

    /// Set OIDC configuration and advance to `S3`.
    ///
    /// `client_secret_ref` must start with `env:` or `file:`.
    ///
    /// # Errors
    ///
    /// Returns an error if any field is empty, the secret ref is invalid, or
    /// the step is wrong.
    pub fn set_oidc(
        &mut self,
        issuer_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret_ref: impl Into<String>,
    ) -> Result<(), ForgejoManagerError> {
        if self.step != WizardStep::Oidc {
            return Err(ForgejoManagerError::WrongStep);
        }
        let issuer_url = issuer_url.into();
        let client_id = client_id.into();
        let client_secret_ref = client_secret_ref.into();

        if issuer_url.is_empty() {
            return Err(ForgejoManagerError::Validation(
                "OIDC issuer URL must not be empty".into(),
            ));
        }
        if client_id.is_empty() {
            return Err(ForgejoManagerError::Validation(
                "OIDC client ID must not be empty".into(),
            ));
        }
        if !client_secret_ref.starts_with("env:") && !client_secret_ref.starts_with("file:") {
            return Err(ForgejoManagerError::Validation(
                "Client secret reference must start with 'env:' or 'file:'".into(),
            ));
        }

        self.config.oidc = OidcConfig {
            issuer_url,
            client_id,
            client_secret_ref,
        };
        self.step = WizardStep::S3;
        Ok(())
    }

    // ── Step: S3 ──────────────────────────────────────────────────────────────

    /// Configure S3 storage and advance to `Confirm`.
    ///
    /// # Errors
    ///
    /// Returns an error if any field is empty, refs are invalid, or step is wrong.
    pub fn set_s3(
        &mut self,
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        access_key_ref: impl Into<String>,
        secret_key_ref: impl Into<String>,
    ) -> Result<(), ForgejoManagerError> {
        if self.step != WizardStep::S3 {
            return Err(ForgejoManagerError::WrongStep);
        }
        let endpoint = endpoint.into();
        let bucket = bucket.into();
        let access_key_ref = access_key_ref.into();
        let secret_key_ref = secret_key_ref.into();

        if endpoint.is_empty() || bucket.is_empty() {
            return Err(ForgejoManagerError::Validation(
                "S3 endpoint and bucket must not be empty".into(),
            ));
        }
        for (label, r) in [
            ("access key", &access_key_ref),
            ("secret key", &secret_key_ref),
        ] {
            if !r.starts_with("env:") && !r.starts_with("file:") {
                return Err(ForgejoManagerError::Validation(format!(
                    "S3 {label} reference must start with 'env:' or 'file:'"
                )));
            }
        }

        self.config.s3 = Some(S3Config {
            endpoint,
            bucket,
            access_key_ref,
            secret_key_ref,
        });
        self.step = WizardStep::Confirm;
        Ok(())
    }

    /// Skip S3 configuration and advance to `Confirm`.
    ///
    /// # Errors
    ///
    /// Returns [`ForgejoManagerError::WrongStep`] if not on the `S3` step.
    pub fn skip_s3(&mut self) -> Result<(), ForgejoManagerError> {
        if self.step != WizardStep::S3 {
            return Err(ForgejoManagerError::WrongStep);
        }
        self.config.s3 = None;
        self.step = WizardStep::Confirm;
        Ok(())
    }

    // ── Step: Confirm ─────────────────────────────────────────────────────────

    /// Confirm and save the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if not on the `Confirm` step, the config is invalid,
    /// or the file cannot be written.
    pub fn confirm(&mut self) -> Result<WizardOutcome, ForgejoManagerError> {
        if self.step != WizardStep::Confirm {
            return Err(ForgejoManagerError::WrongStep);
        }
        if !self.config.is_valid() {
            return Err(ForgejoManagerError::Validation(
                "Configuration is incomplete".into(),
            ));
        }
        let store = ForgejoConfigStore::new(&self.config_path);
        store
            .save(&self.config)
            .map_err(ForgejoManagerError::ConfigWrite)?;
        self.step = WizardStep::Done;
        Ok(WizardOutcome::Saved(self.config.clone()))
    }

    /// Cancel the wizard without saving.
    #[must_use]
    pub fn cancel(mut self) -> WizardOutcome {
        self.step = WizardStep::Done;
        WizardOutcome::Cancelled
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn wizard() -> ForgejoSetupWizard {
        ForgejoSetupWizard::new(PathBuf::from("/tmp/test-forgejo/config.toml"))
    }

    fn advance_to_ssh(w: &mut ForgejoSetupWizard) {
        w.set_domain("git.example.com").unwrap();
    }

    fn advance_to_oidc(w: &mut ForgejoSetupWizard) {
        advance_to_ssh(w);
        w.set_ssh_port(22).unwrap();
    }

    fn advance_to_s3(w: &mut ForgejoSetupWizard) {
        advance_to_oidc(w);
        w.set_oidc(
            "https://idm.example.com",
            "forgejo",
            "env:FS_FORGEJO_OIDC_SECRET",
        )
        .unwrap();
    }

    fn advance_to_confirm(w: &mut ForgejoSetupWizard) {
        advance_to_s3(w);
        w.skip_s3().unwrap();
    }

    #[test]
    fn initial_step_is_domain() {
        assert_eq!(wizard().step(), &WizardStep::Domain);
    }

    #[test]
    fn empty_domain_rejected() {
        let mut w = wizard();
        assert!(w.set_domain("").is_err());
    }

    #[test]
    fn set_domain_advances_to_ssh() {
        let mut w = wizard();
        w.set_domain("git.example.com").unwrap();
        assert_eq!(w.step(), &WizardStep::Ssh);
    }

    #[test]
    fn zero_ssh_port_rejected() {
        let mut w = wizard();
        advance_to_ssh(&mut w);
        assert!(w.set_ssh_port(0).is_err());
    }

    #[test]
    fn set_ssh_port_advances_to_oidc() {
        let mut w = wizard();
        advance_to_ssh(&mut w);
        w.set_ssh_port(22).unwrap();
        assert_eq!(w.step(), &WizardStep::Oidc);
    }

    #[test]
    fn plain_text_secret_ref_rejected() {
        let mut w = wizard();
        advance_to_oidc(&mut w);
        assert!(w
            .set_oidc("https://idm.example.com", "forgejo", "plain-secret")
            .is_err());
    }

    #[test]
    fn oidc_advances_to_s3() {
        let mut w = wizard();
        advance_to_oidc(&mut w);
        w.set_oidc(
            "https://idm.example.com",
            "forgejo",
            "env:FS_FORGEJO_OIDC_SECRET",
        )
        .unwrap();
        assert_eq!(w.step(), &WizardStep::S3);
    }

    #[test]
    fn skip_s3_advances_to_confirm() {
        let mut w = wizard();
        advance_to_s3(&mut w);
        w.skip_s3().unwrap();
        assert_eq!(w.step(), &WizardStep::Confirm);
        assert!(w.config().s3.is_none());
    }

    #[test]
    fn s3_invalid_key_ref_rejected() {
        let mut w = wizard();
        advance_to_s3(&mut w);
        assert!(w
            .set_s3(
                "https://s3.example.com",
                "forgejo-bucket",
                "plain-access-key",
                "env:FS_FORGEJO_S3_SECRET",
            )
            .is_err());
    }

    #[test]
    fn confirm_on_wrong_step_is_error() {
        let mut w = wizard();
        advance_to_s3(&mut w);
        assert!(w.confirm().is_err());
    }

    #[test]
    fn cancel_returns_cancelled() {
        let w = wizard();
        assert!(matches!(w.cancel(), WizardOutcome::Cancelled));
    }

    #[test]
    fn full_wizard_without_s3_produces_valid_config() {
        let mut w = wizard();
        advance_to_confirm(&mut w);
        assert!(w.config().is_valid());
        assert_eq!(w.config().domain, "git.example.com");
        assert_eq!(w.config().ssh_port, 22);
        assert!(w.config().s3.is_none());
    }

    #[test]
    fn step_title_keys_all_non_empty() {
        for step in [
            WizardStep::Domain,
            WizardStep::Ssh,
            WizardStep::Oidc,
            WizardStep::S3,
            WizardStep::Confirm,
            WizardStep::Done,
        ] {
            assert!(!step.title_key().is_empty());
        }
    }
}
