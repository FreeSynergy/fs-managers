#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
// fs-manager-core — Shared abstractions for all FreeSynergy managers.
//
// Design Pattern: Command (ServiceCommand) + Strategy (ServiceController)
//               + Composite (CategoryManager)
//
// # Modules
//
//   - [`service`]              — ServiceController trait + ServiceStatus + ServiceCommand
//   - [`category`]             — CategoryManager trait + ServiceInfo + ServiceCategory
//   - [`systemd_controller`]   — SystemdServiceController (wraps systemctl)
//   - [`container_controller`] — ContainerServiceController (wraps podman pod)
//   - [`error`]                — ManagerCoreError

pub mod category;
pub mod container_controller;
pub mod error;
pub mod service;
pub mod systemd_controller;

pub use category::{CategoryManager, ServiceCategory, ServiceInfo};
pub use container_controller::ContainerServiceController;
pub use error::ManagerCoreError;
pub use service::{ServiceCommand, ServiceController, ServiceStatus};
pub use systemd_controller::SystemdServiceController;

// Re-export async_trait so consumers don't need to add it separately.
pub use async_trait::async_trait;
