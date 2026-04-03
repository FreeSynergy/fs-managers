// oidc.rs — OidcClientManager: post-wizard OIDC client management.
//
// Design Pattern: Command (each mutation is an explicit command; sync_to_kanidm
// applies pending changes to the live Kanidm instance via REST).
//
// Responsibilities:
//   - Track OIDC clients (add / remove) in local state
//   - Apply changes to a running Kanidm instance via admin bearer token
//   - Report sync errors without crashing — errors are stored, not panicked
//
// NOTE — gRPC-First exception:
//   This module uses reqwest + serde_json (raw REST) instead of gRPC because
//   Kanidm is a 3rd-party fork with no gRPC interface — only REST.
//   All FS-internal service-to-service calls MUST use gRPC (tonic).
//   Once fs-manager-auth runs as a standalone container, other FS programs
//   will call it via gRPC — this file remains the only REST boundary.

use crate::{config::OidcClient, error::AuthManagerError};

// ── Kanidm REST types ─────────────────────────────────────────────────────────

/// Minimal Kanidm `OAuth2` basic client creation request body.
#[derive(serde::Serialize)]
struct KanidmOAuth2BasicRequest {
    /// Unique client name (used as the client ID).
    name: String,
    /// Display name shown in the Kanidm consent screen.
    displayname: String,
    /// Origin URL — Kanidm uses this to derive the allowed redirect URIs.
    origin: String,
}

/// Partial Kanidm `OAuth2` list entry (only fields we need).
#[derive(serde::Deserialize)]
struct KanidmOAuth2Entry {
    /// Client name == client ID.
    name: String,
}

// ── SyncOutcome ───────────────────────────────────────────────────────────────

/// Result of one `sync_to_kanidm` run.
#[derive(Debug, Clone)]
pub struct SyncOutcome {
    /// Clients successfully registered.
    pub registered: Vec<String>,
    /// Clients successfully removed.
    pub removed: Vec<String>,
    /// Per-client errors (`client_id` → error message).
    pub errors: Vec<(String, String)>,
}

impl SyncOutcome {
    /// `true` if no errors occurred.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

// ── OidcClientManager ─────────────────────────────────────────────────────────

/// Manages OIDC clients on a running Kanidm instance after the setup wizard.
///
/// Local mutations (`add_client`, `remove_client`) queue changes in memory.
/// Call `sync_to_kanidm` to apply them to the live instance.
pub struct OidcClientManager {
    kanidm_url: String,
    admin_token: String,
    /// The desired set of OIDC clients (source of truth for the manager).
    clients: Vec<OidcClient>,
    /// Clients that need to be created on the next sync.
    pending_add: Vec<OidcClient>,
    /// Client IDs that need to be deleted on the next sync.
    pending_remove: Vec<String>,
}

impl OidcClientManager {
    /// Create a manager pointed at the given Kanidm instance.
    ///
    /// `clients` should be pre-populated from the stored configuration (e.g.
    /// the output of the setup wizard stored via fs-config).
    #[must_use]
    pub fn new(
        kanidm_url: impl Into<String>,
        admin_token: impl Into<String>,
        clients: Vec<OidcClient>,
    ) -> Self {
        Self {
            kanidm_url: kanidm_url.into(),
            admin_token: admin_token.into(),
            clients,
            pending_add: Vec::new(),
            pending_remove: Vec::new(),
        }
    }

    /// The current list of OIDC clients (desired state).
    #[must_use]
    pub fn clients(&self) -> &[OidcClient] {
        &self.clients
    }

    /// How many clients are waiting to be added on the next sync.
    #[must_use]
    pub fn pending_add_count(&self) -> usize {
        self.pending_add.len()
    }

    /// How many clients are waiting to be removed on the next sync.
    #[must_use]
    pub fn pending_remove_count(&self) -> usize {
        self.pending_remove.len()
    }

    // ── Commands ──────────────────────────────────────────────────────────────

    /// Queue a new OIDC client for registration.
    ///
    /// Validates input and rejects duplicates before queuing.
    ///
    /// # Errors
    /// - `Validation` — any field is empty
    /// - `Validation` — a client with the same ID already exists
    pub fn add_client(
        &mut self,
        id: &str,
        display_name: &str,
        redirect_uri: &str,
    ) -> Result<(), AuthManagerError> {
        let id = id.trim();
        let display_name = display_name.trim();
        let redirect_uri = redirect_uri.trim();

        if id.is_empty() || display_name.is_empty() || redirect_uri.is_empty() {
            return Err(AuthManagerError::Validation(
                "auth-wizard-error-oidc-fields-required".into(),
            ));
        }
        if self.clients.iter().any(|c| c.id == id) {
            return Err(AuthManagerError::Validation(
                "auth-manager-error-oidc-duplicate-id".into(),
            ));
        }

        let client = OidcClient {
            id: id.to_string(),
            display_name: display_name.to_string(),
            redirect_uri: redirect_uri.to_string(),
        };
        self.clients.push(client.clone());
        self.pending_add.push(client);
        Ok(())
    }

    /// Queue a client for removal by ID.
    ///
    /// # Errors
    /// - `Validation` — no client with that ID exists
    pub fn remove_client(&mut self, id: &str) -> Result<(), AuthManagerError> {
        let pos = self
            .clients
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| {
                AuthManagerError::Validation("auth-manager-error-oidc-not-found".into())
            })?;
        self.clients.remove(pos);
        // If the client was in pending_add it was never synced to Kanidm —
        // cancel both sides without queuing a removal.
        let was_only_pending = self.pending_add.iter().any(|c| c.id == id);
        self.pending_add.retain(|c| c.id != id);
        if !was_only_pending {
            self.pending_remove.push(id.to_string());
        }
        Ok(())
    }

    // ── Sync ──────────────────────────────────────────────────────────────────

    /// Apply all pending changes to the running Kanidm instance.
    ///
    /// Continues past individual failures and collects them in `SyncOutcome`.
    /// Returns `Err` only for fatal transport-level failures (e.g. cannot
    /// build the HTTP client).
    ///
    /// # Errors
    /// Returns `AuthManagerError::ConfigWrite` if the HTTP client cannot be built.
    pub async fn sync_to_kanidm(&mut self) -> Result<SyncOutcome, AuthManagerError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| AuthManagerError::ConfigWrite(e.to_string()))?;

        let mut outcome = SyncOutcome {
            registered: Vec::new(),
            removed: Vec::new(),
            errors: Vec::new(),
        };

        // Register new clients.
        let to_add = std::mem::take(&mut self.pending_add);
        for client in to_add {
            let body = KanidmOAuth2BasicRequest {
                name: client.id.clone(),
                displayname: client.display_name.clone(),
                origin: client.redirect_uri.clone(),
            };
            let resp = http
                .post(format!("{}/v1/oauth2/_basic", self.kanidm_url))
                .bearer_auth(&self.admin_token)
                .json(&body)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    outcome.registered.push(client.id.clone());
                }
                Ok(r) => {
                    outcome
                        .errors
                        .push((client.id.clone(), format!("HTTP {}", r.status())));
                    // Re-queue on failure so the user can retry.
                    self.pending_add.push(client);
                }
                Err(e) => {
                    outcome.errors.push((client.id.clone(), e.to_string()));
                    self.pending_add.push(client);
                }
            }
        }

        // Remove clients.
        let to_remove = std::mem::take(&mut self.pending_remove);
        for id in to_remove {
            let resp = http
                .delete(format!("{}/v1/oauth2/{id}", self.kanidm_url))
                .bearer_auth(&self.admin_token)
                .send()
                .await;
            match resp {
                Ok(r)
                    if r.status().is_success() || r.status() == reqwest::StatusCode::NOT_FOUND =>
                {
                    outcome.removed.push(id);
                }
                Ok(r) => {
                    outcome
                        .errors
                        .push((id.clone(), format!("HTTP {}", r.status())));
                    self.pending_remove.push(id);
                }
                Err(e) => {
                    outcome.errors.push((id.clone(), e.to_string()));
                    self.pending_remove.push(id);
                }
            }
        }

        Ok(outcome)
    }

    /// Fetch the current list of OIDC clients registered in Kanidm.
    ///
    /// This is a read-only query — it does not modify local state.
    ///
    /// # Errors
    /// Returns `AuthManagerError::ConfigWrite` on HTTP or parse failure.
    pub async fn fetch_from_kanidm(&self) -> Result<Vec<String>, AuthManagerError> {
        let http = reqwest::Client::new();
        let resp = http
            .get(format!("{}/v1/oauth2", self.kanidm_url))
            .bearer_auth(&self.admin_token)
            .send()
            .await
            .map_err(|e| AuthManagerError::ConfigWrite(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthManagerError::ConfigWrite(format!(
                "fetch_from_kanidm: HTTP {}",
                resp.status()
            )));
        }

        let entries: Vec<KanidmOAuth2Entry> = resp
            .json()
            .await
            .map_err(|e| AuthManagerError::ConfigWrite(e.to_string()))?;

        Ok(entries.into_iter().map(|e| e.name).collect())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> OidcClientManager {
        OidcClientManager::new("https://idm.example.com", "token", vec![])
    }

    #[test]
    fn add_client_appears_in_list() {
        let mut m = manager();
        m.add_client("forgejo", "Forgejo", "https://git.example.com/callback")
            .unwrap();
        assert_eq!(m.clients().len(), 1);
        assert_eq!(m.pending_add_count(), 1);
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut m = manager();
        m.add_client("forgejo", "Forgejo", "https://git.example.com/callback")
            .unwrap();
        let result = m.add_client("forgejo", "Forgejo 2", "https://other.example.com/cb");
        assert!(result.is_err());
    }

    #[test]
    fn remove_client_decrements_list() {
        let mut m = manager();
        m.add_client("forgejo", "Forgejo", "https://git.example.com/callback")
            .unwrap();
        m.remove_client("forgejo").unwrap();
        assert!(m.clients().is_empty());
        assert_eq!(m.pending_add_count(), 0, "pending_add cleared on remove");
        assert_eq!(
            m.pending_remove_count(),
            0,
            "pending_remove also empty because add was cancelled"
        );
    }

    #[test]
    fn remove_existing_queues_removal_without_add() {
        let existing = vec![OidcClient {
            id: "outline".to_string(),
            display_name: "Outline".to_string(),
            redirect_uri: "https://docs.example.com/callback".to_string(),
        }];
        let mut m = OidcClientManager::new("https://idm.example.com", "token", existing);
        m.remove_client("outline").unwrap();
        assert!(m.clients().is_empty());
        assert_eq!(m.pending_remove_count(), 1);
    }

    #[test]
    fn remove_unknown_id_is_error() {
        let mut m = manager();
        assert!(m.remove_client("nonexistent").is_err());
    }

    #[test]
    fn empty_fields_rejected() {
        let mut m = manager();
        assert!(m
            .add_client("", "Name", "https://x.example.com/cb")
            .is_err());
        assert!(m.add_client("id", "", "https://x.example.com/cb").is_err());
        assert!(m.add_client("id", "Name", "").is_err());
    }
}
