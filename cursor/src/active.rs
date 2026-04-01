// active.rs — Active cursor set section widget.
//
// Shows completeness and all present slots for the active set.

use fs_render::{FsWidget, ListWidget};

use crate::CursorManager;

/// Widget describing the active cursor set.
///
/// The active set is the first set returned by [`CursorManager::sets`].
/// If no sets are installed the widget shows an empty-state message.
#[must_use]
pub fn widget(manager: &CursorManager) -> Box<dyn FsWidget> {
    let sets = manager.sets();
    let items = match sets.first() {
        None => vec![fs_i18n::t("managers-cursor-no-sets").to_string()],
        Some(set) => {
            let mut rows = vec![
                format!("Name:     {}", set.name),
                format!("Author:   {}", set.author),
                format!("Version:  {}", set.version),
                format!("Complete: {}", if set.is_complete() { "Yes" } else { "No" }),
                format!("Slots:    {}/{}", set.present_slots.len(), 31),
            ];
            let missing = set.missing_required();
            if !missing.is_empty() {
                rows.push(format!(
                    "Missing:  {}",
                    missing
                        .iter()
                        .map(|s| s.filename())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            rows
        }
    };
    Box::new(ListWidget {
        id: "cursor-section-active".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
