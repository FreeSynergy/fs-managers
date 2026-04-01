// download.rs — Download section widget.
//
// Shows subscribed languages and installed pack counts.
// Download actions will be added once the Store client is implemented.

use fs_render::{FsWidget, ListWidget};

use crate::LanguageManager;

/// Widget showing subscribed languages and available download status.
pub fn widget(manager: &LanguageManager) -> Box<dyn FsWidget> {
    let subscribed = manager.subscribed_languages();
    let packs = manager.installed_packs();

    let mut items = Vec::new();
    items.push("Subscribed languages:".into());

    if subscribed.is_empty() {
        items.push("  (none)".into());
    } else {
        for code in &subscribed {
            let count = packs.iter().filter(|p| &p.lang_code == code).count();
            items.push(format!("  {code} — {count} pack(s) installed"));
        }
    }

    items.push(String::new());
    items.push(format!(
        "Total installed codes: {}",
        manager.registry().installed_lang_codes().len()
    ));

    Box::new(ListWidget {
        id: "lang-section-download".into(),
        items,
        selected_index: None,
        enabled: true,
    })
}
