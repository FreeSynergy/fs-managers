// view.rs — FsView + ManagerLayout for TelegramSetupWizard.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain objects (wizard) do NOT import fs-render.

use fs_channel_telegram::keys;
use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::wizard::TelegramSetupWizard;

// ── Wizard summary widget ─────────────────────────────────────────────────────

fn wizard_summary_widget(wizard: &TelegramSetupWizard) -> Box<dyn FsWidget> {
    let cfg = wizard.config();

    let chats_value = if cfg.allowed_chat_ids.is_empty() {
        fs_i18n::t(keys::CONFIG_CHATS_ALL).to_string()
    } else {
        cfg.allowed_chat_ids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };

    let items = vec![
        format!(
            "{}: {}",
            fs_i18n::t(keys::CONFIG_TOKEN_REF_LABEL),
            cfg.bot_token_ref
        ),
        format!("{}: {}", fs_i18n::t(keys::CONFIG_CHATS_LABEL), chats_value),
        format!("Step: {}", fs_i18n::t(wizard.step().title_key())),
    ];

    Box::new(ListWidget {
        id: "telegram-wizard-summary".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for TelegramSetupWizard {
    fn view(&self) -> Box<dyn FsWidget> {
        wizard_summary_widget(self)
    }
}

impl ManagerLayout for TelegramSetupWizard {
    fn title(&self) -> &'static str {
        "Telegram Channel Setup"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![ManagerSidebarItem {
            id: "setup",
            label: fs_i18n::t(keys::WIZARD_TITLE).to_string(),
            icon: "✈",
        }]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "setup" => wizard_summary_widget(self),
            _ => Box::new(ListWidget {
                id: "telegram-wizard-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
