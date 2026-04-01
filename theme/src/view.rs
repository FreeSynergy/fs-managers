// view.rs — Bridge: FsView + ManagerLayout for ThemeManager.
//
// This is the ONLY file in this crate that imports fs-render.

use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::ThemeManager;

// ── List section ──────────────────────────────────────────────────────────────

fn list_widget(manager: &ThemeManager) -> Box<dyn FsWidget> {
    let active = manager.active();
    let items = manager
        .available()
        .iter()
        .map(|t| {
            let marker = if t.id == active.id { "●" } else { " " };
            let kind = if t.is_dark { "Dark" } else { "Light" };
            format!("{marker} {} [{}]", t.display_name, kind)
        })
        .collect();
    Box::new(ListWidget {
        id: "theme-section-list".into(),
        items,
        selected_index: manager.available().iter().position(|t| t.id == active.id),
        enabled: true,
    })
}

// ── Active section ────────────────────────────────────────────────────────────

fn active_widget(manager: &ThemeManager) -> Box<dyn FsWidget> {
    let active = manager.active();
    let items = vec![
        format!("Name:    {}", active.display_name),
        format!("ID:      {}", active.id),
        format!("Style:   {}", if active.is_dark { "Dark" } else { "Light" }),
        format!("CSS class: {}", active.css_class()),
    ];
    Box::new(ListWidget {
        id: "theme-section-active".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for ThemeManager {
    fn view(&self) -> Box<dyn FsWidget> {
        list_widget(self)
    }
}

impl ManagerLayout for ThemeManager {
    fn title(&self) -> &'static str {
        "Theme Manager"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "list",
                label: fs_i18n::t("managers-theme-section-list").to_string(),
                icon: "🎨",
            },
            ManagerSidebarItem {
                id: "active",
                label: fs_i18n::t("managers-theme-section-active").to_string(),
                icon: "✓",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "list" => list_widget(self),
            "active" => active_widget(self),
            _ => Box::new(ListWidget {
                id: "theme-unknown-section".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
