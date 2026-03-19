// FreeSynergy Cursor Manager
//
// Responsibilities:
//   - Manage cursor set repositories (add, remove, enable, disable)
//   - Know which cursor sets are installed (reads manifests from each repo)
//   - Resolve cursor files by slot and animation state
//   - Activate a cursor set (marks it as the active set for the current theme)
//   - Accept a CursorSetDraft for saving/publishing new sets
//
// Repository management uses fsn_core::RepositoryManager<CursorRepository> —
// the same generic abstraction shared by the Store, Icon Manager, and Bundle Manager.
//
// The 31 standard cursor slots are defined by the FSN UI standards doc.
// Each slot maps to a filename (e.g. CursorSlot::Pointer → "pointer.svg").
// Some slots support animation (multiple SVG frames with per-frame durations).

use std::path::{Path, PathBuf};

use fsn_core::{Repository, RepositoryManager};
pub use fsn_core::RepositoryError;

// ── CursorRepository ──────────────────────────────────────────────────────────

/// A configured source repository for cursor sets.
///
/// Builtin repositories cannot be deleted — only disabled.
/// This rule is enforced by [`RepositoryManager`].
#[derive(Debug, Clone)]
pub struct CursorRepository {
    pub id: String,
    pub name: String,
    /// Remote URL or local path.
    pub url: String,
    pub enabled: bool,
    /// Builtin repos cannot be deleted, only disabled.
    pub builtin: bool,
}

impl Repository for CursorRepository {
    fn id(&self) -> &str { &self.id }
    fn builtin(&self) -> bool { self.builtin }
    fn enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
}

// ── CursorSlot ────────────────────────────────────────────────────────────────

/// All 31 standard cursor positions defined by FSN UI standards.
///
/// The `filename` method returns the SVG filename expected in a cursor set
/// directory (without extension). Missing slots fall back to CSS defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorSlot {
    Default,
    Pointer,
    NotAllowed,
    Busy,
    Progress,
    Text,
    TextVertical,
    Move,
    Grab,
    Grabbing,
    Crosshair,
    ZoomIn,
    ZoomOut,
    Help,
    ContextMenu,
    Alias,
    Cell,
    DropOk,
    DropDeny,
    ResizeN,
    ResizeS,
    ResizeE,
    ResizeW,
    ResizeNs,
    ResizeEw,
    ResizeNe,
    ResizeNw,
    ResizeSe,
    ResizeSw,
    ResizeNesw,
    ResizeNwse,
}

impl CursorSlot {
    /// The filename base (without `.svg`) expected in a cursor set directory.
    pub fn filename(self) -> &'static str {
        match self {
            Self::Default      => "default",
            Self::Pointer      => "pointer",
            Self::NotAllowed   => "not-allowed",
            Self::Busy         => "busy",
            Self::Progress     => "progress",
            Self::Text         => "text",
            Self::TextVertical => "text-vertical",
            Self::Move         => "move",
            Self::Grab         => "grab",
            Self::Grabbing     => "grabbing",
            Self::Crosshair    => "crosshair",
            Self::ZoomIn       => "zoom-in",
            Self::ZoomOut      => "zoom-out",
            Self::Help         => "help",
            Self::ContextMenu  => "context-menu",
            Self::Alias        => "alias",
            Self::Cell         => "cell",
            Self::DropOk       => "drop-ok",
            Self::DropDeny     => "drop-deny",
            Self::ResizeN      => "resize-n",
            Self::ResizeS      => "resize-s",
            Self::ResizeE      => "resize-e",
            Self::ResizeW      => "resize-w",
            Self::ResizeNs     => "resize-ns",
            Self::ResizeEw     => "resize-ew",
            Self::ResizeNe     => "resize-ne",
            Self::ResizeNw     => "resize-nw",
            Self::ResizeSe     => "resize-se",
            Self::ResizeSw     => "resize-sw",
            Self::ResizeNesw   => "resize-nesw",
            Self::ResizeNwse   => "resize-nwse",
        }
    }

    /// Default hotspot for this slot. Most cursors use (0, 0).
    pub fn default_hotspot(self) -> (u32, u32) {
        match self {
            Self::Pointer   => (6, 0),
            Self::Crosshair => (12, 12),
            Self::Grabbing  => (12, 8),
            _               => (0, 0),
        }
    }

    /// All 31 slots in standard order.
    pub fn all() -> &'static [CursorSlot] {
        &[
            Self::Default, Self::Pointer, Self::NotAllowed, Self::Busy,
            Self::Progress, Self::Text, Self::TextVertical, Self::Move,
            Self::Grab, Self::Grabbing, Self::Crosshair, Self::ZoomIn,
            Self::ZoomOut, Self::Help, Self::ContextMenu, Self::Alias,
            Self::Cell, Self::DropOk, Self::DropDeny, Self::ResizeN,
            Self::ResizeS, Self::ResizeE, Self::ResizeW, Self::ResizeNs,
            Self::ResizeEw, Self::ResizeNe, Self::ResizeNw, Self::ResizeSe,
            Self::ResizeSw, Self::ResizeNesw, Self::ResizeNwse,
        ]
    }

    /// Minimum required slots — the cursor set is considered incomplete without these.
    pub fn minimum_required() -> &'static [CursorSlot] {
        &[
            Self::Default, Self::Pointer, Self::NotAllowed, Self::Busy,
            Self::Progress, Self::Text, Self::Move, Self::Grab, Self::Grabbing,
            Self::DropOk, Self::DropDeny, Self::ResizeNs, Self::ResizeEw,
            Self::ResizeNwse, Self::ResizeNesw,
        ]
    }

    /// Try to parse a slot from its filename string.
    pub fn from_filename(s: &str) -> Option<Self> {
        CursorSlot::all().iter().copied().find(|slot| slot.filename() == s)
    }
}

// ── Animation ─────────────────────────────────────────────────────────────────

/// Animation data for one cursor slot (e.g. `busy`, `progress`).
#[derive(Debug, Clone)]
pub struct CursorAnimation {
    /// Paths to the individual SVG frames, in order.
    pub frames: Vec<PathBuf>,
    /// Duration in milliseconds for each frame (parallel to `frames`).
    pub frame_ms: Vec<u32>,
    pub loop_animation: bool,
}

// ── Resolved cursor ───────────────────────────────────────────────────────────

/// A resolved cursor slot — either a static SVG or an animation.
#[derive(Debug, Clone)]
pub enum ResolvedCursor {
    Static {
        path: PathBuf,
        hotspot: (u32, u32),
    },
    Animated {
        animation: CursorAnimation,
        hotspot: (u32, u32),
    },
}

// ── CursorSet ─────────────────────────────────────────────────────────────────

/// An installed cursor set with full metadata.
#[derive(Debug, Clone)]
pub struct CursorSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    /// Absolute path to the set directory on disk.
    pub path: PathBuf,
    /// Which slots are present in this set.
    pub present_slots: Vec<CursorSlot>,
    /// Which repository this set came from.
    pub source_repo_id: String,
    pub builtin: bool,
}

impl CursorSet {
    /// Returns true if all minimum required slots are present.
    pub fn is_complete(&self) -> bool {
        CursorSlot::minimum_required()
            .iter()
            .all(|slot| self.present_slots.contains(slot))
    }

    /// Returns the slots from the minimum set that are missing.
    pub fn missing_required(&self) -> Vec<CursorSlot> {
        CursorSlot::minimum_required()
            .iter()
            .copied()
            .filter(|slot| !self.present_slots.contains(slot))
            .collect()
    }
}

// ── CursorSetDraft ────────────────────────────────────────────────────────────

/// An in-progress cursor set — filled by the UI creation form.
///
/// Call [`CursorManager::save_draft`] to write the files to disk
/// and generate the manifest.
#[derive(Debug, Clone, Default)]
pub struct CursorSetDraft {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    /// SVG content per slot (static cursors).
    pub slots: Vec<(CursorSlot, String)>,
    /// Hotspot overrides per slot. Falls back to `CursorSlot::default_hotspot`.
    pub hotspot_overrides: Vec<(CursorSlot, (u32, u32))>,
    /// Animation data per slot (for animated cursors).
    pub animations: Vec<(CursorSlot, AnimationDraft)>,
}

/// Animation data supplied via the creation form.
#[derive(Debug, Clone, Default)]
pub struct AnimationDraft {
    /// SVG content per frame, in order.
    pub frames: Vec<String>,
    /// Duration in milliseconds per frame.
    pub frame_ms: Vec<u32>,
    pub loop_animation: bool,
}

impl CursorSetDraft {
    /// Returns which of the minimum required slots are still missing.
    pub fn missing_required(&self) -> Vec<CursorSlot> {
        let filled: Vec<CursorSlot> = self.slots.iter().map(|(s, _)| *s).collect();
        CursorSlot::minimum_required()
            .iter()
            .copied()
            .filter(|s| !filled.contains(s))
            .collect()
    }
}

// ── CursorManager ─────────────────────────────────────────────────────────────

/// Central manager for cursor sets.
///
/// Knows where cursor sets live on disk, resolves cursors by slot,
/// and accepts drafts for saving new sets.
///
/// Repository management is delegated to
/// `RepositoryManager<CursorRepository>` from `fsn-core`.
pub struct CursorManager {
    /// Root directory that contains the `cursor-sets/` subdirectory.
    icons_root: PathBuf,
    pub repositories: RepositoryManager<CursorRepository>,
}

impl CursorManager {
    pub fn new(
        icons_root: impl Into<PathBuf>,
        repositories: Vec<CursorRepository>,
    ) -> Self {
        Self {
            icons_root: icons_root.into(),
            repositories: RepositoryManager::new(repositories),
        }
    }

    /// Path to the cursor-sets directory.
    fn cursor_sets_dir(&self) -> PathBuf {
        self.icons_root.join("cursor-sets")
    }

    /// Returns all installed cursor sets.
    pub fn sets(&self) -> Vec<CursorSet> {
        let manifest_path = self.icons_root.join("manifest.toml");
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        parse_manifest_cursor_sets(&content)
            .into_iter()
            .map(|proto| {
                let path = self.cursor_sets_dir().join(&proto.id);
                let present_slots = detect_present_slots(&path);
                CursorSet {
                    id: proto.id,
                    name: proto.name,
                    description: proto.description,
                    author: proto.author,
                    version: proto.version,
                    source_repo_id: proto.source_repo_id,
                    builtin: proto.builtin,
                    path,
                    present_slots,
                }
            })
            .collect()
    }

    /// Resolves a single cursor slot from the given set.
    ///
    /// Returns `None` if the slot file is missing (caller should apply CSS fallback).
    pub fn resolve(&self, set_id: &str, slot: CursorSlot) -> Option<ResolvedCursor> {
        let set_dir = self.cursor_sets_dir().join(set_id);
        let manifest_path = set_dir.join("manifest.toml");

        let hotspot = read_hotspot_override(&manifest_path, slot)
            .unwrap_or_else(|| slot.default_hotspot());

        // Check for animation first.
        if let Some(anim) = read_animation(&manifest_path, &set_dir, slot) {
            return Some(ResolvedCursor::Animated { animation: anim, hotspot });
        }

        // Fall back to static SVG.
        let svg_path = set_dir.join(format!("{}.svg", slot.filename()));
        if svg_path.exists() {
            return Some(ResolvedCursor::Static { path: svg_path, hotspot });
        }

        None
    }

    /// Saves a [`CursorSetDraft`] to disk (writes SVG files + manifest.toml).
    ///
    /// Does not push to any repository — that is handled by the git layer above.
    pub fn save_draft(
        &self,
        draft: &CursorSetDraft,
        source_repo_id: &str,
    ) -> Result<PathBuf, CursorError> {
        if draft.id.is_empty() {
            return Err(CursorError::InvalidDraft("id is required".into()));
        }
        if draft.name.is_empty() {
            return Err(CursorError::InvalidDraft("name is required".into()));
        }
        if draft.version.is_empty() {
            return Err(CursorError::InvalidDraft("version is required".into()));
        }

        let set_dir = self.cursor_sets_dir().join(&draft.id);
        std::fs::create_dir_all(&set_dir)
            .map_err(|e| CursorError::IoError(e.to_string()))?;

        // Write static SVG slots.
        for (slot, svg_content) in &draft.slots {
            let path = set_dir.join(format!("{}.svg", slot.filename()));
            std::fs::write(&path, svg_content)
                .map_err(|e| CursorError::IoError(e.to_string()))?;
        }

        // Write animation frames.
        for (slot, anim) in &draft.animations {
            for (i, frame_svg) in anim.frames.iter().enumerate() {
                let filename = format!("{}-frame-{}.svg", slot.filename(), i + 1);
                std::fs::write(set_dir.join(&filename), frame_svg)
                    .map_err(|e| CursorError::IoError(e.to_string()))?;
            }
        }

        // Generate manifest.toml.
        let manifest = generate_manifest(draft, source_repo_id);
        std::fs::write(set_dir.join("manifest.toml"), manifest)
            .map_err(|e| CursorError::IoError(e.to_string()))?;

        Ok(set_dir)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Detects which cursor slots are present by checking for `.svg` files.
fn detect_present_slots(set_dir: &Path) -> Vec<CursorSlot> {
    let Ok(entries) = std::fs::read_dir(set_dir) else {
        return vec![];
    };
    let mut slots = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Skip animation frames (e.g. "busy-frame-1.svg").
        if stem.contains("-frame-") {
            continue;
        }
        if let Some(slot) = CursorSlot::from_filename(stem) {
            slots.push(slot);
        }
    }
    slots
}

/// Reads a hotspot override from a set's manifest.toml for the given slot.
fn read_hotspot_override(manifest_path: &Path, slot: CursorSlot) -> Option<(u32, u32)> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let mut in_hotspots = false;
    let key = slot.filename();
    for line in content.lines() {
        let line = line.trim();
        if line == "[hotspots]" {
            in_hotspots = true;
            continue;
        }
        if in_hotspots {
            if line.starts_with('[') {
                break; // Left [hotspots] section.
            }
            if let Some(rest) = line.strip_prefix(&format!("{key} =")) {
                // Parse "[x, y]"
                let rest = rest.trim().trim_matches('[').trim_matches(']');
                let mut parts = rest.split(',');
                let x = parts.next()?.trim().parse().ok()?;
                let y = parts.next()?.trim().parse().ok()?;
                return Some((x, y));
            }
        }
    }
    None
}

/// Reads animation data for one slot from the manifest, if present.
fn read_animation(manifest_path: &Path, set_dir: &Path, slot: CursorSlot) -> Option<CursorAnimation> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let section_header = format!("[animated.{}]", slot.filename());
    let mut in_section = false;
    let mut frame_files: Vec<String> = Vec::new();
    let mut frame_ms: Vec<u32> = Vec::new();
    let mut loop_anim = true;

    for line in content.lines() {
        let line = line.trim();
        if line == section_header {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with('[') {
                break;
            }
            if let Some(rest) = line.strip_prefix("frames =") {
                frame_files = parse_toml_string_array(rest.trim());
            } else if let Some(rest) = line.strip_prefix("frame_ms =") {
                frame_ms = parse_toml_u32_array(rest.trim());
            } else if let Some(rest) = line.strip_prefix("loop =") {
                loop_anim = rest.trim() == "true";
            }
        }
    }

    if frame_files.is_empty() {
        return None;
    }

    let frames: Vec<PathBuf> = frame_files
        .into_iter()
        .map(|f| set_dir.join(f))
        .filter(|p| p.exists())
        .collect();

    if frames.is_empty() {
        return None;
    }

    Some(CursorAnimation { frames, frame_ms, loop_animation: loop_anim })
}

fn parse_toml_string_array(s: &str) -> Vec<String> {
    s.trim_matches('[').trim_matches(']')
        .split(',')
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

fn parse_toml_u32_array(s: &str) -> Vec<u32> {
    s.trim_matches('[').trim_matches(']')
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect()
}

/// Generates the manifest.toml content for a cursor set draft.
fn generate_manifest(draft: &CursorSetDraft, source_repo_id: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!("id          = \"{}\"\n", draft.id));
    out.push_str(&format!("name        = \"{}\"\n", draft.name));
    out.push_str(&format!("description = \"{}\"\n", draft.description));
    out.push_str(&format!("author      = \"{}\"\n", draft.author));
    out.push_str(&format!("version     = \"{}\"\n", draft.version));
    out.push_str(&format!("source_repo_id = \"{source_repo_id}\"\n"));
    out.push_str("builtin     = false\n");

    // Hotspot overrides (only those that differ from the slot default).
    let non_default_hotspots: Vec<_> = draft
        .hotspot_overrides
        .iter()
        .filter(|(slot, hs)| *hs != slot.default_hotspot())
        .collect();

    if !non_default_hotspots.is_empty() {
        out.push_str("\n[hotspots]\n");
        for (slot, (x, y)) in &non_default_hotspots {
            out.push_str(&format!("{} = [{x}, {y}]\n", slot.filename()));
        }
    }

    // Animation sections.
    for (slot, anim) in &draft.animations {
        out.push_str(&format!("\n[animated.{}]\n", slot.filename()));

        let frame_filenames: Vec<String> = (1..=anim.frames.len())
            .map(|i| format!("{}-frame-{i}.svg", slot.filename()))
            .collect();
        let frames_toml = frame_filenames
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("frames   = [{frames_toml}]\n"));

        let ms_toml = anim
            .frame_ms
            .iter()
            .map(|ms| ms.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("frame_ms = [{ms_toml}]\n"));
        out.push_str(&format!("loop     = {}\n", anim.loop_animation));
    }

    out
}

// ── Manifest parser ───────────────────────────────────────────────────────────

struct CursorSetProto {
    id: String,
    name: String,
    description: String,
    author: String,
    version: String,
    source_repo_id: String,
    builtin: bool,
}

fn parse_manifest_cursor_sets(content: &str) -> Vec<CursorSetProto> {
    let mut sets = Vec::new();
    let mut current: Option<CursorSetBuilder> = None;

    for line in content.lines() {
        let line = line.trim();
        if line == "[[cursor_set]]" {
            if let Some(builder) = current.take() {
                if let Some(set) = builder.build() {
                    sets.push(set);
                }
            }
            current = Some(CursorSetBuilder::default());
            continue;
        }
        if let Some(ref mut builder) = current {
            if let Some(val) = kv(line, "id") {
                builder.id = val;
            } else if let Some(val) = kv(line, "name") {
                builder.name = val;
            } else if let Some(val) = kv(line, "description") {
                builder.description = val;
            } else if let Some(val) = kv(line, "author") {
                builder.author = val;
            } else if let Some(val) = kv(line, "version") {
                builder.version = val;
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

fn kv(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} =");
    let rest = line.strip_prefix(&prefix)?.trim();
    Some(rest.trim_matches('"').to_string())
}

#[derive(Default)]
struct CursorSetBuilder {
    id: String,
    name: String,
    description: String,
    author: String,
    version: String,
    source_repo_id: String,
    builtin: bool,
}

impl CursorSetBuilder {
    fn build(self) -> Option<CursorSetProto> {
        if self.id.is_empty() {
            return None;
        }
        Some(CursorSetProto {
            id: self.id,
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            source_repo_id: self.source_repo_id,
            builtin: self.builtin,
        })
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors for cursor set operations.
///
/// For repository errors use [`RepositoryError`] (re-exported from `fsn-core`).
#[derive(Debug)]
pub enum CursorError {
    SetNotFound(String),
    InvalidDraft(String),
    IoError(String),
}

impl std::fmt::Display for CursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetNotFound(id) => write!(f, "Cursor set not found: {id}"),
            Self::InvalidDraft(msg) => write!(f, "Invalid cursor set draft: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for CursorError {}
