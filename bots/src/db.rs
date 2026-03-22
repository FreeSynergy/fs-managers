// Bot-manager DB — reads from the bot-runtime's fs-botmanager.db (read-only for most queries).

use anyhow::Result;
use fs_db::{DbBackend, DbConnection, sea_orm::{Statement, Value}};

pub async fn connect(db_path: &str) -> Result<DbConnection> {
    let backend = DbBackend::Sqlite { path: db_path.to_string() };
    DbConnection::connect(backend).await.map_err(|e| anyhow::anyhow!(e.to_string()))
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

pub async fn list_instances(db: &DbConnection) -> Result<Vec<BotInstance>> {
    use fs_db::sea_orm::ConnectionTrait;
    let conn = db.inner();
    let stmt = Statement::from_string(
        conn.get_database_backend(),
        "SELECT name, bot_type, data_dir, status, pid, created_at FROM child_bots ORDER BY name",
    );
    let rows = conn.query_all_raw(stmt).await?;
    rows.into_iter()
        .map(|r| Ok(BotInstance {
            name:       r.try_get("", "name")?,
            bot_type:   r.try_get("", "bot_type")?,
            data_dir:   r.try_get("", "data_dir")?,
            status:     r.try_get("", "status")?,
            pid:        r.try_get("", "pid")?,
            created_at: r.try_get("", "created_at")?,
        }))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e: fs_db::sea_orm::DbErr| anyhow::anyhow!(e.to_string()))
}

pub async fn list_pending_requests(db: &DbConnection) -> Result<Vec<JoinRequest>> {
    use fs_db::sea_orm::ConnectionTrait;
    let conn = db.inner();
    let stmt = Statement::from_string(
        conn.get_database_backend(),
        "SELECT id, platform, room_id, user_id, status, created_at
         FROM join_requests WHERE status = 'pending' ORDER BY created_at",
    );
    let rows = conn.query_all_raw(stmt).await?;
    rows.into_iter()
        .map(|r| Ok(JoinRequest {
            id:         r.try_get("", "id")?,
            platform:   r.try_get("", "platform")?,
            room_id:    r.try_get("", "room_id")?,
            user_id:    r.try_get("", "user_id")?,
            status:     r.try_get("", "status")?,
            created_at: r.try_get("", "created_at")?,
        }))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e: fs_db::sea_orm::DbErr| anyhow::anyhow!(e.to_string()))
}

pub async fn approve_request(db: &DbConnection, id: i64, approve: bool) -> Result<bool> {
    use fs_db::sea_orm::ConnectionTrait;
    let conn = db.inner();
    let status = if approve { "approved" } else { "denied" };
    let stmt = Statement::from_sql_and_values(
        conn.get_database_backend(),
        "UPDATE join_requests SET status = $1, resolved_at = datetime('now') WHERE id = $2",
        [Value::from(status), Value::from(id)],
    );
    let result = conn.execute_raw(stmt).await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_subscriptions(db: &DbConnection) -> Result<Vec<Subscription>> {
    use fs_db::sea_orm::ConnectionTrait;
    let conn = db.inner();
    let stmt = Statement::from_string(
        conn.get_database_backend(),
        "SELECT platform, room_id, topic, created_at FROM subscriptions ORDER BY platform, room_id, topic",
    );
    let rows = conn.query_all_raw(stmt).await?;
    rows.into_iter()
        .map(|r| Ok(Subscription {
            platform:   r.try_get("", "platform")?,
            room_id:    r.try_get("", "room_id")?,
            topic:      r.try_get("", "topic")?,
            created_at: r.try_get("", "created_at")?,
        }))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e: fs_db::sea_orm::DbErr| anyhow::anyhow!(e.to_string()))
}

pub async fn recent_audit(db: &DbConnection, limit: i64) -> Result<Vec<AuditEntry>> {
    use fs_db::sea_orm::ConnectionTrait;
    let conn = db.inner();
    let stmt = Statement::from_sql_and_values(
        conn.get_database_backend(),
        "SELECT id, actor_type, actor_id, action, result, created_at
         FROM audit_log ORDER BY id DESC LIMIT $1",
        [Value::from(limit)],
    );
    let rows = conn.query_all_raw(stmt).await?;
    rows.into_iter()
        .map(|r| Ok(AuditEntry {
            id:         r.try_get("", "id")?,
            actor_type: r.try_get("", "actor_type")?,
            actor_id:   r.try_get("", "actor_id")?,
            action:     r.try_get("", "action")?,
            result:     r.try_get("", "result")?,
            created_at: r.try_get("", "created_at")?,
        }))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e: fs_db::sea_orm::DbErr| anyhow::anyhow!(e.to_string()))
}
