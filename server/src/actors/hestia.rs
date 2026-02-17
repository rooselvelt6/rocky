// server/src/actors/hestia.rs
// Hestia: Persistencia y Cache (Valkey)

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;

pub struct Hestia {
    cached_items: u64,
    persisted_items: u64,
    messages_count: u64,
}

impl Hestia {
    pub fn new() -> Self {
        Self {
            cached_items: 0,
            persisted_items: 0,
            messages_count: 0,
        }
    }
}

#[async_trait]
impl OlympianActor for Hestia {
    fn name(&self) -> GodName {
        GodName::Hestia
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;

        match &msg.payload {
            MessagePayload::Command { action, data } => {
                match action.as_str() {
                    "cache_set" => {
                        self.cached_items += 1;
                        tracing::debug!("🏛️ Hestia: Cached item");
                    }
                    "persist" => {
                        self.persisted_items += 1;
                        tracing::debug!("🏛️ Hestia: Persisted item");
                    }
                    _ => {}
                }
                None
            }
            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Hestia,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Cache: {}, Persisted: {}", self.cached_items, self.persisted_items),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("🏛️ Hestia: Conectando a Valkey...");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🏛️ Hestia: Flush cache...");
        Ok(())
    }
}
