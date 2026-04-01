// list.rs — Language list section widget.
//
// Shows all available languages. The active language is marked with a bullet.

use fs_render::{FsWidget, ListWidget};

use crate::LanguageManager;

/// Widget listing all available languages.
///
/// The currently active language is marked with `●`; others with two spaces.
pub fn widget(manager: &LanguageManager) -> Box<dyn FsWidget> {
    let active_id = manager.active().id;
    let available = manager.available();
    let selected_index = available.iter().position(|l| l.id == active_id);
    let items = available
        .iter()
        .map(|l| {
            if l.id == active_id {
                format!("● {} ({})", l.display_name, l.id)
            } else {
                format!("  {} ({})", l.display_name, l.id)
            }
        })
        .collect();
    Box::new(ListWidget {
        id: "lang-section-list".into(),
        items,
        selected_index,
        enabled: true,
    })
}
