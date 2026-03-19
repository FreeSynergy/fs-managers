// FreeSynergy Icon Manager
//
// Responsibilities:
//   - Know which icon sets are available (reads FreeSynergy.Icons/manifest.toml)
//   - Resolve icon paths by name and variant (light/dark)
//   - Provide a UI icon picker component for use across all programs
//
// Any program that needs an icon picker uses IconManager instead of
// building its own file browser or hardcoding paths.

/// An icon set as described in manifest.toml.
#[derive(Debug, Clone)]
pub struct IconSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub has_dark_variants: bool,
    pub builtin: bool,
}

/// A resolved icon with its file path.
#[derive(Debug, Clone)]
pub struct Icon {
    pub set_id: String,
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dark: bool,
}

/// Manages icon sets and provides icon lookup for the FreeSynergy ecosystem.
pub struct IconManager {
    icons_root: std::path::PathBuf,
}

impl IconManager {
    pub fn new(icons_root: impl Into<std::path::PathBuf>) -> Self {
        Self { icons_root: icons_root.into() }
    }

    /// Returns all available icon sets from manifest.toml.
    pub fn sets(&self) -> Vec<IconSet> {
        let manifest_path = self.icons_root.join("manifest.toml");
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        parse_manifest_sets(&content)
    }

    /// Resolves an icon by set ID and name, preferring the dark variant if requested.
    pub fn resolve(&self, set_id: &str, name: &str, prefer_dark: bool) -> Option<Icon> {
        let set_dir = self.icons_root.join(set_id);

        if prefer_dark {
            let dark_path = set_dir.join(format!("{name}-dark.svg"));
            if dark_path.exists() {
                return Some(Icon {
                    set_id: set_id.into(),
                    name: name.into(),
                    path: dark_path,
                    is_dark: true,
                });
            }
        }

        let light_path = set_dir.join(format!("{name}.svg"));
        if light_path.exists() {
            return Some(Icon {
                set_id: set_id.into(),
                name: name.into(),
                path: light_path,
                is_dark: false,
            });
        }

        None
    }

    /// Lists all icons in a set.
    pub fn list(&self, set_id: &str) -> Result<Vec<String>, IconError> {
        let set_dir = self.icons_root.join(set_id);
        if !set_dir.exists() {
            return Err(IconError::SetNotFound(set_id.into()));
        }

        let mut names = Vec::new();
        for entry in std::fs::read_dir(&set_dir).map_err(|e| IconError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| IconError::IoError(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("svg") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Skip dark variants from the name list
                    if !stem.ends_with("-dark") {
                        names.push(stem.to_string());
                    }
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

// ── Manifest parser ───────────────────────────────────────────────────────────

/// Parses `[[set]]` entries from manifest.toml without pulling in a TOML crate.
///
/// Minimal hand-rolled parser: iterates [[set]] blocks and extracts known keys.
fn parse_manifest_sets(content: &str) -> Vec<IconSet> {
    let mut sets = Vec::new();
    let mut current: Option<IconSetBuilder> = None;

    for line in content.lines() {
        let line = line.trim();
        if line == "[[set]]" {
            if let Some(builder) = current.take() {
                if let Some(set) = builder.build() {
                    sets.push(set);
                }
            }
            current = Some(IconSetBuilder::default());
            continue;
        }
        if let Some(ref mut builder) = current {
            if let Some(val) = kv(line, "id") {
                builder.id = val;
            } else if let Some(val) = kv(line, "name") {
                builder.name = val;
            } else if let Some(val) = kv(line, "description") {
                builder.description = val;
            } else if let Some(val) = kv(line, "has_dark_variants") {
                builder.has_dark_variants = val == "true";
            } else if let Some(val) = kv(line, "builtin") {
                builder.builtin = val == "true";
            }
        }
    }
    if let Some(builder) = current {
        if let Some(set) = builder.build() {
            sets.push(set);
        }
    }
    sets
}

/// Extracts the value of `key = "value"` or `key = value` from a TOML line.
fn kv<'a>(line: &'a str, key: &str) -> Option<String> {
    let prefix = format!("{key} =");
    let rest = line.strip_prefix(&prefix)?.trim();
    // Strip surrounding quotes if present.
    let val = rest.trim_matches('"');
    Some(val.to_string())
}

#[derive(Default)]
struct IconSetBuilder {
    id: String,
    name: String,
    description: String,
    has_dark_variants: bool,
    builtin: bool,
}

impl IconSetBuilder {
    fn build(self) -> Option<IconSet> {
        if self.id.is_empty() { return None; }
        Some(IconSet {
            id: self.id,
            name: self.name,
            description: self.description,
            has_dark_variants: self.has_dark_variants,
            builtin: self.builtin,
        })
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum IconError {
    SetNotFound(String),
    IoError(String),
}

impl std::fmt::Display for IconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetNotFound(id) => write!(f, "Icon set not found: {id}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for IconError {}
