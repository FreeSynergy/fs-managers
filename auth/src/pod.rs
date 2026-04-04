// pod.rs — KanidmPodConfigurator: PodConfigurator impl for Kanidm.
//
// Design Pattern: Strategy (implements PodConfigurator)
// Delegates shared logic (validate / export_yaml / diff) to BasePodConfigurator.

use fs_pod_forge::{
    BasePodConfigurator, ManifestDiff, PodConfig, PodConfigurator, PodForgeError, PodManifest,
    PodManifestBuilder, ValidationResult,
};

// ── KanidmPodConfigurator ─────────────────────────────────────────────────────

/// Generates the `pod.yml` for the Kanidm IAM container.
///
/// Required config keys:
/// - `domain`    — Kanidm domain (e.g. `idm.example.com`)
/// - `https_port` — host port for HTTPS (default: `8443`)
pub struct KanidmPodConfigurator {
    base: BasePodConfigurator,
}

impl KanidmPodConfigurator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BasePodConfigurator::new("kanidm"),
        }
    }
}

impl Default for KanidmPodConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl PodConfigurator for KanidmPodConfigurator {
    fn package_name(&self) -> &'static str {
        "kanidm"
    }

    fn generate(&self, config: &PodConfig) -> Result<PodManifest, PodForgeError> {
        let domain = config
            .require("domain")
            .map_err(PodForgeError::Validation)?;
        let https_port: u16 = config
            .get("https_port")
            .unwrap_or("8443")
            .parse()
            .unwrap_or(8443);

        let manifest = PodManifestBuilder::new("kanidm-pod", "kanidm")
            .container("kanidm", "docker.io/kanidm/server:latest", |c| {
                c.port(https_port, 8443)
                    .mount("kanidm-data", "/data")
                    .env("KANIDM_DOMAIN", domain)
                    .env("KANIDM_ORIGIN", format!("https://{domain}"))
            })
            .volume("kanidm-data")
            .build();

        Ok(manifest)
    }

    fn validate(&self, manifest: &PodManifest) -> ValidationResult {
        self.base.validate(manifest)
    }

    fn export_yaml(&self, manifest: &PodManifest) -> Result<String, PodForgeError> {
        self.base.export_yaml(manifest)
    }

    fn diff(&self, old: &PodManifest, new: &PodManifest) -> ManifestDiff {
        self.base.diff(old, new)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn config(domain: &str) -> PodConfig {
        let mut c = PodConfig::default();
        c.values.insert("domain".into(), domain.into());
        c
    }

    #[test]
    fn generate_produces_valid_manifest() {
        let cfg = KanidmPodConfigurator::new();
        let manifest = cfg.generate(&config("idm.example.com")).unwrap();
        assert_eq!(manifest.pod_name, "kanidm-pod");
        assert_eq!(manifest.containers.len(), 1);
        assert_eq!(manifest.containers[0].ports[0].host_port, 8443);
    }

    #[test]
    fn missing_domain_is_error() {
        let cfg = KanidmPodConfigurator::new();
        assert!(cfg.generate(&PodConfig::default()).is_err());
    }

    #[test]
    fn custom_port_is_honoured() {
        let mut c = config("idm.example.com");
        c.values.insert("https_port".into(), "9443".into());
        let cfg = KanidmPodConfigurator::new();
        let manifest = cfg.generate(&c).unwrap();
        assert_eq!(manifest.containers[0].ports[0].host_port, 9443);
    }
}
