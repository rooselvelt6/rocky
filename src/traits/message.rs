// src/traits/message.rs
// OLYMPUS v13 - Sistema de Mensajes Tipados

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::actors::GodName;

/// Mensaje tipado para comunicación entre actores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    pub id: String,
    pub from: Option<GodName>,
    pub to: GodName,
    pub payload: MessagePayload,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub priority: MessagePriority,
    pub metadata: serde_json::Value,
}

impl ActorMessage {
    pub fn new(to: GodName, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: None,
            to,
            payload,
            timestamp: chrono::Utc::now(),
            priority: MessagePriority::Normal,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_from(from: GodName, to: GodName, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: Some(from),
            to,
            payload,
            timestamp: chrono::Utc::now(),
            priority: MessagePriority::Normal,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_priority(priority: MessagePriority, to: GodName, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: None,
            to,
            payload,
            timestamp: chrono::Utc::now(),
            priority,
            metadata: serde_json::json!({}),
        }
    }
}

/// Prioridad del mensaje
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
    Immediate,
}

/// Payload del mensaje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Comandos que cambian estado
    Command(CommandPayload),
    /// Consultas que no cambian estado
    Query(QueryPayload),
    /// Eventos que notifican cambios
    Event(EventPayload),
    /// Respuestas a mensajes anteriores
    Response(ResponsePayload),
}

/// Comandos para los dioses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandPayload {
    // Zeus commands
    StartActor {
        actor: GodName,
    },
    StopActor {
        actor: GodName,
        reason: String,
    },
    RestartActor {
        actor: GodName,
    },
    EmergencyShutdown {
        reason: String,
    },

    // Erinyes commands
    RecoverActor {
        actor: GodName,
        strategy: RecoveryStrategy,
    },
    ConfigureHeartbeat {
        actor: GodName,
        interval_ms: u64,
    },

    // Poseidon commands
    Connect {
        url: String,
    },
    Disconnect {
        connection_id: String,
    },
    FlushBuffer,

    // Hermes commands
    SendMessage {
        to: GodName,
        message: Box<ActorMessage>,
    },
    Broadcast {
        message: Box<ActorMessage>,
    },

    // Generic commands
    Shutdown,
    Configure {
        config: serde_json::Value,
    },
}

/// Consultas para los dioses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPayload {
    // Status queries
    HealthStatus,
    ActorState,
    Metrics,

    // Data queries
    GetData { key: String },
    GetHistory { limit: u64 },
    Search { query: String },

    // Configuration queries
    GetConfig,
    ListActors,
}

/// Eventos emitidos por los dioses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    // Lifecycle events
    ActorStarted {
        actor: GodName,
    },
    ActorStopped {
        actor: GodName,
        reason: String,
    },
    ActorRecovered {
        actor: GodName,
        attempt: u32,
    },

    // Data events
    DataReceived {
        source: GodName,
        data_type: String,
    },
    DataPersisted {
        key: String,
    },

    // Error events
    ErrorOccurred {
        error: String,
        actor: GodName,
    },
    HeartbeatMissed {
        actor: GodName,
    },

    // Domain events
    ClinicalAlert {
        patient_id: String,
        severity: String,
    },
    SecurityAlert {
        threat_type: String,
        source: String,
    },
    ConfigChanged {
        key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
}

/// Respuestas de los dioses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponsePayload {
    Success { data: serde_json::Value },
    Error { error: String, code: i32 },
    Data { data: serde_json::Value },
    Ack { message_id: String },
    Status { status: serde_json::Value },
}

/// Estrategia de recuperación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    OneForOne,  // Solo el caído se reinicia
    OneForAll,  // Todos se reinician
    RestForOne, // El caído y los posteriores
    Escalate,   // Notificar a Zeus
}

/// Resultado de enviar un mensaje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub message_id: String,
    pub status: SendStatus,
    pub delivery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SendStatus {
    Delivered,
    Queued,
    DeadLettered,
    Failed,
}

/// Confirmación de entrega
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfirmation {
    pub message_id: String,
    pub delivered_to: GodName,
    pub delivered_at: chrono::DateTime<chrono::Utc>,
    pub attempts: u32,
}
