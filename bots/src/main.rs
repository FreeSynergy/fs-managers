// fsn-bot-runtime — FreeSynergy bot instance entry point.
//
// Usage: fsn-bot-runtime --config <path/to/bot.toml>

use std::sync::Arc;
use anyhow::{Context, Result};
use fsn_bot::CommandRegistry;
use tracing_subscriber::EnvFilter;

use fsn_bots::{
    audit::AuditLog,
    config::BotInstanceConfig,
    db::BotDb,
    dispatcher::CommandDispatcher,
    runtime::BotRuntime,
    trigger::TriggerEngine,
};

mod commands;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config_path = std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .unwrap_or_else(|| "bot.toml".to_owned());

    let config_str = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Cannot read config file '{}'", config_path))?;
    let config: BotInstanceConfig = toml::from_str(&config_str)
        .context("Invalid bot.toml")?;

    let db_path = format!("{}/fsn-botmanager.db", config.data_dir);
    let db = BotDb::open(&db_path).await
        .with_context(|| format!("Cannot open database '{}'", db_path))?;
    let db = Arc::new(db);

    let audit = AuditLog::new(Arc::clone(&db));

    let mut registry = CommandRegistry::new();
    commands::register_all(&mut registry);

    let dispatcher = CommandDispatcher::new(Arc::new(registry), audit.clone());
    let trigger = TriggerEngine::new(audit.clone());

    let runtime = BotRuntime::new(config, dispatcher, trigger, Arc::clone(&db), audit);
    runtime.run().await;

    Ok(())
}
