// preview.rs — Format preview section widget.
//
// Shows concrete date / time / number examples rendered with the active locale settings.

use fs_render::{FsWidget, ListWidget};

use crate::LanguageManager;

/// Widget showing format examples for the active locale settings.
pub fn widget(manager: &LanguageManager) -> Box<dyn FsWidget> {
    let settings = manager.effective_settings();
    let items = vec![
        "Format preview:".into(),
        format!("  Date:     {}", settings.format_date(2026, 3, 19)),
        format!("  Time:     {}", settings.format_time(14, 30)),
        format!("  Integer:  {}", settings.format_integer(1_234_567)),
        format!("  Decimal:  {}", settings.format_decimal(1_234.567_8, 2)),
    ];
    Box::new(ListWidget {
        id: "lang-section-preview".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
