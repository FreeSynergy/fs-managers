#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
// fs-manager-matrix — Tuwunel Matrix setup wizard + Kanidm OIDC integration.
//
// Design Pattern: State Machine (TuwunelSetupWizard)
//
// Responsibilities:
//   - Guide the admin through initial Tuwunel configuration after install
//   - Steps: ServerName → TlsCerts → OidcIntegration → Federation → Confirm → Done
//   - Kanidm OIDC is REQUIRED for production (no local Matrix accounts)
//   - Produces a TuwunelConfig ready to write via fs-config
//   - View wired to fs-render (view.rs only)
//
// # Modules
//
//   - [`wizard`]     — TuwunelSetupWizard state machine
//   - [`config`]     — TuwunelConfig (output of the wizard)
//   - [`error`]      — MatrixManagerError
//   - [`view`]       — FsView + ManagerLayout impl (only file importing fs-render)
//   - [`app_config`] — TuwunelMessengerController (ServiceController + CategoryManager)

pub mod app_config;
pub mod config;
pub mod error;
pub mod view;
pub mod wizard;

pub use app_config::TuwunelMessengerController;
pub use config::TuwunelConfig;
pub use error::MatrixManagerError;
pub use wizard::{TuwunelSetupWizard, WizardOutcome, WizardStep};
