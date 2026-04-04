// app_config.rs — KanidmAppConfigurator: AppConfigurator impl for Kanidm.
//
// Design Pattern: Strategy (implements AppConfigurator)
// Schema-driven: the manager generates the UI form from schema() automatically.

use fs_app_forge::{
    validate_against_schema, AppConfigurator, AppForgeError, ConfigAdapter, ConfigChanges,
    ConfigField, ConfigFormat, ConfigSchema, ConfigSection, ConfigValues, FieldValue,
    TomlConfigAdapter, ValidationResult,
};

use fs_manager_core::ServiceController as _;

use crate::error::AuthManagerError;

// ── KanidmAppConfigurator ─────────────────────────────────────────────────────

/// Reads and writes the Kanidm TOML config file.
///
/// Config file location: `/etc/freesynergy/kanidm/config.toml`
pub struct KanidmAppConfigurator {
    config_path: String,
    adapter: TomlConfigAdapter,
}

impl KanidmAppConfigurator {
    /// Create a configurator pointing at the given config path.
    #[must_use]
    pub fn new(config_path: impl Into<String>) -> Self {
        Self {
            config_path: config_path.into(),
            adapter: TomlConfigAdapter,
        }
    }

    /// Create with the default system config path.
    #[must_use]
    pub fn default_path() -> Self {
        Self::new("/etc/freesynergy/kanidm/config.toml")
    }

    /// Build the Kanidm config schema.
    ///
    /// Exposed separately so tests can inspect the schema directly.
    #[must_use]
    pub fn build_schema() -> ConfigSchema {
        ConfigSchema::new(
            "kanidm",
            ConfigFormat::Toml,
            vec![
                ConfigSection::new("server", "app-forge-kanidm-section-server")
                    .field(ConfigField::required_string(
                        "domain",
                        "app-forge-kanidm-field-domain-label",
                        "app-forge-kanidm-field-domain-desc",
                    ))
                    .field(ConfigField::required_string(
                        "origin",
                        "app-forge-kanidm-field-origin-label",
                        "app-forge-kanidm-field-origin-desc",
                    ))
                    .field(ConfigField::optional_bool(
                        "trust_x_forward_for",
                        "app-forge-kanidm-field-trust-xff-label",
                        "app-forge-kanidm-field-trust-xff-desc",
                        false,
                    )),
                ConfigSection::new("admin", "app-forge-kanidm-section-admin")
                    .field(ConfigField::required_string(
                        "admin_username",
                        "app-forge-kanidm-field-admin-username-label",
                        "app-forge-kanidm-field-admin-username-desc",
                    ))
                    .field(ConfigField::secret(
                        "admin_password",
                        "app-forge-kanidm-field-admin-password-label",
                        "app-forge-kanidm-field-admin-password-desc",
                    )),
            ],
        )
    }
}

impl AppConfigurator for KanidmAppConfigurator {
    fn package_name(&self) -> &'static str {
        "kanidm"
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

// ── KanidmIamController ───────────────────────────────────────────────────────

/// IAM category manager for Kanidm.
///
/// Implements `ServiceController` and `CategoryManager` so the auth manager
/// can control the Kanidm systemd unit and advertise it as the active IAM provider.
pub struct KanidmIamController {
    controller: fs_manager_core::SystemdServiceController,
}

impl KanidmIamController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            controller: fs_manager_core::SystemdServiceController::new("pod-kanidm-pod.service"),
        }
    }
}

impl Default for KanidmIamController {
    fn default() -> Self {
        Self::new()
    }
}

#[fs_manager_core::async_trait]
impl fs_manager_core::ServiceController for KanidmIamController {
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
impl fs_manager_core::CategoryManager for KanidmIamController {
    fn category(&self) -> fs_manager_core::ServiceCategory {
        fs_manager_core::ServiceCategory::Iam
    }

    async fn list_all(
        &self,
    ) -> Result<Vec<fs_manager_core::ServiceInfo>, fs_manager_core::ManagerCoreError> {
        let kanidm_status = self
            .controller
            .status()
            .await
            .unwrap_or(fs_manager_core::ServiceStatus::Unknown);
        let kanidm_installed = !matches!(kanidm_status, fs_manager_core::ServiceStatus::Unknown);

        let keycloak = fs_manager_core::SystemdServiceController::new("pod-keycloak-pod.service");
        let keycloak_status = keycloak
            .status()
            .await
            .unwrap_or(fs_manager_core::ServiceStatus::Unknown);
        let keycloak_installed =
            !matches!(keycloak_status, fs_manager_core::ServiceStatus::Unknown);

        Ok(vec![
            fs_manager_core::ServiceInfo {
                id: "kanidm".into(),
                display_name: "Kanidm".into(),
                installed: kanidm_installed,
                is_primary: true,
                status: kanidm_status,
                version: None,
            },
            fs_manager_core::ServiceInfo {
                id: "keycloak".into(),
                display_name: "Keycloak".into(),
                installed: keycloak_installed,
                is_primary: false,
                status: keycloak_status,
                version: None,
            },
        ])
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
        let all = self.list_all().await?;
        let target = all.iter().find(|s| s.id == service_id);
        match target {
            Some(info) if info.installed => Ok(()), // fs-registry update happens here in G2
            Some(_) | None => Err(fs_manager_core::ManagerCoreError::NotInstalled(
                service_id.into(),
            )),
        }
    }
}

// ── AuthManagerError conversion ───────────────────────────────────────────────

impl From<AppForgeError> for AuthManagerError {
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
        let schema = KanidmAppConfigurator::build_schema();
        assert!(schema.field("domain").is_some());
    }

    #[test]
    fn schema_has_admin_password_as_secret() {
        use fs_app_forge::FieldType;
        let schema = KanidmAppConfigurator::build_schema();
        let f = schema.field("admin_password").unwrap();
        assert_eq!(f.field_type, FieldType::Secret);
    }
}
