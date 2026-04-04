// app_config.rs — StalwartAppConfigurator: AppConfigurator impl for Stalwart Mail.
//
// Design Pattern: Strategy (implements AppConfigurator)

use fs_app_forge::{
    validate_against_schema, AppConfigurator, AppForgeError, ConfigAdapter, ConfigChanges,
    ConfigField, ConfigFormat, ConfigSchema, ConfigSection, ConfigValues, FieldValue,
    TomlConfigAdapter, ValidationResult,
};

use fs_manager_core::ServiceController as _;

use crate::error::MailManagerError;

// ── StalwartAppConfigurator ───────────────────────────────────────────────────

/// Reads and writes the Stalwart Mail TOML config file.
///
/// Config file location: `/etc/freesynergy/stalwart/config.toml`
pub struct StalwartAppConfigurator {
    config_path: String,
    adapter: TomlConfigAdapter,
}

impl StalwartAppConfigurator {
    #[must_use]
    pub fn new(config_path: impl Into<String>) -> Self {
        Self {
            config_path: config_path.into(),
            adapter: TomlConfigAdapter,
        }
    }

    #[must_use]
    pub fn default_path() -> Self {
        Self::new("/etc/freesynergy/stalwart/config.toml")
    }

    #[must_use]
    pub fn build_schema() -> ConfigSchema {
        ConfigSchema::new(
            "stalwart",
            ConfigFormat::Toml,
            vec![
                ConfigSection::new("server", "app-forge-stalwart-section-server")
                    .field(ConfigField::required_string(
                        "domain",
                        "app-forge-stalwart-field-domain-label",
                        "app-forge-stalwart-field-domain-desc",
                    ))
                    .field(ConfigField::required_string(
                        "admin_email",
                        "app-forge-stalwart-field-admin-email-label",
                        "app-forge-stalwart-field-admin-email-desc",
                    )),
                ConfigSection::new("tls", "app-forge-stalwart-section-tls")
                    .field(ConfigField::optional_bool(
                        "tls.use_acme",
                        "app-forge-stalwart-field-use-acme-label",
                        "app-forge-stalwart-field-use-acme-desc",
                        true,
                    ))
                    .field(ConfigField {
                        key: "tls.cert_path".into(),
                        label_key: "app-forge-stalwart-field-cert-path-label".into(),
                        description_key: "app-forge-stalwart-field-cert-path-desc".into(),
                        field_type: fs_app_forge::FieldType::Path,
                        default: None,
                        required: false,
                        validator: None,
                    })
                    .field(ConfigField {
                        key: "tls.key_path".into(),
                        label_key: "app-forge-stalwart-field-key-path-label".into(),
                        description_key: "app-forge-stalwart-field-key-path-desc".into(),
                        field_type: fs_app_forge::FieldType::Path,
                        default: None,
                        required: false,
                        validator: None,
                    }),
                ConfigSection::new("oidc", "app-forge-stalwart-section-oidc")
                    .field(ConfigField::optional_bool(
                        "skip_oidc",
                        "app-forge-stalwart-field-skip-oidc-label",
                        "app-forge-stalwart-field-skip-oidc-desc",
                        false,
                    ))
                    .field(ConfigField::required_string(
                        "oidc.issuer_url",
                        "app-forge-stalwart-field-oidc-issuer-label",
                        "app-forge-stalwart-field-oidc-issuer-desc",
                    ))
                    .field(ConfigField::required_string(
                        "oidc.client_id",
                        "app-forge-stalwart-field-oidc-client-id-label",
                        "app-forge-stalwart-field-oidc-client-id-desc",
                    ))
                    .field(ConfigField::secret(
                        "oidc.client_secret",
                        "app-forge-stalwart-field-oidc-client-secret-label",
                        "app-forge-stalwart-field-oidc-client-secret-desc",
                    )),
            ],
        )
    }
}

impl AppConfigurator for StalwartAppConfigurator {
    fn package_name(&self) -> &'static str {
        "stalwart"
    }

    fn schema(&self) -> ConfigSchema {
        Self::build_schema()
    }

    fn read(&self) -> Result<ConfigValues, AppForgeError> {
        let flat = self.adapter.read(&self.config_path)?;
        let mut values = ConfigValues::default();
        for (k, v) in flat {
            values.set(k, FieldValue::Text(v));
        }
        Ok(values)
    }

    fn validate(&self, changes: &ConfigChanges) -> ValidationResult {
        validate_against_schema(&Self::build_schema(), changes)
    }

    fn apply(&self, changes: ConfigChanges) -> Result<(), AppForgeError> {
        let result = self.validate(&changes);
        if !result.is_valid() {
            let first = &result.errors[0];
            return Err(AppForgeError::Validation {
                field: first.field.clone(),
                reason: first.message_key.clone(),
            });
        }
        let flat: std::collections::HashMap<String, String> = changes
            .changes
            .into_iter()
            .filter_map(|(k, v)| match v {
                FieldValue::Text(s) => Some((k, s)),
                FieldValue::Bool(b) => Some((k, b.to_string())),
                FieldValue::Int(i) => Some((k, i.to_string())),
                _ => None,
            })
            .collect();
        self.adapter.write(&self.config_path, &flat)
    }

    fn export(&self) -> Result<String, AppForgeError> {
        self.adapter.export(&self.config_path)
    }

    fn config_path(&self) -> &str {
        &self.config_path
    }
}

// ── StalwartMailController ────────────────────────────────────────────────────

/// Mail category controller for Stalwart.
pub struct StalwartMailController {
    controller: fs_manager_core::SystemdServiceController,
}

impl StalwartMailController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            controller: fs_manager_core::SystemdServiceController::new("pod-stalwart-pod.service"),
        }
    }
}

impl Default for StalwartMailController {
    fn default() -> Self {
        Self::new()
    }
}

#[fs_manager_core::async_trait]
impl fs_manager_core::ServiceController for StalwartMailController {
    fn name(&self) -> &str {
        self.controller.name()
    }
    async fn start(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.start().await
    }
    async fn stop(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.stop().await
    }
    async fn restart(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.restart().await
    }
    async fn enable(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.enable().await
    }
    async fn disable(&self) -> Result<(), fs_manager_core::ManagerCoreError> {
        self.controller.disable().await
    }
    async fn status(
        &self,
    ) -> Result<fs_manager_core::ServiceStatus, fs_manager_core::ManagerCoreError> {
        self.controller.status().await
    }
}

// ── CategoryManager ───────────────────────────────────────────────────────────

#[fs_manager_core::async_trait]
impl fs_manager_core::CategoryManager for StalwartMailController {
    fn category(&self) -> fs_manager_core::ServiceCategory {
        fs_manager_core::ServiceCategory::Mail
    }

    async fn list_all(
        &self,
    ) -> Result<Vec<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let stalwart_status = self
            .controller
            .status()
            .await
            .unwrap_or(fs_manager_core::ServiceStatus::Unknown);
        let stalwart_installed =
            !matches!(stalwart_status, fs_manager_core::ServiceStatus::Unknown);

        Ok(vec![fs_manager_core::ServiceInfo {
            id: "stalwart".into(),
            display_name: "Stalwart Mail".into(),
            installed: stalwart_installed,
            is_primary: true,
            status: stalwart_status,
            version: None,
        }])
    }

    async fn list_running(
        &self,
    ) -> Result<Vec<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|s| s.status.is_running()).collect())
    }

    async fn get_active(
        &self,
    ) -> Result<Option<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|s| s.is_primary))
    }

    async fn set_active(&self, service_id: &str) -> Result<(), fs_manager_core::ManagerCoreError> {
        if service_id == "stalwart" {
            Ok(())
        } else {
            Err(fs_manager_core::ManagerCoreError::NotInstalled(
                service_id.into(),
            ))
        }
    }
}

// ── MailManagerError conversion ───────────────────────────────────────────────

impl From<AppForgeError> for MailManagerError {
    fn from(e: AppForgeError) -> Self {
        Self::Validation(e.to_string())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_has_domain_field() {
        let schema = StalwartAppConfigurator::build_schema();
        assert!(schema.field("domain").is_some());
    }

    #[test]
    fn schema_has_oidc_secret() {
        use fs_app_forge::FieldType;
        let schema = StalwartAppConfigurator::build_schema();
        let f = schema.field("oidc.client_secret").unwrap();
        assert_eq!(f.field_type, FieldType::Secret);
    }
}
