// FreeSynergy Theme Manager
//
// Responsibilities:
//   - Read the active theme from the Store
//   - Write a new active theme to the Store (requires permission)
//   - Provide a UI picker component for theme selection
//
// Programs that need theming import this crate and call ThemeManager.
// Settings calls ThemeManager to render the theme picker.

/// Common interface for all FreeSynergy managers.
pub trait FsManager {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn is_healthy(&self) -> bool;
}

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

/// Manages the active theme for the FreeSynergy ecosystem.
pub struct ThemeManager;

impl ThemeManager {
    pub fn new() -> Self {
        Self
    }

    /// Returns the currently active theme.
    pub fn active(&self) -> Theme {
        // TODO: read from Store
        Theme {
            id: "fs-dark".into(),
            display_name: "FreeSynergy Dark".into(),
            is_dark: true,
        }
    }

    /// Returns all available themes.
    pub fn available(&self) -> Vec<Theme> {
        // TODO: read from Store
        vec![
            Theme { id: "fs-dark".into(), display_name: "FreeSynergy Dark".into(), is_dark: true },
            Theme { id: "fs-light".into(), display_name: "FreeSynergy Light".into(), is_dark: false },
        ]
    }

    /// Sets the active theme. Requires Store write permission.
    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        // TODO: write to Store
        let _ = id;
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self { Self::new() }
}

impl FsManager for ThemeManager {
    fn id(&self)         -> &str { "theme" }
    fn name(&self)       -> &str { "Theme Manager" }
    fn is_healthy(&self) -> bool { true }
}

#[derive(Debug)]
pub enum ThemeError {
    NotFound(String),
    PermissionDenied,
    StoreError(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Theme not found: {id}"),
            Self::PermissionDenied => write!(f, "Permission denied: cannot set theme"),
            Self::StoreError(msg) => write!(f, "Store error: {msg}"),
        }
    }
}

impl std::error::Error for ThemeError {}
