# FreeSynergy · Managers

**The glue between Store and programs**
by KalEl · Cyan + White

Managers are the connection layer between the FreeSynergy Store (what's available) and all programs (what they use). Instead of every program managing its own language dropdown, theme picker, or app list — it calls the relevant Manager.

## Principle

```
Store ←→ Manager ←→ Programs / UI
```

- A Manager reads the current state from the Store
- A Manager provides a ready-to-use UI component (e.g. language picker)
- A Manager writes changes back to the Store (with permission)
- Settings calls Managers — it has no logic of its own for these concerns
- Inventory receives the installed results

## Managers

| Crate | Name | Responsibility |
|---|---|---|
| `language/` | `fs-manager-language` | Active language — read, set, UI picker |
| `theme/` | `fs-manager-theme` | Active theme — read, set, UI picker |
| `container/` | `fs-manager-container-app` | Container apps — install, remove, start, stop (formerly Conductor) |
| `icons/` | `fs-manager-icons` | Icon sets — resolve paths, UI icon picker |

## Usage

Add the relevant manager as a dependency:

```toml
[dependencies]
fs-manager-language = { path = "../FreeSynergy.Managers/language" }
```

Then call the manager instead of building your own state:

```rust
use fs_manager_language::LanguageManager;

let mgr = LanguageManager::new();
let lang = mgr.active();          // current language
let all = mgr.available();        // all languages
mgr.set_active("de")?;            // change language → updates Store
```

## Context-awareness

Not every Manager makes sense everywhere. Icons in a terminal make no sense — but language does. Each program only includes the Managers it actually needs.

## License

MIT — see [LICENSE](LICENSE)
