#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
// fs-manager-wiki — Outline + Wiki.js setup wizard and service controller.
//
// Design Pattern: State Machine (WikiSetupWizard)
//               + Strategy (WikiProvider for Outline vs Wiki.js)
//               + Composite (CategoryManager)
//
// Responsibilities:
//   - Guide the admin through initial wiki configuration after install
//   - Steps: Platform → Domain → Oidc → S3 → Confirm → Done
//   - Both Outline and Wiki.js are supported via WikiProvider trait
//   - ServiceController + CategoryManager for both implementations
//
// # Modules
//
//   - [`wizard`]   — WikiSetupWizard state machine
//   - [`config`]   — WikiConfig + WikiPlatform + WikiConfigStore
//   - [`error`]    — WikiManagerError
//   - [`provider`] — WikiProvider trait + OutlineAdapter + WikiJsAdapter
//   - [`keys`]     — FTL key name constants

pub mod config;
pub mod error;
pub mod keys;
pub mod provider;
pub mod service_controller;
pub mod view;
pub mod wizard;

pub use config::{OidcConfig, S3Config, WikiConfig, WikiConfigStore, WikiPlatform};
pub use error::WikiManagerError;
pub use provider::{provider_for, OutlineAdapter, WikiJsAdapter, WikiProvider};
pub use service_controller::WikiCategoryController;
pub use wizard::{WikiSetupWizard, WizardOutcome, WizardStep};
