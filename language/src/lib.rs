// FreeSynergy Language Manager
//
// Two-layer locale model:
//   - Store default  : system-wide standard set by the node/admin
//   - Inventory      : per-user overrides in ~/.config/fsn/locale_settings.toml
//
// LanguageManager::effective_settings() merges both layers; inventory wins.
// All programs that need locale info import this crate instead of managing
// language state themselves.

pub mod git;
pub mod git_contributor;

pub use git_contributor::{ContributorStatus, GitContributorCheck};

use fs_core::{FsManager, SelectableManager};
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
    ///
    /// SVG data is embedded at compile time from `flags/<lang>.svg` via `include_str!`.
    /// To add a new flag: create `flags/<lang_id>.svg` and add an entry to `FLAG_REGISTRY`.
    pub fn flag_svg(&self) -> &'static str {
        flag_registry().get(self.id.as_str()).copied().unwrap_or("")
    }
}

// ── Flag Registry ─────────────────────────────────────────────────────────────

/// Returns the static map of language id → inline SVG flag.
///
/// Each SVG is embedded at compile time. Adding a new language flag requires
/// only a new file in `flags/` and a new entry here — no logic changes.
fn flag_registry() -> &'static std::collections::HashMap<&'static str, &'static str> {
    static REGISTRY: std::sync::OnceLock<std::collections::HashMap<&'static str, &'static str>> =
        std::sync::OnceLock::new();

    REGISTRY.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        m.insert("en", include_str!("../flags/en.svg").trim());
        m.insert("de", include_str!("../flags/de.svg").trim());
        m
    })
}

// ── FormatVariant ─────────────────────────────────────────────────────────────

/// Common interface for locale format enums (date, time, number).
///
/// Implemented by [`DateFormat`], [`TimeFormat`], and [`NumberFormat`] so that
/// UI pickers and iterators can handle all three uniformly.
pub trait FormatVariant: Sized + 'static {
    /// Short label shown in the settings UI, e.g. `"DD.MM.YYYY"`.
    fn label(&self) -> &'static str;

    /// Example value rendered with this format, e.g. `"19.03.2026"`.
    ///
    /// Returns an empty string if the format has no meaningful example.
    fn example(&self) -> &'static str { "" }

    /// All available variants in display order.
    fn all() -> &'static [Self];
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

impl FormatVariant for DateFormat {
    fn label(&self) -> &'static str {
        match self {
            Self::DmY => "DD.MM.YYYY",
            Self::MdY => "MM/DD/YYYY",
            Self::Ymd => "YYYY-MM-DD (ISO)",
        }
    }

    fn example(&self) -> &'static str {
        match self {
            Self::DmY => "19.03.2026",
            Self::MdY => "03/19/2026",
            Self::Ymd => "2026-03-19",
        }
    }

    fn all() -> &'static [DateFormat] {
        &[Self::DmY, Self::MdY, Self::Ymd]
    }
}

impl DateFormat {
    /// Formats a date according to this format variant.
    pub fn format(&self, year: i32, month: u32, day: u32) -> String {
        match self {
            Self::DmY => format!("{:02}.{:02}.{}", day, month, year),
            Self::MdY => format!("{:02}/{:02}/{}", month, day, year),
            Self::Ymd => format!("{}-{:02}-{:02}", year, month, day),
        }
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

impl FormatVariant for TimeFormat {
    fn label(&self) -> &'static str {
        match self {
            Self::H24 => "24h  (14:30)",
            Self::H12 => "12h  (2:30 PM)",
        }
    }

    fn example(&self) -> &'static str {
        match self {
            Self::H24 => "14:30",
            Self::H12 => "2:30 PM",
        }
    }

    fn all() -> &'static [TimeFormat] {
        &[Self::H24, Self::H12]
    }
}

impl TimeFormat {
    /// Formats an hour/minute pair according to this format variant.
    pub fn format(&self, hour: u32, minute: u32) -> String {
        match self {
            Self::H24 => format!("{:02}:{:02}", hour, minute),
            Self::H12 => {
                let (h, ampm) = match hour {
                    0       => (12, "AM"),
                    1..=11  => (hour, "AM"),
                    12      => (12, "PM"),
                    _       => (hour - 12, "PM"),
                };
                format!("{:02}:{:02} {}", h, minute, ampm)
            }
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

impl FormatVariant for NumberFormat {
    fn label(&self) -> &'static str {
        match self {
            Self::EuropeDot  => "1.234,56",
            Self::UsComma    => "1,234.56",
            Self::SpaceComma => "1 234,56",
        }
    }

    fn example(&self) -> &'static str { self.label() }

    fn all() -> &'static [NumberFormat] {
        &[Self::EuropeDot, Self::UsComma, Self::SpaceComma]
    }
}

impl NumberFormat {
    /// Returns the thousands separator character for this format.
    fn thousands_sep(&self) -> char {
        match self {
            Self::EuropeDot  => '.',
            Self::UsComma    => ',',
            Self::SpaceComma => ' ',
        }
    }

    /// Returns the decimal separator character for this format.
    fn decimal_sep(&self) -> char {
        match self {
            Self::EuropeDot  => ',',
            Self::UsComma    => '.',
            Self::SpaceComma => ',',
        }
    }

    /// Formats an integer with thousands separators.
    pub fn format_integer(&self, value: i64) -> String {
        let abs_str = value.unsigned_abs().to_string();
        let grouped = group_thousands(&abs_str, self.thousands_sep());
        if value < 0 { format!("-{}", grouped) } else { grouped }
    }

    /// Formats a float with thousands and decimal separators.
    pub fn format_decimal(&self, value: f64, decimal_places: usize) -> String {
        let raw = format!("{:.prec$}", value.abs(), prec = decimal_places);
        let mut parts = raw.splitn(2, '.');
        let int_part = parts.next().unwrap_or("0");
        let dec_part = parts.next().unwrap_or("");
        let grouped = group_thousands(int_part, self.thousands_sep());
        let sign = if value < 0.0 { "-" } else { "" };
        if decimal_places > 0 {
            format!("{}{}{}{}", sign, grouped, self.decimal_sep(), dec_part)
        } else {
            format!("{}{}", sign, grouped)
        }
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

impl ResolvedLocaleSettings {
    /// Formats year/month/day according to the user's date format preference.
    pub fn format_date(&self, year: i32, month: u32, day: u32) -> String {
        self.date_format.format(year, month, day)
    }

    /// Formats hour/minute according to the user's time format preference.
    pub fn format_time(&self, hour: u32, minute: u32) -> String {
        self.time_format.format(hour, minute)
    }

    /// Formats an integer with thousands separators.
    pub fn format_integer(&self, value: i64) -> String {
        self.number_format.format_integer(value)
    }

    /// Formats a float with thousands and decimal separators.
    pub fn format_decimal(&self, value: f64, decimal_places: usize) -> String {
        self.number_format.format_decimal(value, decimal_places)
    }
}

/// Groups decimal digit string into thousands blocks separated by `sep`.
fn group_thousands(digits: &str, sep: char) -> String {
    let sep_str = sep.to_string();
    let groups: Vec<&str> = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect();
    groups.join(&sep_str)
}

impl Language {
    /// Constructs a `Language` from a language code, using known display names and locales.
    pub fn from_code(code: &str) -> Language {
        let (display_name, locale) = match code {
            "en" => ("English",    "en-US"),
            "de" => ("Deutsch",    "de-DE"),
            "fr" => ("Français",   "fr-FR"),
            "es" => ("Español",    "es-ES"),
            "it" => ("Italiano",   "it-IT"),
            "pt" => ("Português",  "pt-PT"),
            "nl" => ("Nederlands", "nl-NL"),
            "pl" => ("Polski",     "pl-PL"),
            "ru" => ("Русский",    "ru-RU"),
            "ja" => ("日本語",     "ja-JP"),
            "zh" => ("中文",       "zh-CN"),
            "ko" => ("한국어",     "ko-KR"),
            "ar" => ("العربية",   "ar-SA"),
            other => (other, other),
        };
        Language {
            id:           code.to_string(),
            display_name: display_name.to_string(),
            locale:       locale.to_string(),
        }
    }
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
        Language::from_code(&code)
    }

    /// Returns the built-in languages.
    ///
    /// Note: user-installed language packs are tracked in the Desktop-side PackageRegistry.
    /// This list covers only languages that are always available without installation.
    pub fn available(&self) -> Vec<Language> {
        vec![
            Language::from_code("en"),
            Language::from_code("de"),
        ]
    }

    /// Sets the active language in the user Inventory.
    pub fn set_active(&self, id: &str) -> Result<(), LanguageError> {
        let mut inv = LocaleSettings::load_inventory();
        inv.language = Some(id.to_string());
        inv.save_inventory().map_err(fs_core::ManagerError::StoreError)
    }

    /// Saves updated Inventory settings (partial update — only provided fields are stored).
    pub fn save_settings(&self, settings: LocaleSettings) -> Result<(), LanguageError> {
        settings.save_inventory().map_err(fs_core::ManagerError::StoreError)
    }
}

impl Default for LanguageManager {
    fn default() -> Self { Self::new() }
}

impl SelectableManager for LanguageManager {
    type Item  = Language;
    type Error = LanguageError;

    fn active(&self)               -> Language         { LanguageManager::active(self) }
    fn available(&self)            -> Vec<Language>    { LanguageManager::available(self) }
    fn set_active(&self, id: &str) -> Result<(), LanguageError> { LanguageManager::set_active(self, id) }
}

impl FsManager for LanguageManager {
    fn id(&self)   -> &str { "language" }
    fn name(&self) -> &str { "Language Manager" }
}

// ── LanguageError ─────────────────────────────────────────────────────────────

/// Error type for the Language Manager — alias of the shared [`fs_core::ManagerError`].
pub type LanguageError = fs_core::ManagerError;
