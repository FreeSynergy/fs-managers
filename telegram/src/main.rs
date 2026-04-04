#![deny(clippy::all, clippy::pedantic, warnings)]
// fs-telegram — FreeSynergy Telegram Channel Adapter Manager CLI
//
// Commands:
//   fs-telegram setup              — Run the setup wizard (interactive)
//   fs-telegram status             — Show current config + connectivity
//   fs-telegram set-token <ref>    — Set bot token reference non-interactively
//   fs-telegram set-chats [id...]  — Set allowed chat IDs (empty = all)
//   fs-telegram show               — Print current config

use anyhow::{bail, Context, Result};
use fs_channel_telegram::config::{
    default_config_path, TelegramChannelConfig, TelegramConfigStore,
};
use fs_channel_telegram::keys;

mod wizard_cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config_path = std::env::var("FS_TELEGRAM_CONFIG")
        .map_or_else(|_| default_config_path(), std::path::PathBuf::from);

    let cli = TelegramCli { config_path };

    match args.as_slice() {
        [cmd] if cmd == "setup" => cli.setup(),
        [cmd] if cmd == "status" => cli.status().await,
        [cmd] if cmd == "show" => cli.show(),
        [cmd, token_ref] if cmd == "set-token" => cli.set_token(token_ref),
        [cmd, chat_ids @ ..] if cmd == "set-chats" => {
            let ids = chat_ids
                .iter()
                .map(|s| {
                    s.parse::<i64>()
                        .with_context(|| format!("invalid chat ID: {s}"))
                })
                .collect::<Result<Vec<_>>>()?;
            cli.set_chats(&ids)
        }
        _ => {
            eprintln!("{}", fs_i18n::t(keys::WIZARD_TITLE));
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  fs-telegram setup");
            eprintln!("  fs-telegram status");
            eprintln!("  fs-telegram show");
            eprintln!("  fs-telegram set-token <env:VAR | file:PATH>");
            eprintln!("  fs-telegram set-chats [chat_id...]");
            std::process::exit(1);
        }
    }
}

// ── TelegramCli ───────────────────────────────────────────────────────────────

struct TelegramCli {
    config_path: std::path::PathBuf,
}

impl TelegramCli {
    fn store(&self) -> TelegramConfigStore {
        TelegramConfigStore::new(&self.config_path)
    }

    fn setup(&self) -> Result<()> {
        wizard_cli::run_wizard(&self.config_path)
    }

    async fn status(&self) -> Result<()> {
        let cfg = self.store().load().context("Cannot load config")?;

        if cfg.bot_token_ref.is_empty() || cfg.bot_token_ref == "env:FS_TELEGRAM_BOT_TOKEN" {
            // Check if the env var is actually set before calling it unconfigured.
            let has_token = std::env::var("FS_TELEGRAM_BOT_TOKEN")
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            if !has_token && cfg.bot_token_ref == "env:FS_TELEGRAM_BOT_TOKEN" {
                println!("{}", fs_i18n::t(keys::STATUS_NO_TOKEN));
                return Ok(());
            }
        }

        // Resolve the token reference to verify connectivity.
        let token = match resolve_token(&cfg.bot_token_ref) {
            Ok(t) => t,
            Err(e) => {
                println!("{}", fs_i18n::t(keys::STATUS_NOT_CONNECTED));
                println!("  Reason: {e}");
                return Ok(());
            }
        };

        // Try connecting to Telegram to verify the token.
        match ping_telegram(&token).await {
            Ok(bot_name) => {
                println!("{}", fs_i18n::t(keys::STATUS_CONNECTED));
                println!("  Bot: @{bot_name}");
            }
            Err(e) => {
                println!("{}", fs_i18n::t(keys::STATUS_NOT_CONNECTED));
                println!("  Error: {e}");
            }
        }

        println!();
        print_config(&cfg);
        Ok(())
    }

    fn show(&self) -> Result<()> {
        let cfg = self.store().load().context("Cannot load config")?;
        print_config(&cfg);
        Ok(())
    }

    fn set_token(&self, token_ref: &str) -> Result<()> {
        if !token_ref.starts_with("env:") && !token_ref.starts_with("file:") {
            bail!("Token reference must start with 'env:' or 'file:'");
        }
        let store = self.store();
        let mut cfg = store.load().context("Cannot load config")?;
        token_ref.clone_into(&mut cfg.bot_token_ref);
        store.save(&cfg).context("Cannot save config")?;
        println!("Bot token reference updated.");
        Ok(())
    }

    fn set_chats(&self, chat_ids: &[i64]) -> Result<()> {
        let store = self.store();
        let mut cfg = store.load().context("Cannot load config")?;
        cfg.allowed_chat_ids = chat_ids.to_vec();
        store.save(&cfg).context("Cannot save config")?;
        if chat_ids.is_empty() {
            println!(
                "{}: {}",
                fs_i18n::t(keys::CONFIG_CHATS_LABEL),
                fs_i18n::t(keys::CONFIG_CHATS_ALL)
            );
        } else {
            println!(
                "{}: {}",
                fs_i18n::t(keys::CONFIG_CHATS_LABEL),
                chat_ids
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn print_config(cfg: &TelegramChannelConfig) {
    let chats = if cfg.allowed_chat_ids.is_empty() {
        fs_i18n::t(keys::CONFIG_CHATS_ALL).to_string()
    } else {
        cfg.allowed_chat_ids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!(
        "{}: {}",
        fs_i18n::t(keys::CONFIG_TOKEN_REF_LABEL),
        cfg.bot_token_ref
    );
    println!("{}: {}", fs_i18n::t(keys::CONFIG_CHATS_LABEL), chats);
}

fn resolve_token(token_ref: &str) -> Result<String> {
    if let Some(var) = token_ref.strip_prefix("env:") {
        std::env::var(var)
            .map_err(|_| anyhow::anyhow!("env var '{var}' not set"))
            .and_then(|v| {
                if v.is_empty() {
                    bail!("env var '{var}' is empty");
                }
                Ok(v)
            })
    } else if let Some(path) = token_ref.strip_prefix("file:") {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("cannot read '{path}'"))?;
        let trimmed = content.trim().to_owned();
        if trimmed.is_empty() {
            bail!("file '{path}' is empty");
        }
        Ok(trimmed)
    } else {
        bail!("invalid token reference '{token_ref}'")
    }
}

/// Ping the Telegram Bot API — returns the bot's username on success.
async fn ping_telegram(token: &str) -> Result<String> {
    let url = format!("https://api.telegram.org/bot{token}/getMe");
    let resp = reqwest::get(&url)
        .await
        .context("HTTP request to Telegram API failed")?
        .json::<serde_json::Value>()
        .await
        .context("Telegram API response is not valid JSON")?;

    if resp["ok"].as_bool().unwrap_or(false) {
        let username = resp["result"]["username"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned();
        Ok(username)
    } else {
        let desc = resp["description"]
            .as_str()
            .unwrap_or("unknown error")
            .to_owned();
        bail!("Telegram API error: {desc}");
    }
}
