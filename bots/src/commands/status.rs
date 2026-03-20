// /status — shows the bot's connected messengers and their feature support.

use async_trait::async_trait;
use fsn_bot::{BotCommand, BotResponse, CommandContext};
use fsn_bot::rights::Right;

pub struct StatusCommand;

#[async_trait]
impl BotCommand for StatusCommand {
    fn name(&self) -> &str { "status" }
    fn description(&self) -> &str { "Show bot status and connected messengers." }
    fn required_right(&self) -> Right { Right::Member }
    fn usage(&self) -> Option<&str> { Some("/status") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let platform = ctx.platform.label();
        BotResponse::text(format!(
            "FreeSynergy Bot — online\nPlatform: {}\nType /help for available commands.",
            platform
        ))
    }
}
