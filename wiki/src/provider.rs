// provider.rs — WikiProvider trait + concrete implementations.
//
// Design Pattern: Strategy
//   Both OutlineAdapter and WikiJsAdapter implement WikiProvider.
//   The caller receives a &dyn WikiProvider without knowing which is active.

use crate::config::WikiPlatform;

// ── WikiProvider ──────────────────────────────────────────────────────────────

/// Common interface for wiki platform adapters.
///
/// Provides platform-specific metadata and defaults used by the wizard
/// and service controller without making any network calls.
pub trait WikiProvider: Send + Sync {
    /// Stable service identifier (matches the Store package ID).
    fn provider_id(&self) -> &'static str;

    /// Human-readable display name.
    fn display_name(&self) -> &'static str;

    /// HTTP path used for readiness / health checks.
    fn health_path(&self) -> &'static str;

    /// Default TCP port the container exposes.
    fn default_port(&self) -> u16;

    /// Suggested default OIDC client ID for this platform.
    fn default_oidc_client_id(&self) -> &'static str;

    /// The [`WikiPlatform`] variant this adapter represents.
    fn platform(&self) -> WikiPlatform;
}

// ── OutlineAdapter ────────────────────────────────────────────────────────────

/// [`WikiProvider`] implementation for Outline.
pub struct OutlineAdapter;

impl WikiProvider for OutlineAdapter {
    fn provider_id(&self) -> &'static str {
        "outline"
    }

    fn display_name(&self) -> &'static str {
        "Outline"
    }

    fn health_path(&self) -> &'static str {
        "/_health"
    }

    fn default_port(&self) -> u16 {
        3000
    }

    fn default_oidc_client_id(&self) -> &'static str {
        "outline"
    }

    fn platform(&self) -> WikiPlatform {
        WikiPlatform::Outline
    }
}

// ── WikiJsAdapter ─────────────────────────────────────────────────────────────

/// [`WikiProvider`] implementation for Wiki.js.
pub struct WikiJsAdapter;

impl WikiProvider for WikiJsAdapter {
    fn provider_id(&self) -> &'static str {
        "wikijs"
    }

    fn display_name(&self) -> &'static str {
        "Wiki.js"
    }

    fn health_path(&self) -> &'static str {
        "/healthz"
    }

    fn default_port(&self) -> u16 {
        3000
    }

    fn default_oidc_client_id(&self) -> &'static str {
        "wikijs"
    }

    fn platform(&self) -> WikiPlatform {
        WikiPlatform::WikiJs
    }
}

// ── Factory ───────────────────────────────────────────────────────────────────

/// Return a boxed `WikiProvider` for the given platform.
#[must_use]
pub fn provider_for(platform: &WikiPlatform) -> Box<dyn WikiProvider> {
    match platform {
        WikiPlatform::Outline => Box::new(OutlineAdapter),
        WikiPlatform::WikiJs => Box::new(WikiJsAdapter),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_provider_id() {
        assert_eq!(OutlineAdapter.provider_id(), "outline");
    }

    #[test]
    fn wikijs_provider_id() {
        assert_eq!(WikiJsAdapter.provider_id(), "wikijs");
    }

    #[test]
    fn provider_for_returns_correct_type() {
        assert_eq!(
            provider_for(&WikiPlatform::Outline).provider_id(),
            "outline"
        );
        assert_eq!(provider_for(&WikiPlatform::WikiJs).provider_id(), "wikijs");
    }

    #[test]
    fn health_paths_differ() {
        assert_ne!(OutlineAdapter.health_path(), WikiJsAdapter.health_path());
    }
}
