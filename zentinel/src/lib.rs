#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
// fs-manager-zentinel — Zentinel reverse-proxy manager.
//
// Design Pattern: Facade (ZentinelManager as Facade over Zentinel API)
//
// Responsibilities:
//   - Hold the route table (ServiceId → RouteConfig)
//   - Add / remove / update routes via Zentinel Admin API
//   - Auto-register routes when a new service registers in fs-registry
//   - View wired to fs-render (view.rs only)
//
// # Modules
//
//   - [`route`]   — RouteConfig + RouteTable
//   - [`manager`] — ZentinelManager (Facade)
//   - [`error`]   — ZentinelManagerError
//   - [`view`]    — FsView + ManagerLayout (only file importing fs-render)

pub mod bus_handler;
pub mod error;
pub mod manager;
pub mod route;
pub mod view;

pub use bus_handler::{ServiceRegisteredPayload, ServiceStoppedPayload, ZentinelBusHandler};
pub use error::ZentinelManagerError;
pub use manager::ZentinelManager;
pub use route::{RouteConfig, RouteProtocol, RouteTable};
