// FreeSynergy Theme Manager
//
// Responsibilities:
//   - Read the active theme from the Store
//   - Write a new active theme to the Store (requires permission)
//   - Provide a UI picker component for theme selection
//
// Programs that need theming import this crate and call ThemeManager.
// Settings calls ThemeManager to render the theme picker.

use std::sync::Arc;

use fs_core::{FsManager, ManagerStore, NoopStore, SelectableManager};

/// A theme entry as stored and used across all programs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub id: String,
    pub display_name: String,
    pub is_dark: bool,
}

impl Theme {
    pub fn css_class(&self) -> &str {
        if self.is_dark { "fs-theme-dark" } else { "fs-theme-light" }
    }
}

/// Known built-in themes.
const BUILTIN_THEMES: &[(&str, &str, bool)] = &[
    ("fs-dark",  "FreeSynergy Dark",  true),
    ("fs-light", "FreeSynergy Light", false),
];

const DEFAULT_THEME_ID: &str = "fs-dark";

/// Manages the active theme for the FreeSynergy ecosystem.
pub struct ThemeManager {
    store: Arc<dyn ManagerStore>,
}

impl ThemeManager {
    /// Create a manager backed by `store`.
    pub fn new(store: Arc<dyn ManagerStore>) -> Self {
        Self { store }
    }

    /// Create a manager with a no-op store (test / offline use).
    pub fn with_noop() -> Self {
        Self::new(Arc::new(NoopStore))
    }

    /// Returns the currently active theme.
    pub fn active(&self) -> Theme {
        let id = self
            .store
            .read_setting("theme.active")
            .unwrap_or_else(|| DEFAULT_THEME_ID.into());
        self.find_by_id(&id).unwrap_or_else(|| self.default_theme())
    }

    /// Returns all available themes.
    pub fn available(&self) -> Vec<Theme> {
        BUILTIN_THEMES
            .iter()
            .map(|(id, name, dark)| Theme {
                id:           (*id).into(),
                display_name: (*name).into(),
                is_dark:      *dark,
            })
            .collect()
    }

    /// Sets the active theme. Requires Store write permission.
    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        if self.find_by_id(id).is_none() {
            return Err(ThemeError::NotFound(id.into()));
        }
        self.store
            .write_setting("theme.active", id)
            .map_err(|e| ThemeError::StoreError(e.to_string()))
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn find_by_id(&self, id: &str) -> Option<Theme> {
        BUILTIN_THEMES.iter().find(|(tid, _, _)| *tid == id).map(|(tid, name, dark)| Theme {
            id:           (*tid).into(),
            display_name: (*name).into(),
            is_dark:      *dark,
        })
    }

    fn default_theme(&self) -> Theme {
        Theme {
            id:           DEFAULT_THEME_ID.into(),
            display_name: "FreeSynergy Dark".into(),
            is_dark:      true,
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self { Self::with_noop() }
}

impl SelectableManager for ThemeManager {
    type Item  = Theme;
    type Error = ThemeError;

    fn active(&self)               -> Theme                   { ThemeManager::active(self) }
    fn available(&self)            -> Vec<Theme>              { ThemeManager::available(self) }
    fn set_active(&self, id: &str) -> Result<(), ThemeError>  { ThemeManager::set_active(self, id) }
}

impl FsManager for ThemeManager {
    fn id(&self)         -> &str { "theme" }
    fn name(&self)       -> &str { "Theme Manager" }
    fn is_healthy(&self) -> bool { true }
}

/// Error type for the Theme Manager — alias of the shared [`fs_core::ManagerError`].
pub type ThemeError = fs_core::ManagerError;
