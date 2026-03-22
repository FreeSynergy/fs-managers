// Git abstraction layer.
//
// ALL gix API calls are isolated in this module — specifically in `GixRepo`.
// Callers import only `GitRepoPort`, `GixRepo`, `CommitAuthor`, `GitError`, and `OidBytes`.
// When the gix API changes, only `GixRepo` and its impl need updating.

use std::path::Path;

use gix::bstr::ByteSlice;

// ── Domain types ──────────────────────────────────────────────────────────────

/// Opaque object identifier (raw hash bytes, SHA-1 or SHA-256).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidBytes(Vec<u8>);

impl OidBytes {
    fn from_gix(id: gix::ObjectId) -> Self {
        Self(id.as_bytes().to_vec())
    }

    fn to_gix(&self) -> Result<gix::ObjectId, GitError> {
        gix::ObjectId::try_from(self.0.as_slice())
            .map_err(|_| GitError::Head(format!("invalid OID bytes: {:?}", self.0)))
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Author / committer identity for a git commit.
pub struct CommitAuthor {
    pub name:  String,
    pub email: String,
}

// ── GitError ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum GitError {
    Open(String),
    BlobWrite(String),
    TreeWrite(String),
    Commit(String),
    Push(String),
    Head(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open(e)      => write!(f, "git open: {e}"),
            Self::BlobWrite(e) => write!(f, "git blob write: {e}"),
            Self::TreeWrite(e) => write!(f, "git tree write: {e}"),
            Self::Commit(e)    => write!(f, "git commit: {e}"),
            Self::Push(e)      => write!(f, "git push: {e}"),
            Self::Head(e)      => write!(f, "git HEAD: {e}"),
        }
    }
}

impl std::error::Error for GitError {}

// ── GitRepoPort trait ─────────────────────────────────────────────────────────

/// Abstraction over git repository operations.
///
/// All callers depend only on this trait. `GixRepo` is the single implementation
/// that touches `gix` directly. When the `gix` API changes, only `GixRepo` needs
/// updating — the rest of the crate is unaffected.
pub trait GitRepoPort {
    /// Reads a git config value by dot-separated key (e.g. `"user.name"`).
    fn config_string(&self, key: &str) -> Option<String>;

    /// Returns the full ref name of HEAD (e.g. `"refs/heads/main"`).
    /// Errors on detached HEAD.
    fn head_ref(&self) -> Result<String, GitError>;

    /// Returns `(commit_id, tree_id)` for the current HEAD commit.
    fn head_commit_and_tree(&self) -> Result<(OidBytes, OidBytes), GitError>;

    /// Writes raw bytes as a blob object and returns its OID.
    fn write_blob(&self, content: &[u8]) -> Result<OidBytes, GitError>;

    /// Inserts `blob` at `path_components` inside `tree`, creating intermediate
    /// subtrees as needed. Returns the new root tree OID.
    fn insert_blob_at_path(
        &self,
        tree: OidBytes,
        path: &[&str],
        blob: OidBytes,
    ) -> Result<OidBytes, GitError>;

    /// Creates a commit on HEAD referencing the given tree and single parent.
    fn create_commit(
        &self,
        author:  &CommitAuthor,
        message: &str,
        tree:    OidBytes,
        parent:  OidBytes,
    ) -> Result<OidBytes, GitError>;

    /// Pushes `refspec` (e.g. `"refs/heads/main:refs/heads/main"`) to `origin`.
    fn push_to_origin(&self, refspec: &str) -> Result<(), GitError>;
}

// ── GixRepo ───────────────────────────────────────────────────────────────────

/// `gix`-backed implementation of `GitRepoPort`.
///
/// This is the ONLY place in the crate that imports or uses `gix` types.
/// All gix API calls are confined to this struct and its `impl` blocks.
pub struct GixRepo {
    inner: gix::Repository,
}

impl GixRepo {
    pub fn open(path: &Path) -> Result<Self, GitError> {
        gix::open(path)
            .map(|inner| Self { inner })
            .map_err(|e| GitError::Open(e.to_string()))
    }
}

impl GitRepoPort for GixRepo {
    fn config_string(&self, key: &str) -> Option<String> {
        self.inner
            .config_snapshot()
            .string(key)
            .map(|v| v.to_str_lossy().into_owned())
    }

    fn head_ref(&self) -> Result<String, GitError> {
        self.inner
            .head_name()
            .map_err(|e| GitError::Head(e.to_string()))?
            .ok_or_else(|| GitError::Head("detached HEAD — cannot push".into()))
            .map(|r| r.to_string())
    }

    fn head_commit_and_tree(&self) -> Result<(OidBytes, OidBytes), GitError> {
        let commit   = self.inner.head_commit().map_err(|e| GitError::Head(e.to_string()))?;
        let tree_id  = commit.tree_id().map_err(|e| GitError::Head(e.to_string()))?.detach();
        Ok((OidBytes::from_gix(commit.id), OidBytes::from_gix(tree_id)))
    }

    fn write_blob(&self, content: &[u8]) -> Result<OidBytes, GitError> {
        self.inner
            .write_blob(content)
            .map(|id| OidBytes::from_gix(id.detach()))
            .map_err(|e| GitError::BlobWrite(e.to_string()))
    }

    fn insert_blob_at_path(
        &self,
        tree: OidBytes,
        path: &[&str],
        blob: OidBytes,
    ) -> Result<OidBytes, GitError> {
        insert_blob_into_gix_tree(&self.inner, tree.to_gix()?, path, blob.to_gix()?)
            .map(OidBytes::from_gix)
            .map_err(|e| GitError::TreeWrite(e.to_string()))
    }

    fn create_commit(
        &self,
        author:  &CommitAuthor,
        message: &str,
        tree:    OidBytes,
        parent:  OidBytes,
    ) -> Result<OidBytes, GitError> {
        let time   = gix::date::Time::now_local_or_utc();
        let offset = time.offset.unsigned_abs();
        let time_str = format!(
            "{} {}{:02}{:02}",
            time.seconds,
            if time.offset >= 0 { '+' } else { '-' },
            offset / 3600,
            (offset % 3600) / 60,
        );
        let sig_ref = gix::actor::SignatureRef {
            name:  gix::bstr::BStr::new(author.name.as_bytes()),
            email: gix::bstr::BStr::new(author.email.as_bytes()),
            time:  &time_str,
        };
        self.inner
            .commit_as(sig_ref, sig_ref, "HEAD", message, tree.to_gix()?, [parent.to_gix()?])
            .map(|id| OidBytes::from_gix(id.detach()))
            .map_err(|e| GitError::Commit(e.to_string()))
    }

    fn push_to_origin(&self, refspec: &str) -> Result<(), GitError> {
        // gix push API changed significantly in 0.80; delegate to the git CLI to stay stable.
        let workdir = self.inner
            .workdir()
            .ok_or_else(|| GitError::Push("bare repository has no work dir".into()))?;

        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(workdir)
            .arg("push")
            .arg("origin")
            .arg(refspec)
            .status()
            .map_err(|e| GitError::Push(e.to_string()))?;

        if !status.success() {
            return Err(GitError::Push(format!("git push exited with {status}")));
        }

        Ok(())
    }
}

// ── Tree helper (gix-internal, not exported) ──────────────────────────────────

/// Recursively inserts `blob_id` at `path_components` inside `tree_id`,
/// creating intermediate subtrees as needed. Returns the new root tree OID.
fn insert_blob_into_gix_tree(
    repo:            &gix::Repository,
    tree_id:         gix::ObjectId,
    path_components: &[&str],
    blob_id:         gix::ObjectId,
) -> Result<gix::ObjectId, Box<dyn std::error::Error>> {
    use gix::objs::tree::{Entry, EntryKind};
    use gix::objs::Tree;

    let existing_entries: Vec<Entry> =
        if tree_id == gix::ObjectId::empty_tree(repo.object_hash()) {
            vec![]
        } else {
            let obj  = repo.find_object(tree_id)?;
            let tree = obj.peel_to_tree()?;
            tree.decode()?
                .entries
                .iter()
                .map(|e| Entry {
                    mode:     e.mode,
                    filename: e.filename.to_owned(),
                    oid:      e.oid.to_owned(),
                })
                .collect()
        };

    let target_name  = path_components[0].as_bytes();
    let mut entries  = existing_entries;

    if path_components.len() == 1 {
        let new_entry = Entry {
            mode:     EntryKind::Blob.into(),
            filename: target_name.into(),
            oid:      blob_id,
        };
        match entries.iter().position(|e| e.filename.as_slice() == target_name) {
            Some(pos) => entries[pos] = new_entry,
            None      => entries.push(new_entry),
        }
    } else {
        let sub_tree_id = entries
            .iter()
            .find(|e| e.filename.as_slice() == target_name)
            .map(|e| e.oid)
            .unwrap_or_else(|| gix::ObjectId::empty_tree(repo.object_hash()));

        let new_sub_tree_id =
            insert_blob_into_gix_tree(repo, sub_tree_id, &path_components[1..], blob_id)?;

        let new_entry = Entry {
            mode:     EntryKind::Tree.into(),
            filename: target_name.into(),
            oid:      new_sub_tree_id,
        };
        match entries.iter().position(|e| e.filename.as_slice() == target_name) {
            Some(pos) => entries[pos] = new_entry,
            None      => entries.push(new_entry),
        }
    }

    // Git requires lexicographic order within a tree.
    entries.sort_by(|a, b| a.filename.cmp(&b.filename));

    let new_tree_id = repo.write_object(&Tree { entries })?.detach();
    Ok(new_tree_id)
}
