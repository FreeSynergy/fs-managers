// view.rs — FsView + ManagerLayout for ZentinelManager.
//
// This is the ONLY file in this crate that imports fs-render.

use fs_core::FsManager;
use fs_render::{FsView, FsWidget, ListWidget, ManagerLayout, ManagerSidebarItem};

use crate::manager::ZentinelManager;

// ── Route list widget ─────────────────────────────────────────────────────────

fn route_list_widget(manager: &ZentinelManager) -> Box<dyn FsWidget> {
    let items = if manager.routes().is_empty() {
        vec![fs_i18n::t("zentinel-no-routes").to_string()]
    } else {
        manager
            .routes()
            .iter()
            .map(|r| format!("{} {} → {}", r.id, r.path, r.upstream))
            .collect()
    };
    Box::new(ListWidget {
        id: "zentinel-route-list".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

fn status_widget(manager: &ZentinelManager) -> Box<dyn FsWidget> {
    let items = vec![
        format!(
            "{}: {}",
            fs_i18n::t("zentinel-field-control-plane"),
            manager.control_plane_url()
        ),
        format!(
            "{}: {}",
            fs_i18n::t("zentinel-field-route-count"),
            manager.route_count()
        ),
        format!(
            "{}: {}",
            fs_i18n::t("zentinel-field-health"),
            if manager.is_healthy() {
                fs_i18n::t("zentinel-status-healthy").to_string()
            } else {
                fs_i18n::t("zentinel-status-unreachable").to_string()
            }
        ),
    ];
    Box::new(ListWidget {
        id: "zentinel-status".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}

// ── FsView + ManagerLayout ────────────────────────────────────────────────────

impl FsView for ZentinelManager {
    fn view(&self) -> Box<dyn FsWidget> {
        route_list_widget(self)
    }
}

impl ManagerLayout for ZentinelManager {
    fn title(&self) -> &'static str {
        "Zentinel"
    }

    fn sidebar_items(&self) -> Vec<ManagerSidebarItem> {
        vec![
            ManagerSidebarItem {
                id: "routes",
                label: fs_i18n::t("zentinel-nav-routes").to_string(),
                icon: "🔀",
            },
            ManagerSidebarItem {
                id: "status",
                label: fs_i18n::t("zentinel-nav-status").to_string(),
                icon: "📡",
            },
        ]
    }

    fn content_for(&self, item_id: &str) -> Box<dyn FsWidget> {
        match item_id {
            "routes" => route_list_widget(self),
            "status" => status_widget(self),
            _ => Box::new(ListWidget {
                id: "zentinel-unknown".into(),
                items: vec![format!("Unknown section: {item_id}")],
                selected_index: None,
                enabled: false,
            }),
        }
    }
}
