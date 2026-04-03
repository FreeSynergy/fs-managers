// view.rs — FsView + ManagerLayout for KanidmSetupWizard.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain objects (wizard, config) do NOT import fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::{wizard::KanidmSetupWizard, wizard::WizardStep};

// ── Wizard summary widget ──────────────────────────────────────────────────────

fn wizard_summary_widget(wizard: &KanidmSetupWizard) -> Box<dyn FsWidget> {
    let config = wizard.config();
    let mut items = vec![
        format!(
            "{}: {}",
            fs_i18n::t("auth-wizard-field-domain"),
            if config.domain.is_empty() {
                fs_i18n::t("auth-wizard-field-not-set").to_string()
            } else {
                config.domain.clone()
            }
        ),
        format!(
            "{}: {}",
            fs_i18n::t("auth-wizard-field-admin"),
            if config.admin_username.is_empty() {
                fs_i18n::t("auth-wizard-field-not-set").to_string()
            } else {
                config.admin_username.clone()
            }
        ),
        format!(
            "{}: {}",
            fs_i18n::t("auth-wizard-field-oidc-clients"),
            config.oidc_clients.len()
        ),
    ];

    if *wizard.step() == WizardStep::Done {
        items.push(fs_i18n::t("auth-wizard-step-done-title").to_string());
    } else {
        items.push(format!(
            "{}: {}",
            fs_i18n::t("auth-wizard-current-step"),
            fs_i18n::t(wizard.step().title_key())
        ));
    }

    Box::new(ListWidget {
        id: "auth-wizard-summary".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

fn oidc_clients_widget(wizard: &KanidmSetupWizard) -> Box<dyn FsWidget> {
    let items = if wizard.config().oidc_clients.is_empty() {
        vec![fs_i18n::t("auth-wizard-oidc-none").to_string()]
    } else {
        wizard
            .config()
            .oidc_clients
            .iter()
            .map(|c| format!("{} → {}", c.id, c.redirect_uri))
            .collect()
    };
    Box::new(ListWidget {
        id: "auth-wizard-oidc-list".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for KanidmSetupWizard {
    fn view(&self) -> Box<dyn FsWidget> {
        wizard_summary_widget(self)
    }
}

impl ManagerLayout for KanidmSetupWizard {
    fn title(&self) -> &'static str {
        "Kanidm Setup"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "setup",
                label: fs_i18n::t("auth-wizard-nav-setup").to_string(),
                icon: "🔐",
            },
            ManagerSidebarItem {
                id: "oidc",
                label: fs_i18n::t("auth-wizard-nav-oidc").to_string(),
                icon: "🔗",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "setup" => wizard_summary_widget(self),
            "oidc" => oidc_clients_widget(self),
            _ => Box::new(ListWidget {
                id: "auth-wizard-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
