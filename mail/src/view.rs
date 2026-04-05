// view.rs — FsView + ManagerLayout for StalwartSetupWizard.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain objects (wizard, config) do NOT import fs-render.

use fs_render::{
    FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem, ProgramView,
    ProgramViewProvider,
};

use crate::{wizard::StalwartSetupWizard, wizard::WizardStep};

// ── Wizard summary widget ──────────────────────────────────────────────────────

fn wizard_summary_widget(wizard: &StalwartSetupWizard) -> Box<dyn FsWidget> {
    let config = wizard.config();
    let tls_status = if config.tls.use_acme {
        fs_i18n::t("mail-wizard-tls-acme").to_string()
    } else if config.tls.is_configured() {
        fs_i18n::t("mail-wizard-tls-manual").to_string()
    } else {
        fs_i18n::t("mail-wizard-field-not-set").to_string()
    };

    let oidc_status = if config.skip_oidc {
        fs_i18n::t("mail-wizard-oidc-skipped").to_string()
    } else if config.oidc.is_configured() {
        config.oidc.issuer_url.clone()
    } else {
        fs_i18n::t("mail-wizard-field-not-set").to_string()
    };

    let items = vec![
        format!(
            "{}: {}",
            fs_i18n::t("mail-wizard-field-domain"),
            if config.domain.is_empty() {
                fs_i18n::t("mail-wizard-field-not-set").to_string()
            } else {
                config.domain.clone()
            }
        ),
        format!(
            "{}: {}",
            fs_i18n::t("mail-wizard-field-admin-email"),
            if config.admin_email.is_empty() {
                fs_i18n::t("mail-wizard-field-not-set").to_string()
            } else {
                config.admin_email.clone()
            }
        ),
        format!("{}: {}", fs_i18n::t("mail-wizard-field-tls"), tls_status),
        format!("{}: {}", fs_i18n::t("mail-wizard-field-oidc"), oidc_status),
        format!(
            "{}: {}",
            fs_i18n::t("mail-wizard-current-step"),
            if *wizard.step() == WizardStep::Done {
                fs_i18n::t("mail-wizard-step-done-title").to_string()
            } else {
                fs_i18n::t(wizard.step().title_key()).to_string()
            }
        ),
    ];

    Box::new(ListWidget {
        id: "mail-wizard-summary".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

fn dns_hints_widget() -> Box<dyn FsWidget> {
    let items = vec![
        fs_i18n::t("mail-wizard-dns-mx-hint").to_string(),
        fs_i18n::t("mail-wizard-dns-spf-hint").to_string(),
        fs_i18n::t("mail-wizard-dns-dkim-hint").to_string(),
        fs_i18n::t("mail-wizard-dns-dmarc-hint").to_string(),
        fs_i18n::t("mail-wizard-dns-note").to_string(),
    ];
    Box::new(ListWidget {
        id: "mail-wizard-dns-hints".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── Services tab widget ───────────────────────────────────────────────────────

fn services_widget() -> Box<dyn FsWidget> {
    let items = vec![
        format!(
            "Stalwart Mail  —  {}",
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
        id: "mail-services".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── ProgramViewProvider + FsView + ManagerLayout ─────────────────────────────

impl ProgramViewProvider for StalwartSetupWizard {
    fn available_views(&self) -> Vec<ProgramView> {
        vec![
            ProgramView::Info,
            ProgramView::Manual,
            ProgramView::SettingsContainer,
        ]
    }
}

impl FsView for StalwartSetupWizard {
    fn view(&self) -> Box<dyn FsWidget> {
        wizard_summary_widget(self)
    }
}

impl ManagerLayout for StalwartSetupWizard {
    fn title(&self) -> &'static str {
        "Stalwart Mail Setup"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "setup",
                label: fs_i18n::t("mail-wizard-nav-setup").to_string(),
                icon: "✉",
            },
            ManagerSidebarItem {
                id: "dns",
                label: fs_i18n::t("mail-wizard-nav-dns").to_string(),
                icon: "🌐",
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
            "dns" => dns_hints_widget(),
            "services" => services_widget(),
            _ => Box::new(ListWidget {
                id: "mail-wizard-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
