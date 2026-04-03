#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
// fs-manager-mail — Stalwart Mail setup wizard + domain configuration manager.
//
// Design Pattern: State Machine (StalwartSetupWizard with WizardStep trait)
//
// Responsibilities:
//   - Guide the admin through initial Stalwart configuration after install
//   - Steps: Domain → TlsCerts → OidcIntegration → Confirm → Done
//   - Produces a StalwartConfig ready to write via fs-config
//   - View wired to fs-render (view.rs only)
//
// # Modules
//
//   - [`wizard`]  — StalwartSetupWizard state machine + WizardStep trait
//   - [`config`]  — StalwartConfig (output of the wizard)
//   - [`error`]   — MailManagerError
//   - [`view`]    — FsView + ManagerLayout impl (only file importing fs-render)

pub mod config;
pub mod error;
pub mod view;
pub mod wizard;

pub use config::StalwartConfig;
pub use error::MailManagerError;
pub use wizard::{StalwartSetupWizard, WizardOutcome, WizardStep};
