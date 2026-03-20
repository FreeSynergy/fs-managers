// Trigger engine — listens for Bus events and fires bot module handlers.
//
// The trigger engine subscribes to Bus topics and calls registered
// TriggerHandler implementations when a matching event arrives.

use std::sync::Arc;

use async_trait::async_trait;

use crate::audit::AuditLog;

// ── TriggerEvent ──────────────────────────────────────────────────────────────

/// A Bus event delivered to a trigger handler.
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    /// Bus topic, e.g. `"calendar.event.upcoming"`.
    pub topic: String,
    /// Event payload (JSON).
    pub payload: serde_json::Value,
}

// ── TriggerHandler ────────────────────────────────────────────────────────────

/// A handler that reacts to one or more Bus topics.
///
/// Each bot module that listens to Bus events implements this trait.
#[async_trait]
pub trait TriggerHandler: Send + Sync {
    /// Bus topic patterns this handler subscribes to (e.g. `"calendar.event.*"`).
    fn topics(&self) -> &[&str];

    /// Called when a matching event arrives.
    async fn on_event(&self, event: TriggerEvent);
}

// ── TriggerEngine ─────────────────────────────────────────────────────────────

/// Manages trigger handlers and dispatches Bus events to them.
pub struct TriggerEngine {
    handlers: Vec<Arc<dyn TriggerHandler>>,
    audit: AuditLog,
}

impl TriggerEngine {
    pub fn new(audit: AuditLog) -> Self {
        Self { handlers: Vec::new(), audit }
    }

    /// Register a trigger handler.
    pub fn register(&mut self, handler: impl TriggerHandler + 'static) {
        self.handlers.push(Arc::new(handler));
    }

    /// Dispatch a Bus event to all matching handlers.
    pub async fn dispatch(&self, event: TriggerEvent) {
        let topic = event.topic.clone();
        for handler in &self.handlers {
            if handler.topics().iter().any(|pat| topic_matches(pat, &topic)) {
                let h = Arc::clone(handler);
                let ev = event.clone();
                tokio::spawn(async move {
                    h.on_event(ev).await;
                });
            }
        }
        self.audit.system_action(
            &format!("trigger.dispatch:{}", topic),
            None, None, "ok", None,
        ).await;
    }

    /// All subscribed topics (deduplicated).
    pub fn subscribed_topics(&self) -> Vec<&str> {
        let mut topics: Vec<&str> = self.handlers.iter()
            .flat_map(|h| h.topics().iter().copied())
            .collect();
        topics.sort_unstable();
        topics.dedup();
        topics
    }
}

/// Simple glob-style topic matching: `*` matches a single segment, `**` matches any suffix.
fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == topic || pattern == "**" {
        return true;
    }
    let pat_parts: Vec<&str> = pattern.split('.').collect();
    let top_parts: Vec<&str> = topic.split('.').collect();
    if pat_parts.len() != top_parts.len() {
        // Allow ** as last segment to match any suffix
        if pat_parts.last() == Some(&"**") && top_parts.len() >= pat_parts.len() - 1 {
            return pat_parts[..pat_parts.len()-1].iter().zip(top_parts.iter())
                .all(|(p, t)| *p == "*" || p == t);
        }
        return false;
    }
    pat_parts.iter().zip(top_parts.iter()).all(|(p, t)| *p == "*" || p == t)
}

#[cfg(test)]
mod tests {
    use super::topic_matches;

    #[test]
    fn exact_match() { assert!(topic_matches("a.b.c", "a.b.c")); }
    #[test]
    fn star_segment() { assert!(topic_matches("a.*.c", "a.b.c")); }
    #[test]
    fn double_star_suffix() { assert!(topic_matches("a.**", "a.b.c.d")); }
    #[test]
    fn no_match() { assert!(!topic_matches("a.b", "a.b.c")); }
}
