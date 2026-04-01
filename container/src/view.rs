// view.rs — Bridge: FsView + ManagerLayout for ContainerManager.
//
// This is the ONLY file in this crate that imports fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::ContainerManager;

// ── List section ──────────────────────────────────────────────────────────────

fn list_widget(manager: &ContainerManager) -> Box<dyn FsWidget> {
    let items = manager
        .installed()
        .iter()
        .map(|c| format!("{} [{}]", c.id, c.status.label()))
        .collect();
    Box::new(ListWidget {
        id: "container-section-list".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for ContainerManager {
    fn view(&self) -> Box<dyn FsWidget> {
        list_widget(self)
    }
}

impl ManagerLayout for ContainerManager {
    fn title(&self) -> &'static str {
        "Container App Manager"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![ManagerSidebarItem {
            id: "list",
            label: fs_i18n::t("managers-container-section-list").to_string(),
            icon: "📦",
        }]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "list" => list_widget(self),
            _ => Box::new(ListWidget {
                id: "container-unknown-section".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::ContainerManager;
    use fs_render::{FsView, ManagerLayout};

    fn manager() -> ContainerManager {
        ContainerManager::with_noop()
    }

    #[test]
    fn fsx_view_produces_widget() {
        let w = manager().view();
        assert_eq!(w.widget_id(), "container-section-list");
    }

    #[test]
    fn manager_layout_title() {
        assert_eq!(manager().title(), "Container App Manager");
    }

    #[test]
    fn sidebar_has_list_item() {
        let items = manager().sidebar_items();
        assert!(!items.is_empty());
        assert_eq!(items[0].id, "list");
    }

    #[test]
    fn content_for_list() {
        let w = manager().content_for("list");
        assert_eq!(w.widget_id(), "container-section-list");
    }

    #[test]
    fn content_for_unknown_section() {
        let w = manager().content_for("nonexistent");
        assert_eq!(w.widget_id(), "container-unknown-section");
        assert!(!w.is_enabled());
    }
}
