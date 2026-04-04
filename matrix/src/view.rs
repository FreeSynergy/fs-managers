// view.rs — FsView + ManagerLayout for TuwunelSetupWizard.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain objects (wizard, config) do NOT import fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::{wizard::TuwunelSetupWizard, wizard::WizardStep};

// ── Wizard summary widget ─────────────────────────────────────────────────────

fn wizard_summary_widget(wizard: &TuwunelSetupWizard) -> Box<dyn FsWidget> {
    let config = wizard.config();

    let tls_status = if config.tls.use_acme {
        fs_i18n::t("matrix-wizard-tls-acme").to_string()
    } else if config.tls.is_configured() {
        fs_i18n::t("matrix-wizard-tls-manual").to_string()
    } else {
        fs_i18n::t("matrix-wizard-field-not-set").to_string()
    };

    let oidc_status = if config.skip_oidc {
        fs_i18n::t("matrix-wizard-oidc-offline-only").to_string()
    } else if config.oidc.is_configured() {
        config.oidc.issuer_url.clone()
    } else {
        fs_i18n::t("matrix-wizard-field-not-set").to_string()
    };

    let federation_status = if config.federation_enabled {
        fs_i18n::t("matrix-wizard-federation-enabled").to_string()
    } else {
        fs_i18n::t("matrix-wizard-federation-disabled").to_string()
    };

    let items = vec![
        format!(
            "{}: {}",
            fs_i18n::t("matrix-wizard-field-server"),
            if config.server_name.is_empty() {
                fs_i18n::t("matrix-wizard-field-not-set").to_string()
            } else {
                config.server_name.clone()
            }
        ),
        format!(
            "{}: {}",
            fs_i18n::t("matrix-wizard-field-admin-email"),
            if config.admin_email.is_empty() {
                fs_i18n::t("matrix-wizard-field-not-set").to_string()
            } else {
                config.admin_email.clone()
            }
        ),
        format!("{}: {}", fs_i18n::t("matrix-wizard-field-tls"), tls_status),
        format!(
            "{}: {}",
            fs_i18n::t("matrix-wizard-field-oidc"),
            oidc_status
        ),
        format!(
            "{}: {}",
            fs_i18n::t("matrix-wizard-field-federation"),
            federation_status
        ),
        format!(
            "{}: {}",
            fs_i18n::t("matrix-wizard-current-step"),
            if *wizard.step() == WizardStep::Done {
                fs_i18n::t("matrix-wizard-step-done-title").to_string()
            } else {
                fs_i18n::t(wizard.step().title_key()).to_string()
            }
        ),
    ];

    Box::new(ListWidget {
        id: "matrix-wizard-summary".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

fn iam_info_widget() -> Box<dyn FsWidget> {
    let items = vec![
        fs_i18n::t("matrix-wizard-iam-kanidm-note").to_string(),
        fs_i18n::t("matrix-wizard-iam-oidc-required").to_string(),
        fs_i18n::t("matrix-wizard-iam-accounts-backed").to_string(),
        fs_i18n::t("matrix-wizard-iam-skip-warning").to_string(),
    ];
    Box::new(ListWidget {
        id: "matrix-wizard-iam-info".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── Services tab widget ───────────────────────────────────────────────────────

fn services_widget() -> Box<dyn FsWidget> {
    let items = vec![
        format!(
            "Tuwunel (Matrix)  —  {}",
            fs_i18n::t("manager-service-tab-primary")
        ),
        String::new(),
        format!(
            "[{}]  [{}]  [{}]",
            fs_i18n::t("manager-service-cmd-start"),
            fs_i18n::t("manager-service-cmd-stop"),
            fs_i18n::t("manager-service-cmd-restart"),
        ),
    ];
    Box::new(ListWidget {
        id: "matrix-services".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for TuwunelSetupWizard {
    fn view(&self) -> Box<dyn FsWidget> {
        wizard_summary_widget(self)
    }
}

impl ManagerLayout for TuwunelSetupWizard {
    fn title(&self) -> &'static str {
        "Tuwunel Matrix Setup"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "setup",
                label: fs_i18n::t("matrix-wizard-nav-setup").to_string(),
                icon: "💬",
            },
            ManagerSidebarItem {
                id: "iam",
                label: fs_i18n::t("matrix-wizard-nav-iam").to_string(),
                icon: "🔑",
            },
            ManagerSidebarItem {
                id: "services",
                label: fs_i18n::t("manager-service-tab-title").to_string(),
                icon: "⚙",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "setup" => wizard_summary_widget(self),
            "iam" => iam_info_widget(),
            "services" => services_widget(),
            _ => Box::new(ListWidget {
                id: "matrix-wizard-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
