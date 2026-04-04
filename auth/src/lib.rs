#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
// fs-manager-auth — Kanidm setup wizard + IAM configuration manager.
//
// Design Pattern: State Machine (KanidmSetupWizard with WizardStep-Trait)
//
// Responsibilities:
//   - Guide the admin through initial Kanidm configuration after install
//   - Steps: Domain → AdminAccount → OidcClients → Done
//   - Produces a KanidmConfig ready to write via fs-config
//   - View wired to fs-render (view.rs only)
//
// # Modules
//
//   - [`wizard`]     — KanidmSetupWizard state machine + WizardStep trait
//   - [`config`]     — KanidmConfig (output of the wizard)
//   - [`error`]      — AuthManagerError
//   - [`view`]       — FsView + ManagerLayout impl (only file importing fs-render)
//   - [`pod`]        — KanidmPodConfigurator (PodConfigurator impl)
//   - [`app_config`] — KanidmAppConfigurator (AppConfigurator impl) + KanidmIamController

pub mod app_config;
pub mod config;
pub mod error;
pub mod oidc;
pub mod pod;
pub mod view;
pub mod wizard;

pub use app_config::{KanidmAppConfigurator, KanidmIamController};
pub use config::KanidmConfig;
pub use error::AuthManagerError;
pub use oidc::{OidcClientManager, SyncOutcome};
pub use pod::KanidmPodConfigurator;
pub use wizard::{KanidmSetupWizard, WizardOutcome, WizardStep};
