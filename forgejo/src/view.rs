// view.rs — FsView + ManagerLayout for ForgejoSetupWizard.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain objects (wizard, config, adapter) do NOT import fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::{
    keys,
    wizard::{ForgejoSetupWizard, WizardStep},
};

// ── Wizard summary widget ─────────────────────────────────────────────────────

fn wizard_summary_widget(wizard: &ForgejoSetupWizard) -> Box<dyn FsWidget> {
    let cfg = wizard.config();

    let s3_status = match &cfg.s3 {
        Some(s3) if s3.is_configured() => s3.endpoint.clone(),
        _ => fs_i18n::t(keys::CONFIG_S3_DISABLED).to_string(),
    };

    let oidc_status = if cfg.oidc.is_configured() {
        cfg.oidc.issuer_url.clone()
    } else {
        fs_i18n::t(keys::CONFIG_NOT_SET).to_string()
    };

    let items = vec![
        format!(
            "{}: {}",
            fs_i18n::t(keys::CONFIG_DOMAIN_LABEL),
            if cfg.domain.is_empty() {
                fs_i18n::t(keys::CONFIG_NOT_SET).to_string()
            } else {
                cfg.domain.clone()
            }
        ),
        format!(
            "{}: {}",
            fs_i18n::t(keys::CONFIG_SSH_PORT_LABEL),
            cfg.ssh_port
        ),
        format!(
            "{}: {}",
            fs_i18n::t(keys::CONFIG_OIDC_ISSUER_LABEL),
            oidc_status
        ),
        format!("{}: {}", fs_i18n::t(keys::CONFIG_S3_LABEL), s3_status),
        format!(
            "Step: {}",
            if *wizard.step() == WizardStep::Done {
                fs_i18n::t(keys::WIZARD_STEP_DONE_TITLE).to_string()
            } else {
                fs_i18n::t(wizard.step().title_key()).to_string()
            }
        ),
    ];

    Box::new(ListWidget {
        id: "forgejo-wizard-summary".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── Services tab widget ───────────────────────────────────────────────────────

fn services_widget() -> Box<dyn FsWidget> {
    let items = vec![
        format!(
            "{}  —  {}",
            "Forgejo",
            fs_i18n::t(keys::SERVICE_ACTIVE_LABEL)
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
        id: "forgejo-services".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for ForgejoSetupWizard {
    fn view(&self) -> Box<dyn FsWidget> {
        wizard_summary_widget(self)
    }
}

impl ManagerLayout for ForgejoSetupWizard {
    fn title(&self) -> &'static str {
        "Forgejo Setup"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "setup",
                label: fs_i18n::t(keys::NAV_SETUP).to_string(),
                icon: "🐙",
            },
            ManagerSidebarItem {
                id: "services",
                label: fs_i18n::t(keys::NAV_SERVICES).to_string(),
                icon: "⚙",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "setup" => wizard_summary_widget(self),
            "services" => services_widget(),
            _ => Box::new(ListWidget {
                id: "forgejo-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
