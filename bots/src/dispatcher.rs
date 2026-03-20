// Command dispatcher — routes /commands to BotCommand handlers with FSN rights check.
//
// The dispatcher:
// 1. Receives an IncomingMessage from the polling loop or webhook handler
// 2. Builds a CommandContext (resolves FSN user, right level)
// 3. Delegates to CommandRegistry::dispatch()
// 4. Sends the BotResponse back through the appropriate channel adapter
// 5. Writes the result to the AuditLog

use std::sync::Arc;

use fsn_bot::{CommandContext, CommandRegistry};
use fsn_bot::response::BotResponse;
use fsn_bot::rights::Right;
use fsn_channel::{BotChannel, IncomingMessage, MessageFormat, RoomId};
use fsn_types::resources::MessengerKind;

use crate::audit::AuditLog;

/// Routes incoming messages to registered bot commands.
pub struct CommandDispatcher {
    registry: Arc<CommandRegistry>,
    audit: AuditLog,
}

impl CommandDispatcher {
    pub fn new(registry: Arc<CommandRegistry>, audit: AuditLog) -> Self {
        Self { registry, audit }
    }

    /// Handle one incoming message.
    ///
    /// - If the message is not a command, does nothing.
    /// - Resolves the caller's FSN right (stub: always `Member`; real IAM lookup in Phase P).
    /// - Dispatches to the registry.
    /// - Sends the response back through `channel`.
    pub async fn handle(
        &self,
        msg: IncomingMessage,
        platform: MessengerKind,
        channel: &dyn BotChannel,
    ) {
        if !msg.is_command {
            return;
        }

        let cmd_name = msg.command.clone().unwrap_or_default();

        // TODO Phase P: resolve FSN user and right from IAM bridge.
        // For now: public commands work for everyone, others require Member.
        let caller_right = Right::Member;

        let ctx = CommandContext {
            command: cmd_name.clone(),
            args: msg.command_args.clone(),
            fsn_user_id: None,
            caller_right,
            platform,
            extra: serde_json::Value::Null,
            message: msg.clone(),
        };

        let response = match self.registry.dispatch(ctx).await {
            Some(r) => r,
            None => return,
        };

        let result_label = match &response {
            BotResponse::Error(_) => "error",
            BotResponse::Silent => "silent",
            _ => "ok",
        };

        self.audit.user_action(
            msg.sender.as_str(),
            platform.label(),
            msg.room.as_str(),
            &format!("command.{}", cmd_name),
            None,
            result_label,
            None,
        ).await;

        send_response(response, &msg.room, channel).await;
    }
}

/// Recursively send a BotResponse through the channel adapter.
async fn send_response(response: BotResponse, default_room: &RoomId, channel: &dyn BotChannel) {
    match response {
        BotResponse::Message { room, text, format } => {
            let target = room.as_ref().unwrap_or(default_room);
            let res = match format {
                MessageFormat::Plain => channel.send(target, &text).await,
                _ => channel.send_formatted(target, &text, format).await,
            };
            if let Err(e) = res {
                tracing::error!("send_response failed: {}", e);
            }
        }
        BotResponse::Menu { room, text, buttons } => {
            let target = room.as_ref().unwrap_or(default_room);
            if let Err(e) = channel.send_menu(target, &text, &buttons).await {
                tracing::error!("send_menu failed: {}", e);
            }
        }
        BotResponse::Dm { user, text } => {
            if let Err(e) = channel.send_dm(&user, &text).await {
                tracing::error!("send_dm failed: {}", e);
            }
        }
        BotResponse::Many(responses) => {
            for r in responses {
                Box::pin(send_response(r, default_room, channel)).await;
            }
        }
        BotResponse::Silent => {}
        BotResponse::Error(msg) => {
            if let Err(e) = channel.send(default_room, &format!("Error: {}", msg)).await {
                tracing::error!("send error message failed: {}", e);
            }
        }
    }
}
