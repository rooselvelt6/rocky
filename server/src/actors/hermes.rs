// server/src/actors/hermes.rs
// Hermes: Mensajería y Routing

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;

pub struct Hermes {
    routes: Vec<String>,
    messages_routed: u64,
    messages_count: u64,
}

impl Hermes {
    pub fn new() -> Self {
        Self {
            routes: vec!["Zeus", "Hades", "Poseidon", "Athena"].iter().map(|s| s.to_string()).collect(),
            messages_routed: 0,
            messages_count: 0,
        }
    }
}

#[async_trait]
impl OlympianActor for Hermes {
    fn name(&self) -> GodName {
        GodName::Hermes
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;
        self.messages_routed += 1;
        
        // Hermes solo loguea el routing
        tracing::debug!("📨 Hermes: Routing message from {:?} to {:?}", msg.from, msg.to);
        
        None // Hermes no responde, solo enruta
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Hermes,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Routing {} messages", self.messages_routed),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("📨 Hermes: Inicializando router...");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("📨 Hermes: {} mensajes enrutados", self.messages_routed);
        Ok(())
    }
}
