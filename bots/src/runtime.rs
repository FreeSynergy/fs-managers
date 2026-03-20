// BotRuntime — the core event loop.

use std::sync::Arc;
use std::time::Duration;

use fsn_channel::{BotChannel, ChannelRegistry};
use fsn_types::resources::MessengerKind;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::audit::AuditLog;
use crate::config::BotInstanceConfig;
use crate::db::BotDb;
use crate::dispatcher::CommandDispatcher;
use crate::trigger::TriggerEngine;
use crate::webhook::{self, WebhookState};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const WEBHOOK_CHANNEL_CAP: usize = 256;

// ── BotRuntime ────────────────────────────────────────────────────────────────

pub struct BotRuntime {
    config: BotInstanceConfig,
    dispatcher: Arc<CommandDispatcher>,
    trigger: Arc<TriggerEngine>,
    audit: AuditLog,
    db: Arc<BotDb>,
}

impl BotRuntime {
    pub fn new(
        config: BotInstanceConfig,
        dispatcher: CommandDispatcher,
        trigger: TriggerEngine,
        db: Arc<BotDb>,
        audit: AuditLog,
    ) -> Self {
        Self {
            config,
            dispatcher: Arc::new(dispatcher),
            trigger: Arc::new(trigger),
            audit,
            db,
        }
    }

    pub async fn run(self) {
        info!("Bot '{}' starting (id={})", self.config.name, self.config.instance_id);

        let (webhook_tx, _) = broadcast::channel::<(MessengerKind, fsn_channel::IncomingMessage)>(WEBHOOK_CHANNEL_CAP);
        let webhook_state = WebhookState { tx: webhook_tx.clone() };

        let webhook_port: u16 = std::env::var("FSN_BOT_WEBHOOK_PORT")
            .ok().and_then(|s| s.parse().ok()).unwrap_or(9090);
        let webhook_router = webhook::router(webhook_state);
        let webhook_addr = format!("0.0.0.0:{}", webhook_port);
        tokio::spawn(async move {
            info!("Webhook server listening on {}", webhook_addr);
            let listener = tokio::net::TcpListener::bind(&webhook_addr).await
                .expect("Failed to bind webhook port");
            axum::serve(listener, webhook_router).await.expect("Webhook server crashed");
        });

        // Spawn polling tasks
        for mc in &self.config.messengers {
            let Some(adapter) = ChannelRegistry::build_bot(mc.kind, mc.adapter.clone()) else {
                warn!("Adapter {:?} not compiled — skipping", mc.kind);
                continue;
            };
            if !adapter.bot_features().polling {
                info!("{} is webhook-only — no polling", mc.kind.label());
                continue;
            }
            let dispatcher = Arc::clone(&self.dispatcher);
            let audit = self.audit.clone();
            let db = Arc::clone(&self.db);
            let rooms = mc.rooms.clone();
            let kind = mc.kind;
            tokio::spawn(async move {
                poll_loop(adapter, kind, rooms, dispatcher, audit, db).await;
            });
        }

        // Receive webhook messages
        let mut webhook_rx = webhook_tx.subscribe();
        let dispatcher_wh = Arc::clone(&self.dispatcher);
        let messenger_configs = self.config.messengers.clone();
        tokio::spawn(async move {
            loop {
                match webhook_rx.recv().await {
                    Ok((kind, msg)) => {
                        if let Some(mc) = messenger_configs.iter().find(|m| m.kind == kind) {
                            if let Some(adapter) = ChannelRegistry::build_bot(mc.kind, mc.adapter.clone()) {
                                dispatcher_wh.handle(msg, kind, adapter.as_ref()).await;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => warn!("Webhook channel lagged by {}", n),
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        info!("Bot '{}' running", self.config.name);
        self.audit.system_action("runtime.start", None, None, "ok", None).await;

        tokio::signal::ctrl_c().await.expect("Failed to listen for SIGINT");
        info!("Shutdown — stopping bot '{}'", self.config.name);
        self.audit.system_action("runtime.stop", None, None, "ok", None).await;
    }
}

// ── poll_loop ─────────────────────────────────────────────────────────────────

async fn poll_loop(
    adapter: Box<dyn BotChannel>,
    kind: MessengerKind,
    rooms: Vec<String>,
    dispatcher: Arc<CommandDispatcher>,
    audit: AuditLog,
    db: Arc<BotDb>,
) {
    info!("Polling loop started for {}", kind.label());
    loop {
        for room_id in &rooms {
            let offset = db.get_offset(kind.label(), room_id).await.unwrap_or(0);
            match adapter.receive_updates(offset).await {
                Ok(messages) => {
                    let mut max_offset = offset;
                    for msg in messages {
                        if msg.next_offset > max_offset { max_offset = msg.next_offset; }
                        dispatcher.handle(msg, kind, adapter.as_ref()).await;
                    }
                    if max_offset > offset {
                        if let Err(e) = db.set_offset(kind.label(), room_id, max_offset).await {
                            error!("Failed to update poll offset: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("{} poll error for {}: {}", kind.label(), room_id, e);
                    audit.system_action("poll.error", Some(kind.label()), Some(room_id), "error", Some(&e.to_string())).await;
                }
            }
        }
        sleep(POLL_INTERVAL).await;
    }
}
