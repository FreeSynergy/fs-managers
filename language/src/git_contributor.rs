// Git contributor check via SSH.
//
// Determines whether the user can push to the FreeSynergy GitHub organisation
// by testing their SSH key against GitHub's authentication endpoint.
//
// Strategy:
//   1. Fast pre-check: does any SSH private key file exist in ~/.ssh/?
//   2. If yes: run `ssh -T git@github.com` — GitHub responds with
//              "Hi <username>! You have successfully authenticated…" on stderr.
//   3. Parse the username and cache the result in ~/.config/fsn/git_contributor.toml.
//   4. Cache is valid for 7 days; re-check on expiry or on explicit clear.
//
// Note: successful SSH auth means the key is known to GitHub.
// Actual write access to a specific repo is validated on push (graceful error).
//
// push_translation uses GitRepoPort (implemented by GixRepo) for all git operations.
// No gix imports here — if the gix API changes, only src/git.rs needs updating.

use crate::git::{CommitAuthor, GitRepoPort, GixRepo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── ContributorStatus ─────────────────────────────────────────────────────────

/// Result of the SSH contributor check.
#[derive(Debug, Clone, PartialEq)]
pub enum ContributorStatus {
    /// Not yet checked (no cache and no check run).
    Unknown,
    /// SSH authentication succeeded — the user has a GitHub account with this key.
    /// The `github_user` is the GitHub username returned by GitHub's endpoint.
    Authenticated { github_user: String },
    /// No local SSH key found, or SSH authentication failed.
    NotAuthenticated,
}

impl ContributorStatus {
    /// Converts this status into a cache record for serialization.
    fn to_cache(&self, checked_at_secs: u64) -> ContributorCache {
        match self {
            Self::Authenticated { github_user } => ContributorCache {
                authenticated: true,
                github_user: Some(github_user.clone()),
                checked_at_secs,
            },
            _ => ContributorCache {
                authenticated: false,
                github_user: None,
                checked_at_secs,
            },
        }
    }
}

// ── Cache ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct ContributorCache {
    authenticated: bool,
    github_user: Option<String>,
    /// Unix timestamp (seconds) when the check was last performed.
    checked_at_secs: u64,
}

// ── GitContributorCheck ───────────────────────────────────────────────────────

pub struct GitContributorCheck;

impl GitContributorCheck {
    fn cache_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("fsn")
            .join("git_contributor.toml")
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Returns the cached status if it is less than 7 days old.
    /// Returns `None` if the cache is absent or expired.
    pub fn cached() -> Option<ContributorStatus> {
        let content = std::fs::read_to_string(Self::cache_path()).ok()?;
        let cache: ContributorCache = toml::from_str(&content).ok()?;
        let age_secs = Self::now_secs().saturating_sub(cache.checked_at_secs);
        if age_secs > 7 * 24 * 3600 {
            return None; // expired
        }
        Some(if cache.authenticated {
            ContributorStatus::Authenticated {
                github_user: cache.github_user.unwrap_or_default(),
            }
        } else {
            ContributorStatus::NotAuthenticated
        })
    }

    fn save_cache(status: &ContributorStatus) {
        let cache = status.to_cache(Self::now_secs());
        if let Ok(toml_str) = toml::to_string_pretty(&cache) {
            let path = Self::cache_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, toml_str);
        }
    }

    /// Returns `true` if any standard SSH private key file exists in `~/.ssh/`.
    /// This is a fast local check — no network call needed.
    pub fn has_local_ssh_key() -> bool {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let ssh_dir = PathBuf::from(home).join(".ssh");
        ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"]
            .iter()
            .any(|name| ssh_dir.join(name).exists())
    }

    /// Runs `ssh -T git@github.com` to verify SSH authentication, then caches
    /// the result. This call blocks and may take several seconds.
    ///
    /// Call from a background thread or async task:
    /// ```rust
    /// spawn(async { GitContributorCheck::check_and_cache() });
    /// ```
    pub fn check_and_cache() -> ContributorStatus {
        // Skip the network call if there is no SSH key at all.
        if !Self::has_local_ssh_key() {
            let status = ContributorStatus::NotAuthenticated;
            Self::save_cache(&status);
            return status;
        }

        // GitHub's SSH endpoint always exits with code 1 (no shell allowed),
        // but writes an identifying message to stderr:
        //   success : "Hi <username>! You have successfully authenticated, but …"
        //   failure : "git@github.com: Permission denied (publickey)."
        let result = std::process::Command::new("ssh")
            .args([
                "-T",
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "ConnectTimeout=8",
                "-o",
                "BatchMode=yes", // never prompt for passphrase
                "git@github.com",
            ])
            .output();

        let status = match result {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stderr);
                if let Some(username) = Self::parse_username(&text) {
                    ContributorStatus::Authenticated {
                        github_user: username,
                    }
                } else {
                    ContributorStatus::NotAuthenticated
                }
            }
            Err(_) => ContributorStatus::NotAuthenticated,
        };

        Self::save_cache(&status);
        status
    }

    /// Parses `"Hi <username>!"` from GitHub's SSH response text.
    fn parse_username(output: &str) -> Option<String> {
        let line = output.lines().find(|l| l.starts_with("Hi "))?;
        let name = line.strip_prefix("Hi ")?.split('!').next()?.trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    /// Clears the cache so the next call to `cached()` returns `None`,
    /// forcing a fresh check on next `check_and_cache()`.
    pub fn clear_cache() {
        let _ = std::fs::remove_file(Self::cache_path());
    }

    /// Attempts to push a translated TOML file to the FreeSynergy Node repo.
    ///
    /// Requires:
    ///   - A local clone of the Node repo at `repo_path`
    ///   - Write access to `git@github.com:FreeSynergy/Node.git`
    ///
    /// The file is written to `Node/i18n/{lang_code}/ui.toml` within the clone,
    /// then committed and pushed via `GitRepoPort` (backed by `GixRepo`).
    pub fn push_translation(
        repo_path: &std::path::Path,
        lang_code: &str,
        toml_content: &str,
    ) -> Result<String, String> {
        let repo = GixRepo::open(repo_path).map_err(|e| e.to_string())?;
        push_translation_with(&repo, repo_path, lang_code, toml_content)
    }
}

/// Core push logic — uses only `GitRepoPort`, no gix types.
/// Separated from `push_translation` to allow testing with a mock `GitRepoPort`.
fn push_translation_with(
    repo: &impl GitRepoPort,
    repo_path: &std::path::Path,
    lang_code: &str,
    toml_content: &str,
) -> Result<String, String> {
    // Write the file to disk so the working tree stays in sync.
    let dest_dir = repo_path.join("Node").join("i18n").join(lang_code);
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    std::fs::write(dest_dir.join("ui.toml"), toml_content).map_err(|e| e.to_string())?;

    // 1. Write blob.
    let blob = repo
        .write_blob(toml_content.as_bytes())
        .map_err(|e| e.to_string())?;

    // 2. Build updated tree.
    let (parent_id, root_tree) = repo.head_commit_and_tree().map_err(|e| e.to_string())?;

    let path_components = ["Node", "i18n", lang_code, "ui.toml"];
    let new_tree = repo
        .insert_blob_at_path(root_tree, &path_components, blob)
        .map_err(|e| e.to_string())?;

    // 3. Create commit.
    let author = CommitAuthor {
        name: repo
            .config_string("user.name")
            .unwrap_or_else(|| "FreeSynergy".into()),
        email: repo
            .config_string("user.email")
            .unwrap_or_else(|| "noreply@freesynergy.net".into()),
    };
    let message = format!("i18n: add/update {lang_code} translation");
    let commit_id = repo
        .create_commit(&author, &message, new_tree, parent_id)
        .map_err(|e| e.to_string())?;

    // 4. Push to origin.
    let head_ref = repo.head_ref().map_err(|e| e.to_string())?;
    let refspec = format!("{head_ref}:{head_ref}");
    repo.push_to_origin(&refspec).map_err(|e| e.to_string())?;

    Ok(format!(
        "Translation for '{lang_code}' pushed successfully (commit {}).",
        commit_id.to_hex()
    ))
}
