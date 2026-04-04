// wizard.rs — WikiSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by view.rs or the CLI.
//
// Steps:
//   Platform   → choose Outline (default) or Wiki.js
//   Domain     → enter the public wiki domain
//   Oidc       → configure Kanidm OIDC (issuer, client ID, secret ref)
//   S3         → configure S3 storage (optional, skippable)
//   Confirm    → review all inputs before saving
//   Done       → produces WikiConfig

use crate::{
    config::{OidcConfig, S3Config, WikiConfig, WikiConfigStore, WikiPlatform},
    error::WikiManagerError,
};
use std::path::PathBuf;

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the wiki setup wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Choose the wiki platform (Outline or Wiki.js).
    Platform,
    /// Enter the public wiki domain (e.g. `wiki.example.com`).
    Domain,
    /// Configure Kanidm OIDC integration.
    Oidc,
    /// Configure S3 storage for attachments (optional).
    S3,
    /// Review all inputs before saving.
    Confirm,
    /// Wizard complete — [`WikiConfig`] is ready.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Platform => crate::keys::WIZARD_STEP_PLATFORM_TITLE,
            Self::Domain => crate::keys::WIZARD_STEP_DOMAIN_TITLE,
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
    Saved(WikiConfig),
    /// User cancelled the wizard.
    Cancelled,
}

// ── WikiSetupWizard ───────────────────────────────────────────────────────────

/// State machine that guides an admin through initial wiki configuration.
///
/// # Example
///
/// ```
/// use fs_manager_wiki::wizard::{WikiSetupWizard, WizardStep};
/// use std::path::PathBuf;
///
/// let mut w = WikiSetupWizard::new(PathBuf::from("/tmp/wiki/config.toml"));
/// assert_eq!(w.step(), &WizardStep::Platform);
/// ```
pub struct WikiSetupWizard {
    step: WizardStep,
    config: WikiConfig,
    config_path: PathBuf,
}

impl WikiSetupWizard {
    /// Create a new wizard writing to `config_path`.
    #[must_use]
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            step: WizardStep::Platform,
            config: WikiConfig::default(),
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
    pub fn config(&self) -> &WikiConfig {
        &self.config
    }

    // ── Step: Platform ────────────────────────────────────────────────────────

    /// Set the wiki platform and advance to `Domain`.
    ///
    /// # Errors
    ///
    /// Returns [`WikiManagerError::WrongStep`] if not on the `Platform` step.
    pub fn set_platform(&mut self, platform: WikiPlatform) -> Result<(), WikiManagerError> {
        if self.step != WizardStep::Platform {
            return Err(WikiManagerError::WrongStep);
        }
        self.config.platform = platform;
        self.step = WizardStep::Domain;
        Ok(())
    }

    // ── Step: Domain ──────────────────────────────────────────────────────────

    /// Set the public domain and advance to `Oidc`.
    ///
    /// # Errors
    ///
    /// Returns an error if `domain` is empty or on the wrong step.
    pub fn set_domain(&mut self, domain: impl Into<String>) -> Result<(), WikiManagerError> {
        if self.step != WizardStep::Domain {
            return Err(WikiManagerError::WrongStep);
        }
        let domain = domain.into();
        if domain.is_empty() {
            return Err(WikiManagerError::Validation(
                "Domain must not be empty".into(),
            ));
        }
        self.config.domain = domain;
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
    ) -> Result<(), WikiManagerError> {
        if self.step != WizardStep::Oidc {
            return Err(WikiManagerError::WrongStep);
        }
        let issuer_url = issuer_url.into();
        let client_id = client_id.into();
        let client_secret_ref = client_secret_ref.into();

        if issuer_url.is_empty() {
            return Err(WikiManagerError::Validation(
                "OIDC issuer URL must not be empty".into(),
            ));
        }
        if client_id.is_empty() {
            return Err(WikiManagerError::Validation(
                "OIDC client ID must not be empty".into(),
            ));
        }
        if !client_secret_ref.starts_with("env:") && !client_secret_ref.starts_with("file:") {
            return Err(WikiManagerError::Validation(
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
    /// All four fields are required (`endpoint`, `bucket`, `access_key_ref`, `secret_key_ref`).
    /// Both key refs must start with `env:` or `file:`.
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
    ) -> Result<(), WikiManagerError> {
        if self.step != WizardStep::S3 {
            return Err(WikiManagerError::WrongStep);
        }
        let endpoint = endpoint.into();
        let bucket = bucket.into();
        let access_key_ref = access_key_ref.into();
        let secret_key_ref = secret_key_ref.into();

        if endpoint.is_empty() || bucket.is_empty() {
            return Err(WikiManagerError::Validation(
                "S3 endpoint and bucket must not be empty".into(),
            ));
        }
        for (label, r) in [
            ("access key", &access_key_ref),
            ("secret key", &secret_key_ref),
        ] {
            if !r.starts_with("env:") && !r.starts_with("file:") {
                return Err(WikiManagerError::Validation(format!(
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
    /// Returns [`WikiManagerError::WrongStep`] if not on the `S3` step.
    pub fn skip_s3(&mut self) -> Result<(), WikiManagerError> {
        if self.step != WizardStep::S3 {
            return Err(WikiManagerError::WrongStep);
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
    pub fn confirm(&mut self) -> Result<WizardOutcome, WikiManagerError> {
        if self.step != WizardStep::Confirm {
            return Err(WikiManagerError::WrongStep);
        }
        if !self.config.is_valid() {
            return Err(WikiManagerError::Validation(
                "Configuration is incomplete".into(),
            ));
        }
        let store = WikiConfigStore::new(&self.config_path);
        store
            .save(&self.config)
            .map_err(WikiManagerError::ConfigWrite)?;
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

    fn wizard() -> WikiSetupWizard {
        WikiSetupWizard::new(PathBuf::from("/tmp/test-wiki/config.toml"))
    }

    fn advance_to_domain(w: &mut WikiSetupWizard) {
        w.set_platform(WikiPlatform::Outline).unwrap();
    }

    fn advance_to_oidc(w: &mut WikiSetupWizard) {
        advance_to_domain(w);
        w.set_domain("wiki.example.com").unwrap();
    }

    fn advance_to_s3(w: &mut WikiSetupWizard) {
        advance_to_oidc(w);
        w.set_oidc(
            "https://idm.example.com",
            "outline",
            "env:FS_WIKI_OIDC_SECRET",
        )
        .unwrap();
    }

    fn advance_to_confirm(w: &mut WikiSetupWizard) {
        advance_to_s3(w);
        w.skip_s3().unwrap();
    }

    #[test]
    fn initial_step_is_platform() {
        assert_eq!(wizard().step(), &WizardStep::Platform);
    }

    #[test]
    fn set_platform_advances_to_domain() {
        let mut w = wizard();
        w.set_platform(WikiPlatform::WikiJs).unwrap();
        assert_eq!(w.step(), &WizardStep::Domain);
        assert_eq!(w.config().platform, WikiPlatform::WikiJs);
    }

    #[test]
    fn set_platform_wrong_step_is_error() {
        let mut w = wizard();
        advance_to_domain(&mut w);
        assert!(w.set_platform(WikiPlatform::Outline).is_err());
    }

    #[test]
    fn empty_domain_is_rejected() {
        let mut w = wizard();
        advance_to_domain(&mut w);
        assert!(w.set_domain("").is_err());
    }

    #[test]
    fn set_domain_advances_to_oidc() {
        let mut w = wizard();
        advance_to_domain(&mut w);
        w.set_domain("wiki.example.com").unwrap();
        assert_eq!(w.step(), &WizardStep::Oidc);
    }

    #[test]
    fn plain_text_secret_ref_is_rejected() {
        let mut w = wizard();
        advance_to_oidc(&mut w);
        assert!(w
            .set_oidc("https://idm.example.com", "outline", "my-plain-secret")
            .is_err());
    }

    #[test]
    fn oidc_advances_to_s3() {
        let mut w = wizard();
        advance_to_oidc(&mut w);
        w.set_oidc(
            "https://idm.example.com",
            "outline",
            "env:FS_WIKI_OIDC_SECRET",
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
    fn s3_invalid_key_ref_is_rejected() {
        let mut w = wizard();
        advance_to_s3(&mut w);
        assert!(w
            .set_s3(
                "https://s3.example.com",
                "wiki-bucket",
                "plain-key",
                "env:FS_WIKI_S3_SECRET",
            )
            .is_err());
    }

    #[test]
    fn s3_configured_advances_to_confirm() {
        let mut w = wizard();
        advance_to_s3(&mut w);
        w.set_s3(
            "https://s3.example.com",
            "wiki-bucket",
            "env:FS_WIKI_S3_ACCESS",
            "env:FS_WIKI_S3_SECRET",
        )
        .unwrap();
        assert_eq!(w.step(), &WizardStep::Confirm);
        assert!(w.config().s3.is_some());
    }

    #[test]
    fn cancel_returns_cancelled() {
        let w = wizard();
        assert!(matches!(w.cancel(), WizardOutcome::Cancelled));
    }

    #[test]
    fn step_title_keys_all_non_empty() {
        for step in [
            WizardStep::Platform,
            WizardStep::Domain,
            WizardStep::Oidc,
            WizardStep::S3,
            WizardStep::Confirm,
            WizardStep::Done,
        ] {
            assert!(!step.title_key().is_empty());
        }
    }

    #[test]
    fn confirm_on_wrong_step_is_error() {
        let mut w = wizard();
        advance_to_s3(&mut w); // still on S3 step
        assert!(w.confirm().is_err());
    }

    #[test]
    fn full_wizard_without_s3_produces_valid_config() {
        let mut w = wizard();
        advance_to_confirm(&mut w);
        assert!(w.config().is_valid());
        assert_eq!(w.config().domain, "wiki.example.com");
        assert_eq!(w.config().platform, WikiPlatform::Outline);
        assert!(w.config().s3.is_none());
    }
}
