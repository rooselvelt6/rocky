use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use ractor::{Actor, ActorRef, ActorProcessingErr, RpcReplyPort};

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
#[derive(Debug, Serialize, Deserialize)]
pub enum MessagePayload {
    // Comandos (v16: Ahora con soporte Ractor RPC)
    Command { 
        action: String, 
        data: serde_json::Value,
        #[serde(skip)]
        reply: Option<RpcReplyPort<MessagePayload>>
    },
    // Consultas (v16: Ahora con soporte Ractor RPC)
    Query { 
        query_type: String, 
        params: serde_json::Value,
        #[serde(skip)]
        reply: Option<RpcReplyPort<MessagePayload>>
    },
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
#[derive(Debug, Serialize, Deserialize)]
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
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodHealth {
    pub name: GodName,
    pub healthy: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub messages_processed: u64,
    pub uptime_seconds: u64,
    pub status: String,
}

// Trait para todos los dioses (v16: Ahora basado en Ractor)
pub trait OlympianActor: Actor<Msg = ActorMessage> {
    fn name(&self) -> GodName;
    async fn initialize(&mut self) -> Result<(), String>;
    async fn shutdown(&mut self) -> Result<(), String>;
}

// Nota: En v16, cada actor implementará Actor de Ractor directamente
// para aprovechar la supervisión automática de Zeus.

// Estado del Olimpo - singleton compartido
#[allow(dead_code)]
pub type OlympusState = Arc<RwLock<OlympusInner>>;

#[allow(dead_code)]
pub struct OlympusInner {
    pub actors: HashMap<GodName, ActorRef<ActorMessage>>,
    pub start_time: DateTime<Utc>,
}

#[allow(dead_code)]
impl OlympusInner {
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            start_time: Utc::now(),
        }
    }

    pub async fn send_to(&self, god: GodName, msg: ActorMessage) -> Result<(), String> {
        if let Some(actor) = self.actors.get(&god) {
            actor.send_message(msg).map_err(|e| format!("Fallo al enviar vía Ractor: {}", e))
        } else {
            Err(format!("Dios {:?} no encontrado en el registro", god))
        }
    }
}
