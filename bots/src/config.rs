// Bot instance configuration — loaded from TOML at startup.

use fsn_channel::types::AdapterConfig;
use fsn_types::resources::MessengerKind;
use serde::{Deserialize, Serialize};

// ── BotInstanceConfig ─────────────────────────────────────────────────────────

/// Full configuration for one bot instance.
///
/// Loaded from `<data_dir>/bot.toml` when the runtime starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInstanceConfig {
    /// Human-readable display name.
    pub name: String,
    /// Unique instance ID (UUID v4).
    pub instance_id: String,
    /// Data directory for this instance's SQLite database and state.
    pub data_dir: String,
    /// Messenger connections this instance maintains.
    pub messengers: Vec<MessengerConfig>,
    /// FSN control level for incoming commands.
    pub fsn_level: FsnLevel,
}

/// Control access level for incoming commands.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FsnLevel {
    /// Anyone can trigger commands (unauthenticated OK).
    Public,
    /// Only FSN members can trigger commands.
    #[default]
    Member,
    /// Only operators.
    Operator,
    /// Only admins.
    Admin,
}

// ── MessengerConfig ───────────────────────────────────────────────────────────

/// Configuration for one messenger connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerConfig {
    /// Which platform.
    pub kind: MessengerKind,
    /// Runtime adapter config (token comes from secrets store at load time).
    pub adapter: AdapterConfig,
    /// Rooms / channels the bot should poll or respond in.
    pub rooms: Vec<String>,
    /// Messenger-level access control.
    pub access: MessengerAccess,
}

/// Who can interact with the bot on a given messenger platform.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MessengerAccess {
    /// Everyone in the room.
    #[default]
    Everyone,
    /// Only messenger admins.
    Admins,
    /// Nobody (bot only sends, never responds to commands).
    Nobody,
}

// ── RoomPolicy ────────────────────────────────────────────────────────────────

/// Per-room overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPolicy {
    pub room_id: String,
    /// Override commands allowed in this room.
    pub allowed_commands: Option<Vec<String>>,
    pub access: Option<MessengerAccess>,
}
