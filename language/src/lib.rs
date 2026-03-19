// FreeSynergy Language Manager
//
// Two-layer locale model:
//   - Store default  : system-wide standard set by the node/admin
//   - Inventory      : per-user overrides in ~/.config/fsn/locale_settings.toml
//
// LanguageManager::effective_settings() merges both layers; inventory wins.
// All programs that need locale info import this crate instead of managing
// language state themselves.

pub mod git_contributor;

pub use git_contributor::{ContributorStatus, GitContributorCheck};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Language ──────────────────────────────────────────────────────────────────

/// A language entry as stored and used across all programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub id:           String,
    pub display_name: String,
    pub locale:       String,
}

impl Language {
    /// Returns the inline SVG flag for this language, or an empty string if unknown.
    pub fn flag_svg(&self) -> &'static str {
        match self.id.as_str() {
            "en" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 60 36" width="24" height="14"><rect width="60" height="36" fill="#012169"/><line x1="0" y1="0" x2="60" y2="36" stroke="#FFFFFF" stroke-width="12"/><line x1="60" y1="0" x2="0" y2="36" stroke="#FFFFFF" stroke-width="12"/><line x1="0" y1="0" x2="60" y2="36" stroke="#C8102E" stroke-width="6"/><line x1="60" y1="0" x2="0" y2="36" stroke="#C8102E" stroke-width="6"/><rect x="24" y="0" width="12" height="36" fill="#FFFFFF"/><rect x="0" y="12" width="60" height="12" fill="#FFFFFF"/><rect x="26" y="0" width="8" height="36" fill="#C8102E"/><rect x="0" y="14" width="60" height="8" fill="#C8102E"/></svg>"#,
            "de" => r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 3" width="24" height="14"><rect width="5" height="1" fill="#000000"/><rect y="1" width="5" height="1" fill="#DD0000"/><rect y="2" width="5" height="1" fill="#FFCE00"/></svg>"#,
            _ => "",
        }
    }
}

// ── DateFormat ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    /// DD.MM.YYYY — German/European style.
    #[default]
    DmY,
    /// MM/DD/YYYY — US style.
    MdY,
    /// YYYY-MM-DD — ISO 8601.
    Ymd,
}

impl DateFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DmY => "DD.MM.YYYY",
            Self::MdY => "MM/DD/YYYY",
            Self::Ymd => "YYYY-MM-DD (ISO)",
        }
    }

    pub fn example(&self) -> &'static str {
        match self {
            Self::DmY => "19.03.2026",
            Self::MdY => "03/19/2026",
            Self::Ymd => "2026-03-19",
        }
    }

    pub fn all() -> &'static [DateFormat] {
        &[Self::DmY, Self::MdY, Self::Ymd]
    }
}

// ── TimeFormat ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeFormat {
    /// 24-hour clock (14:30).
    #[default]
    H24,
    /// 12-hour clock (2:30 PM).
    H12,
}

impl TimeFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::H24 => "24h  (14:30)",
            Self::H12 => "12h  (2:30 PM)",
        }
    }
}

// ── NumberFormat ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumberFormat {
    /// 1.234,56 — German/European (dot = thousands, comma = decimal).
    #[default]
    EuropeDot,
    /// 1,234.56 — US/UK (comma = thousands, dot = decimal).
    UsComma,
    /// 1 234,56 — French/Swiss (space = thousands, comma = decimal).
    SpaceComma,
}

impl NumberFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::EuropeDot  => "1.234,56",
            Self::UsComma    => "1,234.56",
            Self::SpaceComma => "1 234,56",
        }
    }

    pub fn all() -> &'static [NumberFormat] {
        &[Self::EuropeDot, Self::UsComma, Self::SpaceComma]
    }
}

// ── LocaleSettings ────────────────────────────────────────────────────────────

/// All locale-related preferences. Every field is Option so the two layers
/// (Store default and Inventory override) can be merged cleanly.
///
/// Rule: **Store provides the default, Inventory can override any field.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LocaleSettings {
    /// Active interface language code (e.g. "de", "en").
    pub language: Option<String>,
    /// Fallback when a translation key is missing in the active language.
    pub fallback_language: Option<String>,
    /// Date display format.
    pub date_format: Option<DateFormat>,
    /// Clock format (24h / 12h).
    pub time_format: Option<TimeFormat>,
    /// Number and decimal separator format.
    pub number_format: Option<NumberFormat>,
    /// Automatically download new language pack updates from the Store.
    pub auto_update_packs: Option<bool>,
}

impl LocaleSettings {
    fn inventory_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("locale_settings.toml")
    }

    /// Load user inventory overrides from disk. Returns empty (all-None) if absent.
    pub fn load_inventory() -> Self {
        let content = std::fs::read_to_string(Self::inventory_path()).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Save user inventory overrides to disk.
    pub fn save_inventory(&self) -> Result<(), String> {
        let path = Self::inventory_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Merge two settings: `self` is the base layer, `other` overrides it for
    /// every field that is `Some` in `other`.
    pub fn merge_with(self, other: &LocaleSettings) -> LocaleSettings {
        LocaleSettings {
            language:          other.language.clone().or(self.language),
            fallback_language: other.fallback_language.clone().or(self.fallback_language),
            date_format:       other.date_format.clone().or(self.date_format),
            time_format:       other.time_format.clone().or(self.time_format),
            number_format:     other.number_format.clone().or(self.number_format),
            auto_update_packs: other.auto_update_packs.or(self.auto_update_packs),
        }
    }

    /// Resolve all Options into concrete values using built-in defaults for any
    /// field still None after merging.
    pub fn resolved(&self) -> ResolvedLocaleSettings {
        ResolvedLocaleSettings {
            language:          self.language.clone().unwrap_or_else(|| "en".into()),
            fallback_language: self.fallback_language.clone().unwrap_or_else(|| "en".into()),
            date_format:       self.date_format.clone().unwrap_or_default(),
            time_format:       self.time_format.clone().unwrap_or_default(),
            number_format:     self.number_format.clone().unwrap_or_default(),
            auto_update_packs: self.auto_update_packs.unwrap_or(true),
        }
    }
}

/// Fully resolved locale settings — all fields are concrete values.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLocaleSettings {
    pub language:          String,
    pub fallback_language: String,
    pub date_format:       DateFormat,
    pub time_format:       TimeFormat,
    pub number_format:     NumberFormat,
    pub auto_update_packs: bool,
}

// ── LanguageManager ───────────────────────────────────────────────────────────

/// Entry point for all language and locale operations.
pub struct LanguageManager;

impl LanguageManager {
    pub fn new() -> Self { Self }

    /// System-wide default locale settings, provided by the Store.
    ///
    /// TODO: fetch from StoreClient once the Store layer is implemented.
    ///       Currently returns a hardcoded default.
    pub fn store_defaults(&self) -> LocaleSettings {
        LocaleSettings {
            language:          Some("en".into()),
            fallback_language: Some("en".into()),
            date_format:       Some(DateFormat::DmY),
            time_format:       Some(TimeFormat::H24),
            number_format:     Some(NumberFormat::EuropeDot),
            auto_update_packs: Some(true),
        }
    }

    /// Per-user overrides from the Inventory.
    pub fn inventory_settings(&self) -> LocaleSettings {
        LocaleSettings::load_inventory()
    }

    /// Effective settings: Store defaults merged with Inventory overrides.
    pub fn effective_settings(&self) -> ResolvedLocaleSettings {
        self.store_defaults()
            .merge_with(&self.inventory_settings())
            .resolved()
    }

    /// Returns the currently active language.
    pub fn active(&self) -> Language {
        let code = self.effective_settings().language;
        Language {
            id:           code.clone(),
            display_name: code.clone(), // TODO: look up from LanguageMeta
            locale:       code,
        }
    }

    /// Returns all available languages.
    /// TODO: read from Store catalog.
    pub fn available(&self) -> Vec<Language> {
        vec![
            Language { id: "en".into(), display_name: "English".into(), locale: "en-US".into() },
            Language { id: "de".into(), display_name: "Deutsch".into(), locale: "de-DE".into() },
        ]
    }

    /// Sets the active language in the user Inventory.
    pub fn set_active(&self, id: &str) -> Result<(), LanguageError> {
        let mut inv = LocaleSettings::load_inventory();
        inv.language = Some(id.to_string());
        inv.save_inventory().map_err(LanguageError::StoreError)
    }

    /// Saves updated Inventory settings (partial update — only provided fields are stored).
    pub fn save_settings(&self, settings: LocaleSettings) -> Result<(), LanguageError> {
        settings.save_inventory().map_err(LanguageError::StoreError)
    }
}

impl Default for LanguageManager {
    fn default() -> Self { Self::new() }
}

// ── LanguageError ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum LanguageError {
    NotFound(String),
    PermissionDenied,
    StoreError(String),
}

impl std::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id)    => write!(f, "Language not found: {id}"),
            Self::PermissionDenied => write!(f, "Permission denied: cannot set language"),
            Self::StoreError(msg) => write!(f, "Store error: {msg}"),
        }
    }
}

impl std::error::Error for LanguageError {}
