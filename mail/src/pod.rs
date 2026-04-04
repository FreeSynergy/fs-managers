// pod.rs — StalwartPodConfigurator: PodConfigurator impl for Stalwart Mail.
//
// Design Pattern: Strategy (implements PodConfigurator)

use fs_pod_forge::{
    BasePodConfigurator, ManifestDiff, PodConfig, PodConfigurator, PodForgeError, PodManifest,
    PodManifestBuilder, ValidationResult,
};

/// Generates the `pod.yml` for the Stalwart Mail container.
///
/// Required config keys:
/// - `domain`     — mail domain (e.g. `mail.example.com`)
///
/// Optional config keys:
/// - `smtp_port`  — host port for SMTP (default: `25`)
/// - `imaps_port` — host port for IMAP over TLS (default: `993`)
/// - `https_port` — host port for HTTP admin UI (default: `8080`)
pub struct StalwartPodConfigurator {
    base: BasePodConfigurator,
}

impl StalwartPodConfigurator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BasePodConfigurator::new("stalwart"),
        }
    }
}

impl Default for StalwartPodConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl PodConfigurator for StalwartPodConfigurator {
    fn package_name(&self) -> &'static str {
        "stalwart"
    }

    fn generate(&self, config: &PodConfig) -> Result<PodManifest, PodForgeError> {
        let domain = config
            .require("domain")
            .map_err(PodForgeError::Validation)?;

        let smtp_port: u16 = config
            .get("smtp_port")
            .unwrap_or("25")
            .parse()
            .unwrap_or(25);
        let imaps_port: u16 = config
            .get("imaps_port")
            .unwrap_or("993")
            .parse()
            .unwrap_or(993);
        let https_port: u16 = config
            .get("https_port")
            .unwrap_or("8080")
            .parse()
            .unwrap_or(8080);

        let manifest = PodManifestBuilder::new("stalwart-pod", "stalwart")
            .container(
                "stalwart",
                "docker.io/stalwartlabs/mail-server:latest",
                |c| {
                    c.port(smtp_port, 25)
                        .port(imaps_port, 993)
                        .port(https_port, 8080)
                        .mount("stalwart-data", "/opt/stalwart-mail")
                        .env("TZ", "UTC")
                        .env("STALWART_DOMAIN", domain)
                },
            )
            .volume("stalwart-data")
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
        let cfg = StalwartPodConfigurator::new();
        let manifest = cfg.generate(&config("mail.example.com")).unwrap();
        assert_eq!(manifest.pod_name, "stalwart-pod");
        let ports: Vec<u16> = manifest.containers[0]
            .ports
            .iter()
            .map(|p| p.host_port)
            .collect();
        assert!(ports.contains(&25));
        assert!(ports.contains(&993));
    }

    #[test]
    fn missing_domain_is_error() {
        let cfg = StalwartPodConfigurator::new();
        assert!(cfg.generate(&PodConfig::default()).is_err());
    }
}
