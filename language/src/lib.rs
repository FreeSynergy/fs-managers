#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_literal_bound)]
// FreeSynergy Language Manager
//
// Three-layer locale model:
//   - Store default        : system-wide standard set by the node/admin
//   - Inventory override   : per-user settings in ~/.config/fsn/locale_settings.toml
//   - LanguagePackRegistry : installed packs tracked in ~/.local/share/fs/i18n/registry.toml
//
// Domain types:
//   Language             — a language the user can activate (id, display_name, locale)
//   InstalledLanguagePack — one .toml file on disk for (lang_code, package_id)
//   LanguagePackRegistry  — registry of all installed packs; load/save/query
//   LocaleSettings        — all user preferences (Option fields for two-layer merge)
//   ResolvedLocaleSettings — fully resolved (all Options filled with defaults)
//
// OOP principles:
//   - HasFlag trait      : flag_svg() on any type that represents a language
//   - FormatVariant trait: label() + example() + all() on DateFormat/TimeFormat/NumberFormat
//   - No match blocks outside format() methods — types carry their own behavior

pub mod git;
pub mod git_contributor;

pub use git_contributor::{ContributorStatus, GitContributorCheck};

use fs_core::{FsManager, SelectableManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

// ── HasFlag ───────────────────────────────────────────────────────────────────

/// Provides an inline SVG flag icon for a type that represents a language.
pub trait HasFlag {
    /// Returns the inline SVG markup for this language's flag.
    /// Empty string if no flag is available.
    fn flag_svg(&self) -> &'static str;
}

// ── Language ──────────────────────────────────────────────────────────────────

/// A language entry as stored and used across all programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    /// ISO 639-1 code (e.g. "de", "en", "fr").
    pub id: String,
    /// Native name: "Deutsch", "Français", "English".
    pub display_name: String,
    /// BCP-47 locale: "de-DE", "fr-FR", "en-US".
    pub locale: String,
}

impl Language {
    /// Build a `Language` from an ISO 639-1 code.
    ///
    /// Uses the `LanguageMeta` registry from `fs-i18n` (50 languages) for native
    /// names.  Falls back to using the code itself when the code is not known.
    pub fn from_code(code: &str) -> Language {
        let meta = fs_i18n::language_meta(code);
        Language {
            id: code.to_string(),
            display_name: meta.map_or(code, |m| m.native_name).to_string(),
            locale: locale_for_code(code),
        }
    }

    /// Return the full `LanguageMeta` for this language, if available.
    ///
    /// Provides direction, script, family, continent and other metadata from
    /// `fs-i18n`'s embedded `languages.toml`.
    pub fn meta(&self) -> Option<&'static fs_i18n::LanguageMeta> {
        fs_i18n::language_meta(&self.id)
    }

    /// Human-readable text direction label ("Left-to-right" / "Right-to-left").
    pub fn direction_label(&self) -> &'static str {
        if self.meta().is_some_and(fs_i18n::LanguageMeta::is_rtl) {
            "Right-to-left"
        } else {
            "Left-to-right"
        }
    }
}

impl HasFlag for Language {
    fn flag_svg(&self) -> &'static str {
        flag_for_code(&self.id)
    }
}

// ── Flag registry (private) ───────────────────────────────────────────────────

/// Returns the inline SVG for the given language code, or `""` if unknown.
fn flag_for_code(code: &str) -> &'static str {
    static REGISTRY: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("en", include_str!("../flags/en.svg").trim());
            m.insert("de", include_str!("../flags/de.svg").trim());
            m
        })
        .get(code)
        .copied()
        .unwrap_or("")
}

/// Derive a BCP-47 locale string from an ISO 639-1 code.
fn locale_for_code(code: &str) -> String {
    match code {
        "en" => "en-US",
        "zh" => "zh-CN",
        "pt" => "pt-PT",
        "ar" => "ar-SA",
        "ko" => "ko-KR",
        "ja" => "ja-JP",
        "yue" => "yue-HK",
        _ => return format!("{}-{}", code, code.to_uppercase()),
    }
    .to_string()
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
    fn example(&self) -> &'static str {
        ""
    }

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
    pub fn format(&self, year: i32, month: u32, day: u32) -> String {
        match self {
            Self::DmY => format!("{day:02}.{month:02}.{year}"),
            Self::MdY => format!("{month:02}/{day:02}/{year}"),
            Self::Ymd => format!("{year}-{month:02}-{day:02}"),
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
    pub fn format(&self, hour: u32, minute: u32) -> String {
        match self {
            Self::H24 => format!("{hour:02}:{minute:02}"),
            Self::H12 => {
                let (h, ampm) = match hour {
                    0 => (12, "AM"),
                    1..=11 => (hour, "AM"),
                    12 => (12, "PM"),
                    _ => (hour - 12, "PM"),
                };
                format!("{h:02}:{minute:02} {ampm}")
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
            Self::EuropeDot => "1.234,56",
            Self::UsComma => "1,234.56",
            Self::SpaceComma => "1 234,56",
        }
    }

    fn example(&self) -> &'static str {
        self.label()
    }

    fn all() -> &'static [NumberFormat] {
        &[Self::EuropeDot, Self::UsComma, Self::SpaceComma]
    }
}

impl NumberFormat {
    fn thousands_sep(&self) -> char {
        match self {
            Self::EuropeDot => '.',
            Self::UsComma => ',',
            Self::SpaceComma => ' ',
        }
    }

    fn decimal_sep(&self) -> char {
        match self {
            Self::EuropeDot | Self::SpaceComma => ',',
            Self::UsComma => '.',
        }
    }

    pub fn format_integer(&self, value: i64) -> String {
        let abs_str = value.unsigned_abs().to_string();
        let grouped = group_thousands(&abs_str, self.thousands_sep());
        if value < 0 {
            format!("-{grouped}")
        } else {
            grouped
        }
    }

    pub fn format_decimal(&self, value: f64, decimal_places: usize) -> String {
        let raw = format!("{:.prec$}", value.abs(), prec = decimal_places);
        let mut parts = raw.splitn(2, '.');
        let int_part = parts.next().unwrap_or("0");
        let dec_part = parts.next().unwrap_or("");
        let grouped = group_thousands(int_part, self.thousands_sep());
        let sign = if value < 0.0 { "-" } else { "" };
        let dec = self.decimal_sep();
        if decimal_places > 0 {
            format!("{sign}{grouped}{dec}{dec_part}")
        } else {
            format!("{sign}{grouped}")
        }
    }
}

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

// ── InstalledLanguagePack ─────────────────────────────────────────────────────

/// One installed language pack on disk.
///
/// A pack is identified by `(lang_code, package_id)` — e.g. `("de", "fs-container-app")`.
/// The `package_id` `"common"` refers to the shared system-wide strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledLanguagePack {
    /// ISO 639-1 language code (e.g. "de", "fr").
    pub lang_code: String,
    /// Package this translation belongs to (e.g. "fs-container-app", "common").
    pub package_id: String,
    /// Pack version string.
    pub version: String,
    /// Absolute path to the `.toml` translation file on disk.
    pub file_path: PathBuf,
}

// ── LanguagePackRegistry ──────────────────────────────────────────────────────

/// Registry of all installed language packs.
///
/// Persisted to `~/.local/share/fs/i18n/registry.toml`.
/// Callers load it via [`LanguagePackRegistry::load`], mutate it, then
/// call [`LanguagePackRegistry::save`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LanguagePackRegistry {
    pub packs: Vec<InstalledLanguagePack>,
}

impl LanguagePackRegistry {
    fn registry_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("fs")
            .join("i18n")
            .join("registry.toml")
    }

    /// Load the registry from disk. Returns an empty registry if absent.
    pub fn load() -> Self {
        let content = std::fs::read_to_string(Self::registry_path()).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Persist the registry to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Return all packs for `lang_code`.
    pub fn packs_for_lang(&self, lang: &str) -> Vec<&InstalledLanguagePack> {
        self.packs.iter().filter(|p| p.lang_code == lang).collect()
    }

    /// Return all packs for `package_id`.
    pub fn packs_for_package(&self, package_id: &str) -> Vec<&InstalledLanguagePack> {
        self.packs
            .iter()
            .filter(|p| p.package_id == package_id)
            .collect()
    }

    /// Return `true` if the given `(lang_code, package_id)` pair is installed.
    pub fn is_installed(&self, lang: &str, package_id: &str) -> bool {
        self.packs
            .iter()
            .any(|p| p.lang_code == lang && p.package_id == package_id)
    }

    /// Return all distinct language codes that have at least one installed pack.
    pub fn installed_lang_codes(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        self.packs
            .iter()
            .filter(|p| seen.insert(p.lang_code.clone()))
            .map(|p| p.lang_code.clone())
            .collect()
    }

    /// Register a pack — replaces any existing entry for the same (lang, package_id).
    pub fn register(&mut self, pack: InstalledLanguagePack) {
        if let Some(pos) = self
            .packs
            .iter()
            .position(|p| p.lang_code == pack.lang_code && p.package_id == pack.package_id)
        {
            self.packs[pos] = pack;
        } else {
            self.packs.push(pack);
        }
    }

    /// Remove a pack for the given (lang, package_id).
    pub fn unregister(&mut self, lang: &str, package_id: &str) {
        self.packs
            .retain(|p| !(p.lang_code == lang && p.package_id == package_id));
    }
}

// ── LocaleSettings ────────────────────────────────────────────────────────────

/// All locale-related preferences.
///
/// Every field is `Option` so the two layers (Store default + Inventory override)
/// can be merged cleanly with [`merge_with`](Self::merge_with).
///
/// Rule: **Store provides the default, Inventory can override any field.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LocaleSettings {
    /// Active interface language code (e.g. "de", "en").
    pub language: Option<String>,
    /// Fallback when a translation key is missing in the active language.
    pub fallback_language: Option<String>,
    /// Languages the user has subscribed to — packs are downloaded for all of them.
    pub subscribed_languages: Option<Vec<String>>,
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
        PathBuf::from(home)
            .join(".config")
            .join("fsn")
            .join("locale_settings.toml")
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

    /// Merge two settings: `self` is the base layer, `other` overrides it.
    ///
    /// For every `Some` field in `other`, that value wins.
    /// For `subscribed_languages`, the two lists are unioned (no deduplication loss).
    #[must_use]
    pub fn merge_with(self, other: &LocaleSettings) -> LocaleSettings {
        LocaleSettings {
            language: other.language.clone().or(self.language),
            fallback_language: other.fallback_language.clone().or(self.fallback_language),
            subscribed_languages: {
                match (
                    self.subscribed_languages,
                    other.subscribed_languages.clone(),
                ) {
                    (Some(base), Some(over)) => {
                        let mut combined = base;
                        for code in over {
                            if !combined.contains(&code) {
                                combined.push(code);
                            }
                        }
                        Some(combined)
                    }
                    (base, over) => over.or(base),
                }
            },
            date_format: other.date_format.clone().or(self.date_format),
            time_format: other.time_format.clone().or(self.time_format),
            number_format: other.number_format.clone().or(self.number_format),
            auto_update_packs: other.auto_update_packs.or(self.auto_update_packs),
        }
    }

    /// Resolve all `Option`s into concrete values using built-in defaults.
    pub fn resolved(&self) -> ResolvedLocaleSettings {
        ResolvedLocaleSettings {
            language: self.language.clone().unwrap_or_else(|| "en".into()),
            fallback_language: self
                .fallback_language
                .clone()
                .unwrap_or_else(|| "en".into()),
            subscribed_languages: self
                .subscribed_languages
                .clone()
                .unwrap_or_else(|| vec!["en".into(), "de".into()]),
            date_format: self.date_format.clone().unwrap_or_default(),
            time_format: self.time_format.clone().unwrap_or_default(),
            number_format: self.number_format.clone().unwrap_or_default(),
            auto_update_packs: self.auto_update_packs.unwrap_or(true),
        }
    }
}

// ── ResolvedLocaleSettings ────────────────────────────────────────────────────

/// Fully resolved locale settings — all fields are concrete values.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLocaleSettings {
    pub language: String,
    pub fallback_language: String,
    /// All languages the user has subscribed to (packs downloaded for each).
    pub subscribed_languages: Vec<String>,
    pub date_format: DateFormat,
    pub time_format: TimeFormat,
    pub number_format: NumberFormat,
    pub auto_update_packs: bool,
}

impl ResolvedLocaleSettings {
    pub fn format_date(&self, year: i32, month: u32, day: u32) -> String {
        self.date_format.format(year, month, day)
    }

    pub fn format_time(&self, hour: u32, minute: u32) -> String {
        self.time_format.format(hour, minute)
    }

    pub fn format_integer(&self, value: i64) -> String {
        self.number_format.format_integer(value)
    }

    pub fn format_decimal(&self, value: f64, decimal_places: usize) -> String {
        self.number_format.format_decimal(value, decimal_places)
    }
}

// ── LanguageManager ───────────────────────────────────────────────────────────

/// Entry point for all language and locale operations.
///
/// Combines Store defaults (what the server/admin provides) with per-user
/// Inventory overrides.  All reads go through `effective_settings()`.
pub struct LanguageManager;

impl LanguageManager {
    pub fn new() -> Self {
        Self
    }

    // ── Settings ──────────────────────────────────────────────────────────────

    /// System-wide default locale settings, provided by the Store.
    ///
    /// TODO: fetch from StoreClient once the Store layer is implemented.
    pub fn store_defaults(&self) -> LocaleSettings {
        LocaleSettings {
            language: Some("en".into()),
            fallback_language: Some("en".into()),
            subscribed_languages: Some(vec!["en".into(), "de".into()]),
            date_format: Some(DateFormat::DmY),
            time_format: Some(TimeFormat::H24),
            number_format: Some(NumberFormat::EuropeDot),
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

    /// Save updated Inventory settings (partial update — only `Some` fields are stored).
    pub fn save_settings(&self, settings: &LocaleSettings) -> Result<(), LanguageError> {
        settings
            .save_inventory()
            .map_err(fs_core::ManagerError::StoreError)
    }

    // ── Active language ───────────────────────────────────────────────────────

    /// Returns the currently active language.
    pub fn active(&self) -> Language {
        Language::from_code(&self.effective_settings().language)
    }

    /// Sets the active language in the user Inventory.
    pub fn set_active(&self, id: &str) -> Result<(), LanguageError> {
        let mut inv = LocaleSettings::load_inventory();
        inv.language = Some(id.to_string());
        inv.save_inventory()
            .map_err(fs_core::ManagerError::StoreError)
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────

    /// Returns the list of language codes the user has subscribed to.
    ///
    /// Packs are downloaded for every subscribed language whenever a new
    /// package is installed.
    pub fn subscribed_languages(&self) -> Vec<String> {
        self.effective_settings().subscribed_languages
    }

    /// Subscribe to a language — its packs will be downloaded for all installed packages.
    ///
    /// No-op if already subscribed.  After subscribing, call
    /// [`download_for_language`](Self::download_for_language) to fetch packs.
    pub fn subscribe(&self, lang_code: &str) -> Result<(), LanguageError> {
        let mut inv = LocaleSettings::load_inventory();
        let mut codes = inv.subscribed_languages.unwrap_or_default();
        if !codes.contains(&lang_code.to_string()) {
            codes.push(lang_code.to_string());
        }
        inv.subscribed_languages = Some(codes);
        inv.save_inventory()
            .map_err(fs_core::ManagerError::StoreError)
    }

    /// Unsubscribe from a language.
    ///
    /// Already-downloaded packs remain on disk; they are just no longer
    /// downloaded automatically for future package installs.
    pub fn unsubscribe(&self, lang_code: &str) -> Result<(), LanguageError> {
        let mut inv = LocaleSettings::load_inventory();
        let codes = inv
            .subscribed_languages
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.as_str() != lang_code)
            .collect();
        inv.subscribed_languages = Some(codes);
        inv.save_inventory()
            .map_err(fs_core::ManagerError::StoreError)
    }

    // ── Registry ──────────────────────────────────────────────────────────────

    /// Load the full pack registry from disk.
    pub fn registry(&self) -> LanguagePackRegistry {
        LanguagePackRegistry::load()
    }

    /// Return all installed language packs.
    pub fn installed_packs(&self) -> Vec<InstalledLanguagePack> {
        LanguagePackRegistry::load().packs
    }

    /// Return `true` if the given `(lang_code, package_id)` pair is installed.
    pub fn is_installed(&self, lang: &str, package_id: &str) -> bool {
        LanguagePackRegistry::load().is_installed(lang, package_id)
    }

    // ── Download (Store integration — stubs) ──────────────────────────────────

    /// Download language packs for all subscribed languages for `package_id`.
    ///
    /// Called automatically when a new package is installed via `package.installed`
    /// Bus event.
    ///
    /// TODO: Implement Store client integration.
    pub fn download_for_package(&self, _package_id: &str) -> Result<(), LanguageError> {
        // TODO: for each subscribed language, fetch the matching pack from the Store
        //       and register it via LanguagePackRegistry::register().
        Ok(())
    }

    /// Download language packs for all installed packages for `lang_code`.
    ///
    /// Called when the user subscribes to a new language so existing packages
    /// get translated immediately.
    ///
    /// TODO: Implement Store client integration.
    pub fn download_for_language(&self, _lang_code: &str) -> Result<(), LanguageError> {
        // TODO: for each installed package, fetch the matching pack from the Store
        //       and register it via LanguagePackRegistry::register().
        Ok(())
    }

    // ── I18n loading ──────────────────────────────────────────────────────────

    /// Load all installed language packs for `package_id` into `i18n`.
    ///
    /// Call this at program startup after `fs_i18n::init_with_builtins()` to
    /// layer user-installed translations on top of the built-in snippets.
    pub fn load_into_i18n(&self, package_id: &str, i18n: &mut fs_i18n::I18n) {
        let registry = LanguagePackRegistry::load();
        for pack in registry.packs.iter().filter(|p| p.package_id == package_id) {
            if let Ok(src) = std::fs::read_to_string(&pack.file_path) {
                let _ = i18n.add_toml_str(&pack.lang_code, &src);
            }
        }
    }

    // ── Available languages ───────────────────────────────────────────────────

    /// All languages available for activation.
    ///
    /// Includes: all subscribed languages + all languages with installed packs +
    /// built-in "en" (always present).  Deduplicated.
    pub fn available(&self) -> Vec<Language> {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut languages = Vec::new();

        // 1. Active language first (always reachable)
        let active_code = self.effective_settings().language;
        if seen.insert(active_code.clone()) {
            languages.push(Language::from_code(&active_code));
        }

        // 2. Subscribed languages
        for code in self.subscribed_languages() {
            if seen.insert(code.clone()) {
                languages.push(Language::from_code(&code));
            }
        }

        // 3. Languages from installed packs
        for code in LanguagePackRegistry::load().installed_lang_codes() {
            if seen.insert(code.clone()) {
                languages.push(Language::from_code(&code));
            }
        }

        // 4. "en" is always available (built-in snippets)
        if seen.insert("en".to_string()) {
            languages.push(Language::from_code("en"));
        }

        languages
    }
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectableManager for LanguageManager {
    type Item = Language;
    type Error = LanguageError;

    fn active(&self) -> Language {
        LanguageManager::active(self)
    }
    fn available(&self) -> Vec<Language> {
        LanguageManager::available(self)
    }
    fn set_active(&self, id: &str) -> Result<(), LanguageError> {
        LanguageManager::set_active(self, id)
    }
}

impl FsManager for LanguageManager {
    fn id(&self) -> &str {
        "language"
    }
    fn name(&self) -> &str {
        "Language Manager"
    }
}

// ── LanguageError ─────────────────────────────────────────────────────────────

/// Error type for the Language Manager — alias of the shared [`fs_core::ManagerError`].
pub type LanguageError = fs_core::ManagerError;
