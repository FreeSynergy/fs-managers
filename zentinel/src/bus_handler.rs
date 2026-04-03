// bus_handler.rs — ZentinelBusHandler: auto-routes newly registered services.
//
// Subscribes to `registry::service::registered` bus events.
// For each newly registered service, calls `ZentinelManager::auto_route_for_service`
// to add a Zentinel route automatically.
//
// Also handles `registry::service::stopped` — removes all routes for that service.
//
// Topic patterns:
//   registry::service::registered  → auto_route_for_service
//   registry::service::stopped     → remove_routes_for_service
//
// Unknown topics and malformed payloads are logged and skipped.

use std::sync::Arc;

use async_trait::async_trait;
use fs_bus::topics::{REGISTRY_SERVICE_REGISTERED, REGISTRY_SERVICE_STOPPED};
use fs_bus::{BusError, Event, TopicHandler};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

use crate::manager::ZentinelManager;

// ── Payload types (must match fs-registry bus_handler payloads) ───────────────

/// Payload of `registry::service::registered`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegisteredPayload {
    /// Service identifier, e.g. `"kanidm"`.
    pub service_id: String,
    /// Primary capability, e.g. `"iam.oidc-provider"`.
    pub capability: String,
    /// Base URL of the service, e.g. `"https://kanidm:8443"`.
    pub endpoint: String,
}

/// Payload of `registry::service::stopped`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceStoppedPayload {
    pub service_id: String,
}

// ── ZentinelBusHandler ────────────────────────────────────────────────────────

/// Subscribes to registry bus events and keeps the Zentinel route table in sync.
///
/// - `registry::service::registered` → auto-add a route via `ZentinelManager`
/// - `registry::service::stopped`    → remove all routes for that service
pub struct ZentinelBusHandler {
    manager: Arc<ZentinelManager>,
}

impl ZentinelBusHandler {
    /// Wrap a `ZentinelManager` in a bus handler.
    #[must_use]
    pub fn new(manager: Arc<ZentinelManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl TopicHandler for ZentinelBusHandler {
    fn topic_pattern(&self) -> &'static str {
        "registry::service::*"
    }

    #[instrument(
        name = "zentinel.bus_handler",
        skip(self, event),
        fields(topic = event.topic())
    )]
    async fn handle(&self, event: &Event) -> Result<(), BusError> {
        match event.topic() {
            REGISTRY_SERVICE_REGISTERED => {
                let payload: ServiceRegisteredPayload = match event.parse_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("zentinel: registry::service::registered — bad payload: {e}");
                        return Ok(());
                    }
                };

                match self.manager.auto_route_for_service(
                    &payload.service_id,
                    &payload.capability,
                    &payload.endpoint,
                ) {
                    Ok(route) => {
                        info!(
                            "zentinel: auto-route added — {} → {} ({})",
                            route.id, route.upstream, route.path
                        );
                    }
                    Err(e) => {
                        warn!(
                            "zentinel: auto-route failed for '{}': {e}",
                            payload.service_id
                        );
                    }
                }
            }

            REGISTRY_SERVICE_STOPPED => {
                let payload: ServiceStoppedPayload = match event.parse_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("zentinel: registry::service::stopped — bad payload: {e}");
                        return Ok(());
                    }
                };

                self.manager.remove_routes_for_service(&payload.service_id);
                info!(
                    "zentinel: routes removed for service '{}'",
                    payload.service_id
                );
            }

            other => {
                // Unrecognised sub-topic — not an error, just ignore.
                warn!("zentinel: unexpected topic '{other}' — skipped");
            }
        }

        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use fs_bus::Event;

    use super::*;

    fn manager() -> Arc<ZentinelManager> {
        Arc::new(ZentinelManager::offline())
    }

    fn make_event(topic: &str, payload: impl Serialize) -> Event {
        Event::new(topic, "test", payload).expect("test event")
    }

    #[tokio::test]
    async fn registered_event_adds_route() {
        let m = manager();
        let handler = ZentinelBusHandler::new(Arc::clone(&m));

        let event = make_event(
            REGISTRY_SERVICE_REGISTERED,
            ServiceRegisteredPayload {
                service_id: "kanidm".into(),
                capability: "iam.oidc-provider".into(),
                endpoint: "https://kanidm:8443".into(),
            },
        );

        handler.handle(&event).await.unwrap();
        assert_eq!(m.route_count(), 1);
        assert_eq!(m.routes()[0].id, "kanidm-auto");
        assert_eq!(m.routes()[0].path, "/auth");
    }

    #[tokio::test]
    async fn stopped_event_removes_routes() {
        let m = manager();
        let handler = ZentinelBusHandler::new(Arc::clone(&m));

        // First register.
        handler
            .handle(&make_event(
                REGISTRY_SERVICE_REGISTERED,
                ServiceRegisteredPayload {
                    service_id: "forgejo".into(),
                    capability: "git".into(),
                    endpoint: "http://forgejo:3000".into(),
                },
            ))
            .await
            .unwrap();
        assert_eq!(m.route_count(), 1);

        // Then stop.
        handler
            .handle(&make_event(
                REGISTRY_SERVICE_STOPPED,
                ServiceStoppedPayload {
                    service_id: "forgejo".into(),
                },
            ))
            .await
            .unwrap();
        assert_eq!(m.route_count(), 0);
    }

    #[tokio::test]
    async fn bad_payload_does_not_propagate_error() {
        let m = manager();
        let handler = ZentinelBusHandler::new(Arc::clone(&m));
        let event = Event::new(
            REGISTRY_SERVICE_REGISTERED,
            "test",
            serde_json::json!({"wrong": "fields"}),
        )
        .expect("test event");
        // Should not return Err — bad payloads are logged and skipped.
        let result = handler.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(m.route_count(), 0);
    }

    #[tokio::test]
    async fn unknown_topic_does_not_return_error() {
        let m = manager();
        let handler = ZentinelBusHandler::new(Arc::clone(&m));
        let event = make_event("registry::service::unknown", serde_json::json!({}));
        assert!(handler.handle(&event).await.is_ok());
    }
}
