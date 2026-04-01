// active.rs — Active language section widget.
//
// Shows the currently active language and all locale format settings.

use fs_render::{FsWidget, ListWidget};

use crate::{FormatVariant, LanguageManager};

/// Widget showing the active language and locale format settings.
pub fn widget(manager: &LanguageManager) -> Box<dyn FsWidget> {
    let settings = manager.effective_settings();
    let active = manager.active();
    let items = vec![
        format!("Language:       {} ({})", active.display_name, active.id),
        format!("Direction:      {}", active.direction_label()),
        format!("Date format:    {}", settings.date_format.label()),
        format!("Time format:    {}", settings.time_format.label()),
        format!("Number format:  {}", settings.number_format.label()),
        format!(
            "Auto-update:    {}",
            if settings.auto_update_packs {
                "Yes"
            } else {
                "No"
            }
        ),
    ];
    Box::new(ListWidget {
        id: "lang-section-active".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
