// Bus client — publishes bot.* events and queries the FSN bus REST API.

use anyhow::Result;
use serde_json::{json, Value};

const DEFAULT_BUS_URL: &str = "http://localhost:8081";

pub struct BusClient {
    base_url: String,
    client: reqwest::Client,
}

impl BusClient {
    pub fn new() -> Self {
        let base_url = std::env::var("FS_BUS_URL").unwrap_or_else(|_| DEFAULT_BUS_URL.to_string());
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Publish an event on the bus.
    pub async fn publish(&self, topic: &str, source_role: &str, payload: Value) -> Result<()> {
        let url = format!("{}/api/bus/publish", self.base_url);
        self.client
            .post(&url)
            .json(&json!({ "topic": topic, "source_role": source_role, "payload": payload }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Publish bot.status.request to ask bot instances to report their status.
    pub async fn request_bot_status(&self) -> Result<()> {
        self.publish("bot.status.request", "bot-manager", json!({}))
            .await
    }

    /// Publish bot.broadcast with a message to all groups.
    pub async fn broadcast(&self, message: &str) -> Result<()> {
        self.publish("bot.broadcast", "bot-manager", json!({ "text": message }))
            .await
    }

    /// Query recent bus events.
    #[allow(dead_code)]
    pub async fn recent_events(&self, limit: u32) -> Result<Vec<Value>> {
        let url = format!("{}/api/bus/events?limit={limit}", self.base_url);
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let events: Vec<Value> = resp.json().await?;
        Ok(events)
    }
}
