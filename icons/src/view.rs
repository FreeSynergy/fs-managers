// view.rs — Bridge: FsView + ManagerLayout for IconManager.
//
// This is the ONLY file in this crate that imports fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::IconManager;

// ── List section ──────────────────────────────────────────────────────────────

fn list_widget(manager: &IconManager) -> Box<dyn FsWidget> {
    let items = manager
        .sets()
        .iter()
        .map(|s| format!("{} ({})", s.name, s.icon_count))
        .collect();
    Box::new(ListWidget {
        id: "icons-section-list".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for IconManager {
    fn view(&self) -> Box<dyn FsWidget> {
        list_widget(self)
    }
}

impl ManagerLayout for IconManager {
    fn title(&self) -> &'static str {
        "Icon Manager"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![ManagerSidebarItem {
            id: "list",
            label: fs_i18n::t("managers-icons-section-list").to_string(),
            icon: "🖼",
        }]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "list" => list_widget(self),
            _ => Box::new(ListWidget {
                id: "icons-unknown-section".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
