// Bot-instance SQLite database — schema and access layer.
//
// Uses sqlx (same version as sea-orm in fsn-inventory) to avoid
// libsqlite3-sys version conflicts.
//
// Database file: <data_dir>/fsn-botmanager.db

use anyhow::Result;
use chrono::Utc;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;

// ── Schema ────────────────────────────────────────────────────────────────────

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS bot_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    actor_type  TEXT    NOT NULL,
    actor_id    TEXT    NOT NULL,
    platform    TEXT,
    room_id     TEXT,
    action      TEXT    NOT NULL,
    target      TEXT,
    result      TEXT    NOT NULL,
    detail      TEXT,
    created_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    topic       TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    UNIQUE(platform, room_id, topic)
);

CREATE TABLE IF NOT EXISTS join_requests (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    user_id     TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'pending',
    iam_result  TEXT,
    created_at  TEXT    NOT NULL,
    resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS known_rooms (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    room_name   TEXT,
    member_count INTEGER,
    last_seen   TEXT    NOT NULL,
    UNIQUE(platform, room_id)
);

CREATE TABLE IF NOT EXISTS poll_state (
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    last_offset INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (platform, room_id)
);
"#;

// ── BotDb ─────────────────────────────────────────────────────────────────────

/// Async SQLite database handle for one bot instance.
#[derive(Clone)]
pub struct BotDb {
    pool: SqlitePool,
}

impl BotDb {
    /// Open (or create) the database and run migrations.
    pub async fn open(path: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path))?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePool::connect_with(opts).await?;
        sqlx::query(SCHEMA).execute(&pool).await?;
        Ok(Self { pool })
    }

    // ── Audit ─────────────────────────────────────────────────────────────────

    pub async fn audit(
        &self,
        actor_type: &str,
        actor_id: &str,
        platform: Option<&str>,
        room_id: Option<&str>,
        action: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (actor_type, actor_id, platform, room_id, action, target, result, detail, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(actor_type).bind(actor_id).bind(platform).bind(room_id)
        .bind(action).bind(target).bind(result).bind(detail)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Poll state ────────────────────────────────────────────────────────────

    pub async fn get_offset(&self, platform: &str, room_id: &str) -> Result<u64> {
        let row = sqlx::query(
            "SELECT last_offset FROM poll_state WHERE platform = ? AND room_id = ?"
        )
        .bind(platform).bind(room_id)
        .fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.get::<i64, _>(0) as u64).unwrap_or(0))
    }

    pub async fn set_offset(&self, platform: &str, room_id: &str, offset: u64) -> Result<()> {
        sqlx::query(
            "INSERT INTO poll_state (platform, room_id, last_offset) VALUES (?, ?, ?)
             ON CONFLICT(platform, room_id) DO UPDATE SET last_offset = excluded.last_offset"
        )
        .bind(platform).bind(room_id).bind(offset as i64)
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Known rooms ───────────────────────────────────────────────────────────

    pub async fn upsert_room(&self, platform: &str, room_id: &str, room_name: Option<&str>, member_count: Option<i64>) -> Result<()> {
        sqlx::query(
            "INSERT INTO known_rooms (platform, room_id, room_name, member_count, last_seen) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(platform, room_id) DO UPDATE SET room_name = excluded.room_name, member_count = excluded.member_count, last_seen = excluded.last_seen"
        )
        .bind(platform).bind(room_id).bind(room_name).bind(member_count)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────

    pub async fn subscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO subscriptions (platform, room_id, topic, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(platform).bind(room_id).bind(topic).bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn unsubscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        sqlx::query("DELETE FROM subscriptions WHERE platform = ? AND room_id = ? AND topic = ?")
            .bind(platform).bind(room_id).bind(topic)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn subscriptions_for_room(&self, platform: &str, room_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT topic FROM subscriptions WHERE platform = ? AND room_id = ?")
            .bind(platform).bind(room_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.get::<String, _>(0)).collect())
    }
}
