// view.rs — Bridge: FsView + ManagerLayout for LanguageManager.
//
// This is the ONLY file in this crate that imports fs-render.
// Domain types in lib.rs remain render-agnostic.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::{active, download, list, preview, LanguageManager};

impl FsView for LanguageManager {
    /// Overview widget: the language list (used when no sidebar section is selected).
    fn view(&self) -> Box<dyn FsWidget> {
        list::widget(self)
    }
}

impl ManagerLayout for LanguageManager {
    fn title(&self) -> &'static str {
        "Language Manager"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "list",
                label: fs_i18n::t("managers-language-section-list").to_string(),
                icon: "🌐",
            },
            ManagerSidebarItem {
                id: "active",
                label: fs_i18n::t("managers-language-section-active").to_string(),
                icon: "✓",
            },
            ManagerSidebarItem {
                id: "download",
                label: fs_i18n::t("managers-language-section-download").to_string(),
                icon: "⬇",
            },
            ManagerSidebarItem {
                id: "preview",
                label: fs_i18n::t("managers-language-section-preview").to_string(),
                icon: "👁",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "list" => list::widget(self),
            "active" => active::widget(self),
            "download" => download::widget(self),
            "preview" => preview::widget(self),
            _ => Box::new(ListWidget {
                id: "lang-unknown-section".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
