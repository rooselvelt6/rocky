// server/src/actors/zeus.rs
// Zeus: Gobernador Supremo y Supervisor del Olimpo

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;
use std::collections::HashMap;

pub struct Zeus {
    supervised_actors: HashMap<GodName, bool>, // nombre -> salud
    restart_count: HashMap<GodName, u32>,
}

impl Zeus {
    pub fn new() -> Self {
        let mut supervised = HashMap::new();
        // Todos los dioses excepto Zeus mismo
        for god in [
            GodName::Hades, GodName::Poseidon, GodName::Athena,
            GodName::Hermes, GodName::Hestia, GodName::Erinyes,
            GodName::Apollo, GodName::Artemis, GodName::Hera,
            GodName::Ares, GodName::Hefesto, GodName::Chronos,
            GodName::Moirai, GodName::Chaos, GodName::Aurora,
            GodName::Aphrodite, GodName::Iris, GodName::Demeter,
            GodName::Dionysus,
        ] {
            supervised.insert(god, true);
        }

        Self {
            supervised_actors: supervised,
            restart_count: HashMap::new(),
        }
    }

    async fn handle_supervision(&mut self, from: GodName, healthy: bool) {
        if let Some(status) = self.supervised_actors.get_mut(&from) {
            *status = healthy;
            
            if !healthy {
                let count = self.restart_count.entry(from).or_insert(0);
                *count += 1;
                tracing::warn!("⚡ Zeus: {:?} reportado como no saludable (reinicios: {})", from, *count);
                
                if *count > 5 {
                    tracing::error!("🔥 Zeus: {:?} ha fallado demasiadas veces, escalando...", from);
                }
            } else {
                tracing::debug!("✅ Zeus: {:?} saludable", from);
            }
        }
    }

    async fn get_supervision_status(&self) -> serde_json::Value {
        let healthy: Vec<_> = self.supervised_actors
            .iter()
            .filter(|(_, h)| **h)
            .map(|(n, _)| n.as_str())
            .collect();
        
        let unhealthy: Vec<_> = self.supervised_actors
            .iter()
            .filter(|(_, h)| !**h)
            .map(|(n, _)| n.as_str())
            .collect();

        serde_json::json!({
            "total": self.supervised_actors.len(),
            "healthy": healthy.len(),
            "unhealthy": unhealthy.len(),
            "healthy_list": healthy,
            "unhealthy_list": unhealthy,
        })
    }
}

#[async_trait]
impl OlympianActor for Zeus {
    fn name(&self) -> GodName {
        GodName::Zeus
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        match &msg.payload {
            MessagePayload::Heartbeat { .. } => {
                // Recibir heartbeat de otro dios
                self.handle_supervision(msg.from, true).await;
                None
            }
            
            MessagePayload::Event { event_type, data } => {
                if event_type == "health_check" {
                    if let Ok(healthy) = serde_json::from_value::<bool>(data.clone()) {
                        self.handle_supervision(msg.from, healthy).await;
                    }
                }
                None
            }

            MessagePayload::Query { query_type, .. } => {
                if query_type == "supervision_status" {
                    let status = self.get_supervision_status().await;
                    return Some(ActorMessage::new(
                        GodName::Zeus,
                        msg.from,
                        MessagePayload::Response {
                            success: true,
                            data: status,
                            error: None,
                        }
                    ));
                }
                None
            }

            _ => {
                tracing::debug!("⚡ Zeus recibió mensaje de tipo desconocido");
                None
            }
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Zeus,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: 0,
            uptime_seconds: 0,
            status: "Supervising".to_string(),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("⚡ Zeus: Inicializando supervisión del Olimpo...");
        tracing::info!("⚡ Zeus: Supervisando {} dioses", self.supervised_actors.len());
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("⚡ Zeus: Deteniendo supervisión...");
        Ok(())
    }
}
