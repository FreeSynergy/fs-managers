// keys.rs — FTL key name constants for fs-manager-forgejo.

// Wizard step titles
pub const WIZARD_STEP_DOMAIN_TITLE: &str = "forgejo-wizard-step-domain-title";
pub const WIZARD_STEP_OIDC_TITLE: &str = "forgejo-wizard-step-oidc-title";
pub const WIZARD_STEP_S3_TITLE: &str = "forgejo-wizard-step-s3-title";
pub const WIZARD_STEP_SSH_TITLE: &str = "forgejo-wizard-step-ssh-title";
pub const WIZARD_STEP_CONFIRM_TITLE: &str = "forgejo-wizard-step-confirm-title";
pub const WIZARD_STEP_DONE_TITLE: &str = "forgejo-wizard-step-done-title";

// Wizard prompts
pub const WIZARD_DOMAIN_PROMPT: &str = "forgejo-wizard-domain-prompt";
pub const WIZARD_SSH_PORT_PROMPT: &str = "forgejo-wizard-ssh-port-prompt";
pub const WIZARD_OIDC_ISSUER_PROMPT: &str = "forgejo-wizard-oidc-issuer-prompt";
pub const WIZARD_OIDC_CLIENT_ID_PROMPT: &str = "forgejo-wizard-oidc-client-id-prompt";
pub const WIZARD_OIDC_SECRET_PROMPT: &str = "forgejo-wizard-oidc-secret-prompt";
pub const WIZARD_S3_ENDPOINT_PROMPT: &str = "forgejo-wizard-s3-endpoint-prompt";
pub const WIZARD_S3_BUCKET_PROMPT: &str = "forgejo-wizard-s3-bucket-prompt";
pub const WIZARD_S3_ACCESS_KEY_PROMPT: &str = "forgejo-wizard-s3-access-key-prompt";
pub const WIZARD_S3_SECRET_KEY_PROMPT: &str = "forgejo-wizard-s3-secret-key-prompt";
pub const WIZARD_SKIP_S3: &str = "forgejo-wizard-skip-s3";

// Wizard outcome
pub const WIZARD_DONE_TITLE: &str = "forgejo-wizard-done-title";
pub const WIZARD_DONE_HINT: &str = "forgejo-wizard-done-hint";
pub const WIZARD_CANCELLED: &str = "forgejo-wizard-cancelled";

// Manager view
pub const VIEW_TITLE: &str = "forgejo-view-title";
pub const VIEW_STATUS_RUNNING: &str = "forgejo-view-status-running";
pub const VIEW_STATUS_STOPPED: &str = "forgejo-view-status-stopped";
pub const VIEW_SERVICES_TAB: &str = "forgejo-view-services-tab";
pub const VIEW_CONFIG_TAB: &str = "forgejo-view-config-tab";

// Config labels (used in view.rs)
pub const CONFIG_DOMAIN_LABEL: &str = "forgejo-config-domain-label";
pub const CONFIG_SSH_PORT_LABEL: &str = "forgejo-config-ssh-port-label";
pub const CONFIG_OIDC_ISSUER_LABEL: &str = "forgejo-config-oidc-issuer-label";
pub const CONFIG_S3_LABEL: &str = "forgejo-config-s3-label";
pub const CONFIG_S3_DISABLED: &str = "forgejo-config-s3-disabled";
pub const CONFIG_NOT_SET: &str = "forgejo-config-not-set";

// Service labels
pub const SERVICE_ACTIVE_LABEL: &str = "forgejo-service-active-label";

// Sidebar navigation
pub const NAV_SETUP: &str = "forgejo-nav-setup";
pub const NAV_SERVICES: &str = "forgejo-nav-services";

// Error messages
pub const ERR_VALIDATION: &str = "forgejo-err-validation";
pub const ERR_CONFIG_WRITE: &str = "forgejo-err-config-write";
