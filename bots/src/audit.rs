// Audit log facade — async, backed by BotDb.

use std::sync::Arc;
use crate::db::BotDb;

/// Shared audit log (BotDb is already clone-able via sqlx pool).
#[derive(Clone)]
pub struct AuditLog {
    db: Arc<BotDb>,
}

impl AuditLog {
    pub fn new(db: Arc<BotDb>) -> Self {
        Self { db }
    }

    pub async fn user_action(
        &self,
        user_id: &str,
        platform: &str,
        room_id: &str,
        action: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) {
        if let Err(e) = self.db.audit("user", user_id, Some(platform), Some(room_id), action, target, result, detail).await {
            tracing::warn!("audit write failed: {}", e);
        }
    }

    pub async fn system_action(&self, action: &str, platform: Option<&str>, room_id: Option<&str>, result: &str, detail: Option<&str>) {
        if let Err(e) = self.db.audit("system", "system", platform, room_id, action, None, result, detail).await {
            tracing::warn!("audit write failed: {}", e);
        }
    }
}
