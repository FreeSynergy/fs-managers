// fs-bot — BotManager CLI for FreeSynergy
//
// Commands:
//   fs-bot status                      — List all bot instances + status
//   fs-bot broadcast <message>         — Send broadcast via Bus
//   fs-bot gatekeeper list             — List pending join requests
//   fs-bot gatekeeper approve <id>     — Approve join request
//   fs-bot gatekeeper deny   <id>      — Deny join request
//   fs-bot log [--limit N]             — Show recent audit log entries
//
// The CLI reads from the running bot instance's SQLite DB.
// DB path: $FS_BOT_DB or $HOME/.local/share/fsn/bots/main/fs-botmanager.db

use anyhow::{bail, Context, Result};

mod bus;
mod db;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = BotCli;
    match args.as_slice() {
        [cmd] if cmd == "status" => cli.status().await,
        [cmd, msg @ ..] if cmd == "broadcast" => cli.broadcast(&msg.join(" ")).await,
        [cmd, sub] if cmd == "gatekeeper" && sub == "list" => cli.gatekeeper_list().await,
        [cmd, sub, id] if cmd == "gatekeeper" && (sub == "approve" || sub == "deny") => {
            let id: i64 = id.parse().context("id must be a number")?;
            cli.gatekeeper_resolve(id, sub == "approve").await
        }
        [cmd] if cmd == "log" => cli.log(20).await,
        [cmd, limit_flag, n] if cmd == "log" && limit_flag == "--limit" => {
            let n: i64 = n.parse().context("limit must be a number")?;
            cli.log(n).await
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  fs-bot status");
            eprintln!("  fs-bot broadcast <message>");
            eprintln!("  fs-bot gatekeeper list");
            eprintln!("  fs-bot gatekeeper approve <id>");
            eprintln!("  fs-bot gatekeeper deny   <id>");
            eprintln!("  fs-bot log [--limit N]");
            std::process::exit(1);
        }
    }
}

// ── BotCli ────────────────────────────────────────────────────────────────────

struct BotCli;

impl BotCli {
    fn db_path(&self) -> String {
        if let Ok(p) = std::env::var("FS_BOT_DB") {
            return p;
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{home}/.local/share/fsn/bots/main/fs-botmanager.db")
    }

    async fn open_db(&self) -> Result<fs_db::DbConnection> {
        let path = self.db_path();
        db::connect(&path)
            .await
            .with_context(|| format!("Cannot open bot DB at '{path}'"))
    }

    async fn status(&self) -> Result<()> {
        let pool = self.open_db().await?;
        let instances = db::list_instances(&pool).await?;
        if instances.is_empty() {
            println!("No bot instances registered.");
            return Ok(());
        }
        println!("{:<20} {:<15} {:<10} {}", "Name", "Type", "Status", "PID");
        println!("{}", "-".repeat(60));
        for inst in &instances {
            let pid = inst
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into());
            println!(
                "{:<20} {:<15} {:<10} {}",
                inst.name, inst.bot_type, inst.status, pid
            );
        }
        println!("\n{} instance(s) total.", instances.len());

        let bus = bus::BusClient::new();
        if let Ok(()) = bus.request_bot_status().await {
            println!("[Bus] Status request sent — check bus events for responses.");
        }
        Ok(())
    }

    async fn broadcast(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            bail!("Broadcast message cannot be empty.");
        }
        bus::BusClient::new()
            .broadcast(text)
            .await
            .context("Failed to publish broadcast event")?;
        println!("Broadcast sent: \"{text}\"");
        Ok(())
    }

    async fn gatekeeper_list(&self) -> Result<()> {
        let pool = self.open_db().await?;
        let requests = db::list_pending_requests(&pool).await?;
        if requests.is_empty() {
            println!("No pending join requests.");
            return Ok(());
        }
        println!(
            "{:<6} {:<15} {:<30} {:<20} {}",
            "ID", "Platform", "Room", "User", "Waiting since"
        );
        println!("{}", "-".repeat(90));
        for r in &requests {
            println!(
                "{:<6} {:<15} {:<30} {:<20} {}",
                r.id, r.platform, r.room_id, r.user_id, r.created_at
            );
        }
        println!("\n{} pending request(s).", requests.len());
        Ok(())
    }

    async fn gatekeeper_resolve(&self, id: i64, approve: bool) -> Result<()> {
        let action = if approve { "approved" } else { "denied" };
        let path = self.db_path();
        let rw_pool = db::connect(&path)
            .await
            .with_context(|| format!("Cannot open bot DB at '{path}' (rw)"))?;
        let ok = db::approve_request(&rw_pool, id, approve)
            .await
            .with_context(|| format!("Failed to {action} request #{id}"))?;
        if ok {
            println!("Request #{id} {action}.");
        } else {
            bail!("Request #{id} not found or already resolved.");
        }
        Ok(())
    }

    async fn log(&self, limit: i64) -> Result<()> {
        let pool = self.open_db().await?;
        let entries = db::recent_audit(&pool, limit).await?;
        if entries.is_empty() {
            println!("No audit entries.");
            return Ok(());
        }
        println!(
            "{:<6} {:<8} {:<20} {:<30} {:<10} {}",
            "ID", "Actor", "Actor ID", "Action", "Result", "When"
        );
        println!("{}", "-".repeat(100));
        for e in &entries {
            println!(
                "{:<6} {:<8} {:<20} {:<30} {:<10} {}",
                e.id, e.actor_type, e.actor_id, e.action, e.result, e.created_at
            );
        }
        Ok(())
    }
}
