// server/src/actors/erinyes.rs
// Erinyes: Monitoreo, Heartbeats y Alertas

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;
use std::collections::HashMap;

pub struct Erinyes {
    heartbeats: HashMap<GodName, i64>,
    alerts_triggered: u64,
    messages_count: u64,
}

impl Erinyes {
    pub fn new() -> Self {
        let mut heartbeats = HashMap::new();
        // Inicializar heartbeats para todos los dioses
        for god in [
            GodName::Zeus, GodName::Hades, GodName::Poseidon,
            GodName::Athena, GodName::Hermes, GodName::Hestia,
            GodName::Apollo, GodName::Artemis, GodName::Hera,
            GodName::Ares, GodName::Hefesto, GodName::Chronos,
            GodName::Moirai, GodName::Chaos, GodName::Aurora,
            GodName::Aphrodite, GodName::Iris, GodName::Demeter,
            GodName::Dionysus, GodName::Erinyes,
        ] {
            heartbeats.insert(god, Utc::now().timestamp());
        }

        Self {
            heartbeats,
            alerts_triggered: 0,
            messages_count: 0,
        }
    }

    fn check_health(&self, god: GodName) -> bool {
        if let Some(last_beat) = self.heartbeats.get(&god) {
            let now = Utc::now().timestamp();
            let diff = now - *last_beat;
            diff < 60 // Considerar saludable si heartbeat en últimos 60 segundos
        } else {
            false
        }
    }
}

#[async_trait]
impl OlympianActor for Erinyes {
    fn name(&self) -> GodName {
        GodName::Erinyes
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;

        match &msg.payload {
            MessagePayload::Heartbeat { timestamp } => {
                self.heartbeats.insert(msg.from, timestamp.timestamp());
                tracing::debug!("💓 Erinyes: Heartbeat from {:?}", msg.from);
                None
            }

            MessagePayload::Query { query_type, .. } => {
                if query_type == "get_health" {
                    let mut health_data = Vec::new();
                    
                    for (god, _) in &self.heartbeats {
                        let healthy = self.check_health(*god);
                        if !healthy {
                            self.alerts_triggered += 1;
                        }
                        
                        health_data.push(serde_json::json!({
                            "god": god.as_str(),
                            "healthy": healthy,
                            "last_seen": self.heartbeats.get(god),
                        }));
                    }

                    return Some(ActorMessage::new(
                        GodName::Erinyes,
                        msg.from,
                        MessagePayload::Response {
                            success: true,
                            data: serde_json::json!({ "health": health_data }),
                            error: None,
                        }
                    ));
                }
                None
            }

            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        let healthy_count = self.heartbeats.keys()
            .filter(|g| self.check_health(**g))
            .count();

        GodHealth {
            name: GodName::Erinyes,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Monitoring {} gods, {} healthy", self.heartbeats.len(), healthy_count),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("👁️ Erinyes: Iniciando monitoreo de {} dioses...", self.heartbeats.len());
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("👁️ Erinyes: {} alertas generadas", self.alerts_triggered);
        Ok(())
    }
}
