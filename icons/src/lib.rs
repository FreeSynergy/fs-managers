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
    pub has_dark_variants: bool,
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
        // TODO: parse icons_root/manifest.toml
        vec![
            IconSet {
                id: "homarrlabs".into(),
                name: "Homarr Labs Dashboard Icons".into(),
                has_dark_variants: true,
            },
        ]
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
