// Webhook server — receives push-based updates from webhook-only messengers.
//
// Webhook-only platforms (WhatsApp, LINE, Viber, Teams, Threema) push events
// to an HTTPS endpoint instead of allowing polling.  This module provides an
// axum router that the runtime mounts at /webhook/<platform>.
//
// Parsed IncomingMessages are forwarded to a Tokio broadcast channel so the
// runtime's main loop picks them up alongside polled messages.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::post;
use fsn_channel::{IncomingMessage, RoomId, UserId};
use fsn_types::resources::MessengerKind;
use tokio::sync::broadcast;

// ── WebhookState ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct WebhookState {
    pub tx: broadcast::Sender<(MessengerKind, IncomingMessage)>,
}

// ── Router ────────────────────────────────────────────────────────────────────

/// Build the webhook axum router.
///
/// Each platform gets its own sub-path: `/webhook/telegram`, `/webhook/whatsapp`, …
pub fn router(state: WebhookState) -> Router {
    Router::new()
        .route("/webhook/:platform", post(handle_webhook))
        .with_state(state)
}

// ── Handler ───────────────────────────────────────────────────────────────────

async fn handle_webhook(
    Path(platform_str): Path<String>,
    State(state): State<WebhookState>,
    body: axum::body::Bytes,
) -> StatusCode {
    let kind = match platform_str.as_str() {
        "telegram"       => MessengerKind::Telegram,
        "whatsapp"       => MessengerKind::WhatsApp,
        "line"           => MessengerKind::Line,
        "viber"          => MessengerKind::Viber,
        "teams"          => MessengerKind::Teams,
        "threema"        => MessengerKind::Threema,
        "discord"        => MessengerKind::Discord,
        "slack"          => MessengerKind::Slack,
        "signal"         => MessengerKind::Signal,
        "rocketchat"     => MessengerKind::RocketChat,
        "mattermost"     => MessengerKind::Mattermost,
        "matrix"         => MessengerKind::Matrix,
        "mastodon"       => MessengerKind::Mastodon,
        "revolt"         => MessengerKind::Revolt,
        "nextcloud"      => MessengerKind::NextcloudTalk,
        "zulip"          => MessengerKind::Zulip,
        "xmpp"           => MessengerKind::Xmpp,
        "irc"            => MessengerKind::Irc,
        "wire"           => MessengerKind::Wire,
        "discourse"      => MessengerKind::Discourse,
        "lemmy"          => MessengerKind::Lemmy,
        _ => {
            tracing::warn!("Webhook: unknown platform '{}'", platform_str);
            return StatusCode::NOT_FOUND;
        }
    };

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Webhook {}: invalid JSON: {}", platform_str, e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Platform-specific parsing
    if let Some(msg) = parse_webhook(kind, &payload) {
        let _ = state.tx.send((kind, msg));
    }

    StatusCode::OK
}

/// Parse a webhook payload into an IncomingMessage.
///
/// Each platform has its own format.  Returns `None` if the payload is not a
/// message event (e.g. delivery receipt, typing indicator).
fn parse_webhook(kind: MessengerKind, payload: &serde_json::Value) -> Option<IncomingMessage> {
    match kind {
        // ── Telegram ──────────────────────────────────────────────────────────
        MessengerKind::Telegram => {
            let msg = payload.get("message")?;
            let text = msg["text"].as_str().unwrap_or("").to_owned();
            let chat_id = msg["chat"]["id"].as_i64()?.to_string();
            let from = &msg["from"];
            let sender = from["id"].as_i64()?.to_string();
            let (is_cmd, cmd, args) = IncomingMessage::parse_command(&text);
            Some(IncomingMessage {
                id: msg["message_id"].as_u64()?.to_string(),
                next_offset: payload["update_id"].as_u64().unwrap_or(0) + 1,
                room: RoomId::new(chat_id),
                sender: UserId::new(sender),
                sender_name: format!(
                    "{} {}",
                    from["first_name"].as_str().unwrap_or(""),
                    from["last_name"].as_str().unwrap_or("")
                ).trim().to_owned(),
                text,
                timestamp: msg["date"].as_i64().unwrap_or(0),
                is_command: is_cmd,
                command: cmd,
                command_args: args,
                callback_payload: None,
                is_dm: msg["chat"]["type"].as_str() == Some("private"),
            })
        }

        // ── WhatsApp ──────────────────────────────────────────────────────────
        MessengerKind::WhatsApp => {
            let entry = payload["entry"].as_array()?.first()?;
            let change = entry["changes"].as_array()?.first()?;
            let value = &change["value"];
            let msg = value["messages"].as_array()?.first()?;
            if msg["type"].as_str() != Some("text") { return None; }
            let text = msg["text"]["body"].as_str().unwrap_or("").to_owned();
            let sender = msg["from"].as_str().unwrap_or("").to_owned();
            let phone_id = value["metadata"]["phone_number_id"].as_str().unwrap_or("").to_owned();
            let (is_cmd, cmd, args) = IncomingMessage::parse_command(&text);
            Some(IncomingMessage {
                id: msg["id"].as_str().unwrap_or("").to_owned(),
                next_offset: 0,
                room: RoomId::new(phone_id),
                sender: UserId::new(sender.clone()),
                sender_name: sender,
                text,
                timestamp: msg["timestamp"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
                is_command: is_cmd,
                command: cmd,
                command_args: args,
                callback_payload: None,
                is_dm: true,
            })
        }

        // ── LINE ──────────────────────────────────────────────────────────────
        MessengerKind::Line => {
            let event = payload["events"].as_array()?.first()?;
            if event["type"].as_str() != Some("message") { return None; }
            let msg = &event["message"];
            if msg["type"].as_str() != Some("text") { return None; }
            let text = msg["text"].as_str().unwrap_or("").to_owned();
            let source = &event["source"];
            let sender = source["userId"].as_str().unwrap_or("").to_owned();
            let room = source["groupId"].as_str()
                .or_else(|| source["roomId"].as_str())
                .unwrap_or(sender.as_str())
                .to_owned();
            let (is_cmd, cmd, args) = IncomingMessage::parse_command(&text);
            Some(IncomingMessage {
                id: msg["id"].as_str().unwrap_or("").to_owned(),
                next_offset: 0,
                room: RoomId::new(room),
                sender: UserId::new(sender.clone()),
                sender_name: sender,
                text,
                timestamp: event["timestamp"].as_i64().unwrap_or(0) / 1000,
                is_command: is_cmd,
                command: cmd,
                command_args: args,
                callback_payload: None,
                is_dm: source["type"].as_str() == Some("user"),
            })
        }

        // ── Viber ─────────────────────────────────────────────────────────────
        MessengerKind::Viber => {
            if payload["event"].as_str() != Some("message") { return None; }
            let msg = &payload["message"];
            if msg["type"].as_str() != Some("text") { return None; }
            let text = msg["text"].as_str().unwrap_or("").to_owned();
            let sender = &payload["sender"];
            let sender_id = sender["id"].as_str().unwrap_or("").to_owned();
            let chat_id = payload["chat_id"].as_str().unwrap_or(sender_id.as_str()).to_owned();
            let (is_cmd, cmd, args) = IncomingMessage::parse_command(&text);
            Some(IncomingMessage {
                id: msg["token"].as_str().unwrap_or("").to_owned(),
                next_offset: 0,
                room: RoomId::new(chat_id),
                sender: UserId::new(sender_id),
                sender_name: sender["name"].as_str().unwrap_or("").to_owned(),
                text,
                timestamp: payload["timestamp"].as_i64().unwrap_or(0) / 1000,
                is_command: is_cmd,
                command: cmd,
                command_args: args,
                callback_payload: payload["sender"]["id"].as_str().and_then(|_| {
                    msg.get("tracking_data").and_then(|v| v.as_str()).map(|s| s.to_owned())
                }),
                is_dm: false,
            })
        }

        // ── Threema ───────────────────────────────────────────────────────────
        MessengerKind::Threema => {
            // Threema Gateway callback
            let from = payload["from"].as_str().unwrap_or("").to_owned();
            let text = payload["text"].as_str().unwrap_or("").to_owned();
            let (is_cmd, cmd, args) = IncomingMessage::parse_command(&text);
            Some(IncomingMessage {
                id: payload["messageId"].as_str().unwrap_or("").to_owned(),
                next_offset: 0,
                room: RoomId::new(from.clone()),
                sender: UserId::new(from),
                sender_name: String::new(),
                text,
                timestamp: payload["date"].as_i64().unwrap_or(0),
                is_command: is_cmd,
                command: cmd,
                command_args: args,
                callback_payload: None,
                is_dm: true,
            })
        }

        // All other platforms: not parsed here (they are poll-based or handled elsewhere)
        _ => None,
    }
}
