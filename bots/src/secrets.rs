// Secrets resolver — loads tokens from the Secrets Store, never from disk in plaintext.
//
// In production the Secrets Store is backed by a secrets manager
// (e.g. systemd-creds, age-encrypted files, or an external vault).
// For now we read from environment variables as a safe default.

use anyhow::{bail, Result};

/// Resolves a secret reference to its runtime value.
///
/// `reference` may be:
/// - `env:<VAR_NAME>` → read from environment variable
/// - `file:<path>`    → read from a file (whitespace-trimmed)
///
/// Plain-text values are NEVER accepted to prevent accidental credential leakage.
pub fn resolve(reference: &str) -> Result<String> {
    if let Some(var) = reference.strip_prefix("env:") {
        match std::env::var(var) {
            Ok(val) if !val.is_empty() => Ok(val),
            Ok(_) => bail!("Secret env:{} is set but empty", var),
            Err(_) => bail!("Secret env:{} is not set", var),
        }
    } else if let Some(path) = reference.strip_prefix("file:") {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read secret file '{}': {}", path, e))?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            bail!("Secret file '{}' is empty", path);
        }
        Ok(trimmed.to_owned())
    } else {
        bail!(
            "Invalid secret reference '{}'. Use 'env:<VAR>' or 'file:<path>'.",
            reference
        )
    }
}

/// Resolve a list of named secrets and return as a map.
///
/// `pairs` is a slice of `(name, reference)` tuples.
/// Returns `Err` immediately if any secret cannot be resolved.
pub fn resolve_map<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();
    for (name, reference) in pairs {
        let value = resolve(reference)
            .map_err(|e| anyhow::anyhow!("Secret '{}': {}", name, e))?;
        map.insert(name.to_owned(), value);
    }
    Ok(map)
}
