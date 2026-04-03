// wizard.rs — KanidmSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by the UI (view.rs) or the CLI.
//
// Steps:
//   Domain        → admin enters the Kanidm domain
//   AdminAccount  → admin chooses username + password
//   OidcClients   → register OIDC clients (repeatable, can be empty)
//   Confirm       → review + confirm before writing config
//   Done          → produces KanidmConfig

use crate::{
    config::{KanidmConfig, OidcClient},
    error::AuthManagerError,
};

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the Kanidm setup wizard.
///
/// The state machine advances strictly forward — no going back.
/// The UI renders a different form for each variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Enter the Kanidm domain (e.g. `idm.example.com`).
    Domain,
    /// Choose the initial admin username + password.
    AdminAccount,
    /// Register OIDC clients (optional, repeatable).
    OidcClients,
    /// Review all inputs before applying.
    Confirm,
    /// Wizard complete — `KanidmConfig` is available.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Domain => "auth-wizard-step-domain-title",
            Self::AdminAccount => "auth-wizard-step-admin-title",
            Self::OidcClients => "auth-wizard-step-oidc-title",
            Self::Confirm => "auth-wizard-step-confirm-title",
            Self::Done => "auth-wizard-step-done-title",
        }
    }

    /// FTL key for the step description.
    #[must_use]
    pub fn description_key(&self) -> &'static str {
        match self {
            Self::Domain => "auth-wizard-step-domain-desc",
            Self::AdminAccount => "auth-wizard-step-admin-desc",
            Self::OidcClients => "auth-wizard-step-oidc-desc",
            Self::Confirm => "auth-wizard-step-confirm-desc",
            Self::Done => "auth-wizard-step-done-desc",
        }
    }
}

// ── WizardOutcome ─────────────────────────────────────────────────────────────

/// Result of a completed wizard run.
#[derive(Debug, Clone)]
pub struct WizardOutcome {
    /// The complete, validated Kanidm configuration.
    pub config: KanidmConfig,
}

// ── KanidmSetupWizard ─────────────────────────────────────────────────────────

/// State machine that guides an admin through initial Kanidm configuration.
///
/// # Usage
///
/// ```
/// use fs_manager_auth::wizard::KanidmSetupWizard;
/// use fs_manager_auth::wizard::WizardStep;
///
/// let mut w = KanidmSetupWizard::new();
/// assert_eq!(w.step(), &WizardStep::Domain);
///
/// w.set_domain("idm.example.com").unwrap();
/// w.advance().unwrap();
/// assert_eq!(w.step(), &WizardStep::AdminAccount);
/// ```
#[derive(Debug)]
pub struct KanidmSetupWizard {
    step: WizardStep,
    config: KanidmConfig,
    outcome: Option<WizardOutcome>,
}

impl KanidmSetupWizard {
    /// Create a new wizard at the `Domain` step.
    #[must_use]
    pub fn new() -> Self {
        Self {
            step: WizardStep::Domain,
            config: KanidmConfig::default(),
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
    pub fn config(&self) -> &KanidmConfig {
        &self.config
    }

    /// The completed outcome — only `Some` after reaching `Done`.
    #[must_use]
    pub fn outcome(&self) -> Option<&WizardOutcome> {
        self.outcome.as_ref()
    }

    // ── Domain step ───────────────────────────────────────────────────────────

    /// Set the Kanidm domain.
    ///
    /// Must be called before advancing past `Domain`.
    ///
    /// # Errors
    /// Returns `Validation` if `domain` is empty or contains spaces.
    pub fn set_domain(&mut self, domain: &str) -> Result<(), AuthManagerError> {
        let domain = domain.trim();
        if domain.is_empty() {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-domain-empty".into(),
            ));
        }
        if domain.contains(' ') {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-domain-spaces".into(),
            ));
        }
        self.config.domain = domain.to_string();
        Ok(())
    }

    // ── AdminAccount step ─────────────────────────────────────────────────────

    /// Set the initial admin credentials.
    ///
    /// # Errors
    /// Returns `Validation` if username or password is empty.
    pub fn set_admin(&mut self, username: &str, password: &str) -> Result<(), AuthManagerError> {
        let username = username.trim();
        let password = password.trim();
        if username.is_empty() {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-admin-username-empty".into(),
            ));
        }
        if password.len() < 8 {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-admin-password-short".into(),
            ));
        }
        self.config.admin_username = username.to_string();
        self.config.admin_password = password.to_string();
        Ok(())
    }

    // ── OidcClients step ──────────────────────────────────────────────────────

    /// Add an OIDC client to register on first Kanidm start.
    ///
    /// This step is optional and repeatable.
    ///
    /// # Errors
    /// Returns `Validation` if any field is empty.
    pub fn add_oidc_client(
        &mut self,
        id: &str,
        display_name: &str,
        redirect_uri: &str,
    ) -> Result<(), AuthManagerError> {
        let id = id.trim();
        let display_name = display_name.trim();
        let redirect_uri = redirect_uri.trim();
        if id.is_empty() || display_name.is_empty() || redirect_uri.is_empty() {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-oidc-fields-required".into(),
            ));
        }
        self.config.oidc_clients.push(OidcClient {
            id: id.to_string(),
            display_name: display_name.to_string(),
            redirect_uri: redirect_uri.to_string(),
        });
        Ok(())
    }

    /// Remove an OIDC client by index.
    pub fn remove_oidc_client(&mut self, index: usize) {
        if index < self.config.oidc_clients.len() {
            self.config.oidc_clients.remove(index);
        }
    }

    // ── State machine ─────────────────────────────────────────────────────────

    /// Advance to the next step.
    ///
    /// Validates the current step's required inputs before advancing.
    ///
    /// # Errors
    /// - `Validation`      — required fields missing / invalid
    /// - `AlreadyCompleted` — wizard is already at `Done`
    pub fn advance(&mut self) -> Result<(), AuthManagerError> {
        match &self.step {
            WizardStep::Domain => {
                if self.config.domain.is_empty() {
                    return Err(AuthManagerError::Validation(
                        "auth-wizard-error-domain-empty".into(),
                    ));
                }
                self.step = WizardStep::AdminAccount;
            }
            WizardStep::AdminAccount => {
                if self.config.admin_username.is_empty() || self.config.admin_password.is_empty() {
                    return Err(AuthManagerError::Validation(
                        "auth-wizard-error-admin-required".into(),
                    ));
                }
                self.step = WizardStep::OidcClients;
            }
            WizardStep::OidcClients => {
                self.step = WizardStep::Confirm;
            }
            WizardStep::Confirm => {
                if !self.config.is_valid() {
                    return Err(AuthManagerError::Validation(
                        "auth-wizard-error-config-incomplete".into(),
                    ));
                }
                self.outcome = Some(WizardOutcome {
                    config: self.config.clone(),
                });
                self.step = WizardStep::Done;
            }
            WizardStep::Done => {
                return Err(AuthManagerError::AlreadyCompleted);
            }
        }
        Ok(())
    }

    /// Skip the `OidcClients` step (advance without adding any clients).
    ///
    /// Convenience for setups that add OIDC clients later via the manager UI.
    ///
    /// # Errors
    /// Returns `Validation` if called outside the `OidcClients` step.
    pub fn skip_oidc(&mut self) -> Result<(), AuthManagerError> {
        if self.step != WizardStep::OidcClients {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-skip-invalid-step".into(),
            ));
        }
        self.step = WizardStep::Confirm;
        Ok(())
    }
}

impl Default for KanidmSetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_wizard() -> KanidmSetupWizard {
        let mut w = KanidmSetupWizard::new();
        w.set_domain("idm.example.com").unwrap();
        w.advance().unwrap(); // → AdminAccount
        w.set_admin("admin", "supersecret123").unwrap();
        w.advance().unwrap(); // → OidcClients
        w.skip_oidc().unwrap(); // → Confirm
        w.advance().unwrap(); // → Done
        w
    }

    #[test]
    fn initial_step_is_domain() {
        assert_eq!(KanidmSetupWizard::new().step(), &WizardStep::Domain);
    }

    #[test]
    fn domain_validation_rejects_empty() {
        let mut w = KanidmSetupWizard::new();
        assert!(w.advance().is_err()); // domain not set
    }

    #[test]
    fn domain_validation_rejects_spaces() {
        let mut w = KanidmSetupWizard::new();
        assert!(w.set_domain("id m.example.com").is_err());
    }

    #[test]
    fn full_wizard_produces_outcome() {
        let w = complete_wizard();
        assert_eq!(w.step(), &WizardStep::Done);
        let outcome = w.outcome().unwrap();
        assert_eq!(outcome.config.domain, "idm.example.com");
        assert_eq!(outcome.config.admin_username, "admin");
        assert!(outcome.config.oidc_clients.is_empty());
    }

    #[test]
    fn add_oidc_client_appears_in_config() {
        let mut w = KanidmSetupWizard::new();
        w.set_domain("idm.example.com").unwrap();
        w.advance().unwrap();
        w.set_admin("admin", "supersecret123").unwrap();
        w.advance().unwrap();
        w.add_oidc_client("forgejo", "Forgejo", "https://git.example.com/callback")
            .unwrap();
        assert_eq!(w.config().oidc_clients.len(), 1);
        w.advance().unwrap(); // → Confirm
        w.advance().unwrap(); // → Done
        assert_eq!(w.outcome().unwrap().config.oidc_clients.len(), 1);
    }

    #[test]
    fn remove_oidc_client() {
        let mut w = KanidmSetupWizard::new();
        w.set_domain("idm.example.com").unwrap();
        w.advance().unwrap();
        w.set_admin("admin", "supersecret123").unwrap();
        w.advance().unwrap();
        w.add_oidc_client("forgejo", "Forgejo", "https://git.example.com/callback")
            .unwrap();
        w.add_oidc_client("outline", "Outline", "https://docs.example.com/callback")
            .unwrap();
        w.remove_oidc_client(0);
        assert_eq!(w.config().oidc_clients[0].id, "outline");
    }

    #[test]
    fn admin_password_too_short_rejected() {
        let mut w = KanidmSetupWizard::new();
        w.set_domain("idm.example.com").unwrap();
        w.advance().unwrap();
        assert!(w.set_admin("admin", "short").is_err());
    }

    #[test]
    fn advance_past_done_is_error() {
        let w = complete_wizard();
        let mut w = w;
        assert!(w.advance().is_err());
    }

    #[test]
    fn skip_oidc_from_wrong_step_is_error() {
        let mut w = KanidmSetupWizard::new();
        assert!(w.skip_oidc().is_err());
    }

    #[test]
    fn step_title_keys_are_non_empty() {
        for step in [
            WizardStep::Domain,
            WizardStep::AdminAccount,
            WizardStep::OidcClients,
            WizardStep::Confirm,
            WizardStep::Done,
        ] {
            assert!(!step.title_key().is_empty());
            assert!(!step.description_key().is_empty());
        }
    }
}
