// view.rs — Bridge: FsView + ManagerLayout for CursorManager.
//
// This is the ONLY file in this crate that imports fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::{active, list, preview, CursorManager};

impl FsView for CursorManager {
    fn view(&self) -> Box<dyn FsWidget> {
        list::widget(self)
    }
}

impl ManagerLayout for CursorManager {
    fn title(&self) -> &'static str {
        "Cursor Manager"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "list",
                label: fs_i18n::t("managers-cursor-section-list").to_string(),
                icon: "🖱",
            },
            ManagerSidebarItem {
                id: "active",
                label: fs_i18n::t("managers-cursor-section-active").to_string(),
                icon: "✓",
            },
            ManagerSidebarItem {
                id: "preview",
                label: fs_i18n::t("managers-cursor-section-preview").to_string(),
                icon: "👁",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "list" => list::widget(self),
            "active" => active::widget(self),
            "preview" => preview::widget(self),
            _ => Box::new(ListWidget {
                id: "cursor-unknown-section".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
