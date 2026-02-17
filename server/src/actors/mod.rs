// server/src/actors/mod.rs
// Sistema de Actores Olympus - 20 Dioses

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use chrono::{DateTime, Utc};

pub mod zeus;
pub mod hades;
pub mod poseidon;
pub mod athena;
pub mod hermes;
pub mod hestia;
pub mod erinyes;
pub mod aphrodite;
pub mod minor_gods;

pub use zeus::Zeus;
pub use hades::Hades;
pub use poseidon::Poseidon;
pub use athena::Athena;
pub use hermes::Hermes;
pub use hestia::Hestia;
pub use erinyes::Erinyes;
pub use aphrodite::Aphrodite;
pub use minor_gods::{Apollo, Artemis, Hera, Ares, Hefesto, Chronos, Moirai, Chaos, Aurora, Iris, Demeter, Dionysus};

// Nombres de los 20 dioses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GodName {
    Zeus,
    Hades,
    Poseidon,
    Athena,
    Hermes,
    Hestia,
    Erinyes,
    Apollo,
    Artemis,
    Hera,
    Ares,
    Hefesto,
    Chronos,
    Moirai,
    Chaos,
    Aurora,
    Aphrodite,
    Iris,
    Demeter,
    Dionysus,
}

impl GodName {
    pub fn as_str(&self) -> &'static str {
        match self {
            GodName::Zeus => "Zeus",
            GodName::Hades => "Hades",
            GodName::Poseidon => "Poseidon",
            GodName::Athena => "Athena",
            GodName::Hermes => "Hermes",
            GodName::Hestia => "Hestia",
            GodName::Erinyes => "Erinyes",
            GodName::Apollo => "Apollo",
            GodName::Artemis => "Artemis",
            GodName::Hera => "Hera",
            GodName::Ares => "Ares",
            GodName::Hefesto => "Hefesto",
            GodName::Chronos => "Chronos",
            GodName::Moirai => "Moirai",
            GodName::Chaos => "Chaos",
            GodName::Aurora => "Aurora",
            GodName::Aphrodite => "Aphrodite",
            GodName::Iris => "Iris",
            GodName::Demeter => "Demeter",
            GodName::Dionysus => "Dionysus",
        }
    }

    pub fn domain(&self) -> &'static str {
        match self {
            GodName::Zeus => "Governance",
            GodName::Hades => "Security",
            GodName::Poseidon => "DataFlow",
            GodName::Athena => "Clinical",
            GodName::Hermes => "Messaging",
            GodName::Hestia => "Persistence",
            GodName::Erinyes => "Integrity",
            GodName::Apollo => "Events",
            GodName::Artemis => "Search",
            GodName::Hera => "Validation",
            GodName::Ares => "ConflictResolution",
            GodName::Hefesto => "Configuration",
            GodName::Chronos => "Scheduling",
            GodName::Moirai => "Predictions",
            GodName::Chaos => "Testing",
            GodName::Aurora => "NewBeginnings",
            GodName::Aphrodite => "UI/UX",
            GodName::Iris => "Communications",
            GodName::Demeter => "Resources",
            GodName::Dionysus => "Analysis",
        }
    }
}

// Tipos de mensajes entre dioses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    // Comandos
    Command { action: String, data: serde_json::Value },
    // Consultas
    Query { query_type: String, params: serde_json::Value },
    // Eventos
    Event { event_type: String, data: serde_json::Value },
    // Respuestas
    Response { success: bool, data: serde_json::Value, error: Option<String> },
    // Heartbeat
    Heartbeat { timestamp: DateTime<Utc> },
    // Shutdown
    Shutdown { reason: String },
}

// Mensaje entre actores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    pub id: String,
    pub from: GodName,
    pub to: GodName,
    pub payload: MessagePayload,
    pub timestamp: DateTime<Utc>,
}

impl ActorMessage {
    pub fn new(from: GodName, to: GodName, payload: MessagePayload) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            payload,
            timestamp: Utc::now(),
        }
    }
}

// Estado de salud de un dios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodHealth {
    pub name: GodName,
    pub healthy: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub messages_processed: u64,
    pub uptime_seconds: u64,
    pub status: String,
}

// Trait para todos los dioses
#[async_trait]
pub trait OlympianActor: Send + Sync {
    fn name(&self) -> GodName;
    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage>;
    async fn health(&self) -> GodHealth;
    async fn initialize(&mut self) -> Result<(), String>;
    async fn shutdown(&mut self) -> Result<(), String>;
}

// Runtime de actor
pub struct ActorRuntime {
    actor: Box<dyn OlympianActor>,
    inbox: mpsc::Receiver<ActorMessage>,
    messages_processed: u64,
    start_time: DateTime<Utc>,
}

impl ActorRuntime {
    pub fn new(actor: Box<dyn OlympianActor>, inbox: mpsc::Receiver<ActorMessage>) -> Self {
        Self {
            actor,
            inbox,
            messages_processed: 0,
            start_time: Utc::now(),
        }
    }

    pub async fn run(mut self) {
        let name = self.actor.name();
        tracing::info!("🌟 [{}] Actor iniciado", name.as_str());

        // Inicializar
        if let Err(e) = self.actor.initialize().await {
            tracing::error!("🚨 [{}] Fallo al inicializar: {}", name.as_str(), e);
            return;
        }

        tracing::info!("✨ [{}] Actor listo", name.as_str());

        // Loop principal
        loop {
            match self.inbox.recv().await {
                Some(msg) => {
                    let should_shutdown = matches!(msg.payload, MessagePayload::Shutdown { .. });
                    
                    if let Some(response) = self.actor.handle_message(msg).await {
                        // Si hay respuesta, manejarla (por ahora solo log)
                        tracing::debug!("📨 [{}] Respuesta generada", name.as_str());
                    }
                    
                    self.messages_processed += 1;

                    if should_shutdown {
                        break;
                    }
                }
                None => {
                    tracing::warn!("📭 [{}] Canal cerrado", name.as_str());
                    break;
                }
            }
        }

        // Shutdown
        let _ = self.actor.shutdown().await;
        tracing::info!("🛑 [{}] Actor detenido", name.as_str());
    }

    pub fn get_stats(&self) -> (u64, DateTime<Utc>) {
        (self.messages_processed, self.start_time)
    }
}

// Estado del Olimpo - singleton compartido
pub type OlympusState = Arc<RwLock<OlympusInner>>;

pub struct OlympusInner {
    pub senders: HashMap<GodName, mpsc::Sender<ActorMessage>>,
    pub health: HashMap<GodName, GodHealth>,
    pub start_time: DateTime<Utc>,
}

impl OlympusInner {
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
            health: HashMap::new(),
            start_time: Utc::now(),
        }
    }

    pub async fn send_to(&self, god: GodName, msg: ActorMessage) -> Result<(), String> {
        if let Some(sender) = self.senders.get(&god) {
            sender.send(msg).await.map_err(|e| format!("Failed to send: {}", e))
        } else {
            Err(format!("God {:?} not found", god))
        }
    }

    pub async fn update_health(&mut self, health: GodHealth) {
        self.health.insert(health.name, health);
    }

    pub fn get_all_health(&self) -> Vec<GodHealth> {
        self.health.values().cloned().collect()
    }
}
