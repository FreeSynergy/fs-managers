// list.rs — Installed cursor sets list section widget.

use fs_render::{FsWidget, ListWidget};

use crate::CursorManager;

/// Widget listing all installed cursor sets.
#[must_use]
pub fn widget(manager: &CursorManager) -> Box<dyn FsWidget> {
    let sets = manager.sets();
    let items = if sets.is_empty() {
        vec![fs_i18n::t("managers-cursor-no-sets").to_string()]
    } else {
        sets.iter()
            .map(|s| {
                let complete = if s.is_complete() { "✓" } else { "⚠" };
                format!("{complete} {} v{}", s.name, s.version)
            })
            .collect()
    };
    Box::new(ListWidget {
        id: "cursor-section-list".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
