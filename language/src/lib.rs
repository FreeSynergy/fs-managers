// FreeSynergy Language Manager
//
// Responsibilities:
//   - Read the active language from the Store
//   - Write a new active language to the Store (requires permission)
//   - Provide a UI picker component for language selection
//
// Programs that need language handling import this crate and call
// LanguageManager instead of managing language state themselves.
// Settings calls LanguageManager to render the language picker.

/// The active language selection, as stored and used across all programs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    pub id: String,
    pub display_name: String,
    pub locale: String,
}

/// Manages the active language for the FreeSynergy ecosystem.
pub struct LanguageManager;

impl LanguageManager {
    pub fn new() -> Self {
        Self
    }

    /// Returns the currently active language.
    pub fn active(&self) -> Language {
        // TODO: read from Store
        Language {
            id: "en".into(),
            display_name: "English".into(),
            locale: "en-US".into(),
        }
    }

    /// Returns all available languages.
    pub fn available(&self) -> Vec<Language> {
        // TODO: read from Store
        vec![
            Language { id: "en".into(), display_name: "English".into(), locale: "en-US".into() },
            Language { id: "de".into(), display_name: "Deutsch".into(), locale: "de-DE".into() },
        ]
    }

    /// Sets the active language. Requires Store write permission.
    pub fn set_active(&self, id: &str) -> Result<(), LanguageError> {
        // TODO: write to Store
        let _ = id;
        Ok(())
    }
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum LanguageError {
    NotFound(String),
    PermissionDenied,
    StoreError(String),
}

impl std::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Language not found: {id}"),
            Self::PermissionDenied => write!(f, "Permission denied: cannot set language"),
            Self::StoreError(msg) => write!(f, "Store error: {msg}"),
        }
    }
}

impl std::error::Error for LanguageError {}
