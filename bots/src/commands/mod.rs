// Built-in bot commands registered in every bot instance.
//
// Module-specific commands (broadcast, gatekeeper, calendar, …) are loaded
// dynamically as Store packages in N5+ phases.

use fsn_bot::CommandRegistry;

mod ping;
mod status;

/// Register all built-in commands into the registry.
pub fn register_all(registry: &mut CommandRegistry) {
    registry.register(ping::PingCommand);
    registry.register(status::StatusCommand);
}
