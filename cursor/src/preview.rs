// preview.rs — Cursor slot preview section widget.
//
// Lists all 31 standard cursor slots with their filenames and status.

use fs_render::{FsWidget, ListWidget};

use crate::{CursorManager, CursorSlot};

/// Widget showing all 31 standard cursor slots and their status in the active set.
#[must_use]
pub fn widget(manager: &CursorManager) -> Box<dyn FsWidget> {
    let sets = manager.sets();
    let present: Vec<CursorSlot> = sets
        .first()
        .map(|s| s.present_slots.clone())
        .unwrap_or_default();

    let items: Vec<String> = CursorSlot::all()
        .iter()
        .map(|slot| {
            let status = if present.contains(slot) { "✓" } else { "✗" };
            let required_marker = if CursorSlot::minimum_required().contains(slot) {
                " *"
            } else {
                "  "
            };
            format!("{status}{required_marker} {}", slot.filename())
        })
        .collect();

    Box::new(ListWidget {
        id: "cursor-section-preview".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
