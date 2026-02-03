// src/actors/poseidon/mod.rs
// OLYMPUS v13 - Poseidon: Señor del Flujo de Datos
// WebSocket connections, buffer y control de flujo

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload};
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;

pub mod websocket;
pub mod buffer;
pub mod async_writer;
pub mod flow_control;
pub mod reconnection;

pub use websocket::WebSocketManager;
pub use buffer::EmergencyBuffer;
pub use async_writer::AsyncWriter;
pub use flow_control::FlowController;
pub use reconnection::ReconnectionPolicy;

#[derive(Debug, Clone)]
pub struct Poseidon {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    ws_manager: Arc<RwLock<WebSocketManager>>,
    buffer: Arc<EmergencyBuffer>,
    async_writer: Arc<AsyncWriter>,
    flow_controller: Arc<FlowController>,
    
    // Valkey for buffer
    valkey: Arc<ValkeyStore>,
    
    // Channel for commands
    command_rx: mpsc::Receiver<PoseidonCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonCommand {
    Connect { url: String, domain: DivineDomain },
    Disconnect { connection_id: String },
    SendData { domain: DivineDomain, data: serde_json::Value },
    FlushBuffer,
    GetConnectionStatus,
}

impl Poseidon {
    pub async fn new(valkey: Arc<ValkeyStore>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        Self {
            name: GodName::Poseidon,
            state: ActorState::new(GodName::Poseidon),
            config: ActorConfig::default(),
            
            ws_manager: Arc::new(RwLock::new(WebSocketManager::new())),
            buffer: Arc::new(EmergencyBuffer::new(valkey.clone())),
            async_writer: Arc::new(AsyncWriter::new()),
            flow_controller: Arc::new(FlowController::new()),
            
            valkey,
            command_rx,
        }
    }
}

#[async_trait]
impl OlympianActor for Poseidon {
    fn name(&self) -> GodName {
        GodName::Poseidon
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::DataFlow
    }
    
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();
        
        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Event(event) => self.handle_event(event).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }
    
    fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Poseidon",
            "active_connections": self.ws_manager.read().await.connection_count(),
            "buffered_messages": self.buffer.len().await,
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Poseidon,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Poseidon,
            status: ActorStatus::Healthy,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }
    
    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🌊 Poseidon: Initializing data flow system...");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        self.ws_manager.write().await.close_all().await;
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Poseidon {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Configure { config } => {
                // Configure Poseidon
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unknown command".to_string(), code: 400 }),
        }
    }
    
    async fn handle_query(&self, _query: crate::traits::message::QueryPayload) -> Result<ResponsePayload, ActorError> {
        let status = self.ws_manager.read().await.get_status().await;
        Ok(ResponsePayload::Data { data: serde_json::to_value(status).unwrap() })
    }
    
    async fn handle_event(&mut self, _event: crate::traits::message::EventPayload) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
    }
}
