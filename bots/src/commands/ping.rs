// /ping — responds with "Pong!" to confirm the bot is alive.

use async_trait::async_trait;
use fsn_bot::{BotCommand, BotResponse, CommandContext};
use fsn_bot::rights::Right;

pub struct PingCommand;

#[async_trait]
impl BotCommand for PingCommand {
    fn name(&self) -> &str { "ping" }
    fn description(&self) -> &str { "Check if the bot is alive." }
    fn required_right(&self) -> Right { Right::Public }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        BotResponse::text("Pong!")
    }
}
