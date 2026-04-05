#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
// fs-manager-forgejo — Forgejo Git forge setup wizard and service controller.
//
// Design Pattern: Adapter (ForgejoAdapter implements GitProvider)
//               + State Machine (ForgejoSetupWizard)
//               + Composite (ForgejoCategoryController implements CategoryManager)
//
// Responsibilities:
//   - Guide the admin through initial Forgejo configuration after install
//   - Steps: Domain → Ssh → Oidc → S3 → Confirm → Done
//   - Kanidm OIDC integration (mandatory)
//   - S3 storage for LFS / repository data (optional)
//   - ServiceController + CategoryManager for Forgejo
//
// # Modules
//
//   - [`adapter`]            — GitProvider trait + ForgejoAdapter
//   - [`wizard`]             — ForgejoSetupWizard state machine
//   - [`config`]             — ForgejoConfig + ForgejoConfigStore
//   - [`error`]              — ForgejoManagerError
//   - [`service_controller`] — ForgejoServiceController + ForgejoCategoryController
//   - [`keys`]               — FTL key name constants
//   - [`view`]               — FsView + ManagerLayout (only file that imports fs-render)

pub mod adapter;
pub mod config;
pub mod error;
pub mod keys;
pub mod service_controller;
pub mod view;
pub mod wizard;

pub use adapter::{ForgejoAdapter, GitProvider};
pub use config::{ForgejoConfig, ForgejoConfigStore, OidcConfig, S3Config};
pub use error::ForgejoManagerError;
pub use service_controller::{ForgejoCategoryController, ForgejoServiceController};
pub use wizard::{ForgejoSetupWizard, WizardOutcome, WizardStep};
