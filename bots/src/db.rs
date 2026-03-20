// Bot-manager DB — reads from the bot-runtime's fsn-botmanager.db (read-only for most queries).

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;

pub async fn connect(db_path: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=ro", db_path))?;
    Ok(SqlitePool::connect_with(opts).await?)
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct BotInstance {
    pub name:       String,
    pub bot_type:   String,
    pub data_dir:   String,
    pub status:     String,
    pub pid:        Option<i64>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct JoinRequest {
    pub id:         i64,
    pub platform:   String,
    pub room_id:    String,
    pub user_id:    String,
    pub status:     String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct Subscription {
    pub platform:   String,
    pub room_id:    String,
    pub topic:      String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct AuditEntry {
    pub id:         i64,
    pub actor_type: String,
    pub actor_id:   String,
    pub action:     String,
    pub result:     String,
    pub created_at: String,
}

// ── Queries ───────────────────────────────────────────────────────────────────

pub async fn list_instances(pool: &SqlitePool) -> Result<Vec<BotInstance>> {
    let rows = sqlx::query(
        "SELECT name, bot_type, data_dir, status, pid, created_at FROM child_bots ORDER BY name",
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| BotInstance {
        name:       r.get(0),
        bot_type:   r.get(1),
        data_dir:   r.get(2),
        status:     r.get(3),
        pid:        r.get(4),
        created_at: r.get(5),
    }).collect())
}

pub async fn list_pending_requests(pool: &SqlitePool) -> Result<Vec<JoinRequest>> {
    let rows = sqlx::query(
        "SELECT id, platform, room_id, user_id, status, created_at
         FROM join_requests WHERE status = 'pending' ORDER BY created_at",
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| JoinRequest {
        id:         r.get(0),
        platform:   r.get(1),
        room_id:    r.get(2),
        user_id:    r.get(3),
        status:     r.get(4),
        created_at: r.get(5),
    }).collect())
}

/// Open a read-write pool for mutation queries (approve/deny).
pub async fn connect_rw(db_path: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", db_path))?;
    Ok(SqlitePool::connect_with(opts).await?)
}

pub async fn approve_request(pool: &SqlitePool, id: i64, approve: bool) -> Result<bool> {
    let status = if approve { "approved" } else { "denied" };
    let res = sqlx::query(
        "UPDATE join_requests SET status = ?, resolved_at = datetime('now') WHERE id = ?",
    )
    .bind(status).bind(id).execute(pool).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn list_subscriptions(pool: &SqlitePool) -> Result<Vec<Subscription>> {
    let rows = sqlx::query(
        "SELECT platform, room_id, topic, created_at FROM subscriptions ORDER BY platform, room_id, topic",
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| Subscription {
        platform:   r.get(0),
        room_id:    r.get(1),
        topic:      r.get(2),
        created_at: r.get(3),
    }).collect())
}

pub async fn recent_audit(pool: &SqlitePool, limit: i64) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query(
        "SELECT id, actor_type, actor_id, action, result, created_at
         FROM audit_log ORDER BY id DESC LIMIT ?",
    )
    .bind(limit).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| AuditEntry {
        id:         r.get(0),
        actor_type: r.get(1),
        actor_id:   r.get(2),
        action:     r.get(3),
        result:     r.get(4),
        created_at: r.get(5),
    }).collect())
}
