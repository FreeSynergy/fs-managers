// FreeSynergy Icon Manager
//
// Responsibilities:
//   - Manage icon set repositories (add, remove, enable, disable)
//   - Know which icon sets are installed (reads manifests from each repo)
//   - Resolve icon paths by name and variant (light/dark)
//   - Provide a reusable icon picker for use across all programs
//
// Any program that needs an icon picker uses IconManager instead of
// building its own file browser or hardcoding paths.

use std::path::PathBuf;

// ── Repository ────────────────────────────────────────────────────────────────

/// A configured source repository for icon sets.
///
/// Builtin repositories (e.g. the FreeSynergy.Icons repo) cannot be deleted —
/// only disabled. This rule is enforced by RepositoryManager.
#[derive(Debug, Clone)]
pub struct IconRepository {
    pub id: String,
    pub name: String,
    /// Remote URL or local path.
    pub url: String,
    pub enabled: bool,
    /// Builtin repos cannot be deleted, only disabled.
    pub builtin: bool,
}

// ── Icon Set ──────────────────────────────────────────────────────────────────

/// An installed icon set with full metadata.
#[derive(Debug, Clone)]
pub struct IconSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub has_dark_variants: bool,
    /// Which repository this set came from.
    pub source_repo_id: String,
    /// Absolute path to the set directory on disk.
    pub path: PathBuf,
    /// Number of icons (light variants only, dark variants not counted separately).
    pub icon_count: usize,
    /// Built-in sets ship with FreeSynergy and cannot be removed.
    pub builtin: bool,
}

/// A resolved icon ready for display or copying.
#[derive(Debug, Clone)]
pub struct Icon {
    pub set_id: String,
    pub name: String,
    pub path: PathBuf,
    pub is_dark: bool,
}

// ── Icon Picker ───────────────────────────────────────────────────────────────

/// Describes a request to pick an icon — used by any program that needs
/// an icon selection UI (Theme Manager, package editor, desktop settings, …).
#[derive(Debug, Clone, Default)]
pub struct IconPickerFilter {
    /// Limit results to a specific set. None = all sets.
    pub set_id: Option<String>,
    /// Case-insensitive substring match on icon name.
    pub search: Option<String>,
    pub prefer_dark: bool,
}

/// Result of an icon pick, ready for use or copying to a target path.
#[derive(Debug, Clone)]
pub struct PickedIcon {
    pub icon: Icon,
}

impl PickedIcon {
    /// Copies the icon file to `target_path`.
    ///
    /// The caller is responsible for choosing the destination
    /// (e.g. a program's own assets directory or a theme folder).
    pub fn copy_to(&self, target_path: &std::path::Path) -> Result<(), IconError> {
        std::fs::copy(&self.icon.path, target_path)
            .map(|_| ())
            .map_err(|e| IconError::IoError(e.to_string()))
    }
}

// ── Repository Manager ────────────────────────────────────────────────────────

/// Manages a list of repositories with per-program rules.
///
/// The same pattern is used by the Store, Bundle Manager, and Icon Manager —
/// each program instantiates its own RepositoryManager with its rule set.
///
/// Rules encoded here for the Icon Manager:
/// - Builtin repositories cannot be removed (only disabled).
pub struct RepositoryManager {
    repositories: Vec<IconRepository>,
}

impl RepositoryManager {
    pub fn new(repositories: Vec<IconRepository>) -> Self {
        Self { repositories }
    }

    pub fn list(&self) -> &[IconRepository] {
        &self.repositories
    }

    pub fn enabled(&self) -> impl Iterator<Item = &IconRepository> {
        self.repositories.iter().filter(|r| r.enabled)
    }

    pub fn add(&mut self, repo: IconRepository) {
        self.repositories.push(repo);
    }

    /// Removes a repository by ID.
    ///
    /// Returns `Err` if the repository is builtin — builtin repos can only
    /// be disabled, never deleted.
    pub fn remove(&mut self, id: &str) -> Result<(), IconError> {
        let pos = self
            .repositories
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| IconError::RepositoryNotFound(id.into()))?;

        if self.repositories[pos].builtin {
            return Err(IconError::CannotRemoveBuiltin(id.into()));
        }

        self.repositories.remove(pos);
        Ok(())
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), IconError> {
        let repo = self
            .repositories
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| IconError::RepositoryNotFound(id.into()))?;
        repo.enabled = enabled;
        Ok(())
    }
}

// ── Icon Manager ──────────────────────────────────────────────────────────────

/// Central manager for icon sets.
///
/// Knows where icon sets live on disk, resolves icons by name and variant,
/// and provides a filtered list for the icon picker UI.
pub struct IconManager {
    /// Root directory that contains all installed icon sets.
    icons_root: PathBuf,
    pub repositories: RepositoryManager,
}

impl IconManager {
    pub fn new(icons_root: impl Into<PathBuf>, repositories: Vec<IconRepository>) -> Self {
        Self {
            icons_root: icons_root.into(),
            repositories: RepositoryManager::new(repositories),
        }
    }

    /// Returns all installed icon sets with full metadata (path, icon count, …).
    pub fn sets(&self) -> Vec<IconSet> {
        let manifest_path = self.icons_root.join("manifest.toml");
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        parse_manifest_sets(&content)
            .into_iter()
            .map(|proto| {
                let path = self.icons_root.join(&proto.id);
                let icon_count = count_icons(&path);
                IconSet {
                    id: proto.id,
                    name: proto.name,
                    description: proto.description,
                    has_dark_variants: proto.has_dark_variants,
                    source_repo_id: proto.source_repo_id,
                    builtin: proto.builtin,
                    path,
                    icon_count,
                }
            })
            .collect()
    }

    /// Resolves a single icon by set ID and name.
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

    /// Returns icons matching the picker filter — used by the icon picker UI
    /// embedded in any program that needs icon selection.
    pub fn pick(&self, filter: &IconPickerFilter) -> Vec<PickedIcon> {
        let sets = self.sets();
        let mut results = Vec::new();

        for set in &sets {
            if let Some(ref id) = filter.set_id {
                if &set.id != id {
                    continue;
                }
            }

            let names = match self.list_set(&set.id) {
                Ok(n) => n,
                Err(_) => continue,
            };

            for name in names {
                if let Some(ref search) = filter.search {
                    if !name.to_lowercase().contains(&search.to_lowercase()) {
                        continue;
                    }
                }

                if let Some(icon) = self.resolve(&set.id, &name, filter.prefer_dark) {
                    results.push(PickedIcon { icon });
                }
            }
        }

        results
    }

    /// Lists all icon names in a set (light variants only).
    pub fn list_set(&self, set_id: &str) -> Result<Vec<String>, IconError> {
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_icons(set_dir: &std::path::Path) -> usize {
    let Ok(entries) = std::fs::read_dir(set_dir) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().and_then(|x| x.to_str()) == Some("svg")
                && !p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .ends_with("-dark")
        })
        .count()
}

// ── Manifest parser ───────────────────────────────────────────────────────────

/// Intermediate type used during manifest parsing (no path/count yet).
struct IconSetProto {
    id: String,
    name: String,
    description: String,
    has_dark_variants: bool,
    source_repo_id: String,
    builtin: bool,
}

fn parse_manifest_sets(content: &str) -> Vec<IconSetProto> {
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
            } else if let Some(val) = kv(line, "source_repo_id") {
                builder.source_repo_id = val;
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

fn kv<'a>(line: &'a str, key: &str) -> Option<String> {
    let prefix = format!("{key} =");
    let rest = line.strip_prefix(&prefix)?.trim();
    let val = rest.trim_matches('"');
    Some(val.to_string())
}

#[derive(Default)]
struct IconSetBuilder {
    id: String,
    name: String,
    description: String,
    has_dark_variants: bool,
    source_repo_id: String,
    builtin: bool,
}

impl IconSetBuilder {
    fn build(self) -> Option<IconSetProto> {
        if self.id.is_empty() {
            return None;
        }
        Some(IconSetProto {
            id: self.id,
            name: self.name,
            description: self.description,
            has_dark_variants: self.has_dark_variants,
            source_repo_id: self.source_repo_id,
            builtin: self.builtin,
        })
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum IconError {
    SetNotFound(String),
    RepositoryNotFound(String),
    CannotRemoveBuiltin(String),
    IoError(String),
}

impl std::fmt::Display for IconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetNotFound(id) => write!(f, "Icon set not found: {id}"),
            Self::RepositoryNotFound(id) => write!(f, "Repository not found: {id}"),
            Self::CannotRemoveBuiltin(id) => {
                write!(f, "Cannot remove builtin repository: {id}")
            }
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for IconError {}
