// src/actors/poseidon/mod.rs
// OLYMPUS v16 - Poseidon: Señor del Flujo de Datos
// Implementación completa sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tracing::{debug, error, info, warn};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, EventPayload};
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;

pub mod websocket;
pub mod buffer;
pub mod async_writer;
pub mod flow_control;
pub mod reconnection;

pub use websocket::{WebSocketManager, ConnectionInfo, WebSocketStats, WebSocketError, MessageCallback};
pub use buffer::EmergencyBuffer;
pub use async_writer::AsyncWriter;
pub use flow_control::{FlowController, FlowMetrics, FlowConfig};
pub use reconnection::{ReconnectionManager, ReconnectionState};

/// Poseidon State for Ractor
pub struct PoseidonState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    
    pub ws_manager: Arc<RwLock<WebSocketManager>>,
    pub buffer: Arc<EmergencyBuffer>,
    pub async_writer: Arc<AsyncWriter>,
    pub flow_controller: Arc<FlowController>,
    pub reconnection_manager: Arc<ReconnectionManager>,
    
    pub valkey: Arc<ValkeyStore>,
    pub message_callback: Arc<RwLock<Option<Arc<dyn Fn(String, String) + Send + Sync>>>>,
    pub running: Arc<std::sync::atomic::AtomicBool>,
}

pub struct Poseidon;

#[async_trait]
impl Actor for Poseidon {
    type Msg = ActorMessage;
    type State = PoseidonState;
    type Arguments = (ActorConfig, Arc<ValkeyStore>);

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let (config, valkey) = args;
        
        let ws_manager = Arc::new(RwLock::new(WebSocketManager::new(None)));
        let flow_controller = Arc::new(FlowController::new(None));
        let reconnection_manager = Arc::new(ReconnectionManager::new(None));
        
        let state = PoseidonState {
            name: GodName::Poseidon,
            metadata: ActorState::new(GodName::Poseidon),
            config,
            ws_manager,
            buffer: Arc::new(EmergencyBuffer::new(valkey.clone())),
            async_writer: Arc::new(AsyncWriter::new()),
            flow_controller,
            reconnection_manager,
            valkey,
            message_callback: Arc::new(RwLock::new(None)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        };
        Ok(state)
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        state.flow_controller.start_metrics_loop();
        state.reconnection_manager.start_persistence_loop().await;
        
        info!("🌊 Poseidon: Data Flow System ready");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message.payload {
            MessagePayload::Command(cmd) => {
                let res = self.handle_command(cmd, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            MessagePayload::Query(query) => {
                let res = self.handle_query(query, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Poseidon {
    async fn handle_command(&self, cmd: CommandPayload, state: &mut PoseidonState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");
                match action {
                    "connect" => {
                        let url = data.get("url").and_then(|v| v.as_str()).unwrap_or("ws://localhost:8080");
                        let mut ws_manager = state.ws_manager.write().await;
                        ws_manager.connect(url).await
                            .map_err(|e| ActorError::InvalidCommand { god: GodName::Poseidon, reason: e.to_string() })?;
                        Ok(ResponsePayload::Success { message: format!("Connected to {}", url) })
                    }
                    "disconnect" => {
                        let mut ws_manager = state.ws_manager.write().await;
                        ws_manager.disconnect().await
                            .map_err(|e| ActorError::InvalidCommand { god: GodName::Poseidon, reason: e.to_string() })?;
                        Ok(ResponsePayload::Success { message: "Disconnected".to_string() })
                    }
                    "send" => {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
                        let mut ws_manager = state.ws_manager.write().await;
                        ws_manager.send_message(msg).await
                            .map_err(|e| ActorError::InvalidCommand { god: GodName::Poseidon, reason: e.to_string() })?;
                        Ok(ResponsePayload::Success { message: "Message sent".to_string() })
                    }
                    "get_buffer_stats" => {
                        let stats = state.buffer.get_stats().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(stats).unwrap_or_default() })
                    }
                    "clear_buffer" => {
                        state.buffer.clear().await;
                        Ok(ResponsePayload::Success { message: "Buffer cleared".to_string() })
                    }
                    "clear_flow_buffer" => {
                        state.flow_controller.clear_buffer().await;
                        Ok(ResponsePayload::Success { message: "Flow buffer cleared".to_string() })
                    }
                    _ => Err(ActorError::InvalidCommand { god: GodName::Poseidon, reason: format!("Unknown action: {}", action) }),
                }
            }
            _ => Ok(ResponsePayload::Success { message: "Poseidon command processed".to_string() }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, state: &PoseidonState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::HealthStatus => {
                let ws_manager = state.ws_manager.read().await;
                let is_connected = ws_manager.is_connected();
                let buffer_size = state.buffer.len().await;
                Ok(ResponsePayload::Data { data: json!({
                    "status": if is_connected { "healthy" } else { "degraded" },
                    "domain": "DataFlow",
                    "connected": is_connected,
                    "buffer_size": buffer_size
                }) })
            }
            QueryPayload::GetStats => {
                let ws_manager = state.ws_manager.read().await;
                let stats = ws_manager.get_stats();
                Ok(ResponsePayload::Stats { data: serde_json::to_value(stats).unwrap_or_default() })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                match query_type {
                    "buffer_status" => {
                        let stats = state.buffer.get_stats().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(stats).unwrap_or_default() })
                    }
                    "flow_metrics" => {
                        let metrics = state.flow_controller.get_metrics().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(metrics).unwrap_or_default() })
                    }
                    "connection_info" => {
                        let ws_manager = state.ws_manager.read().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(ws_manager.get_connection_info()).unwrap_or_default() })
                    }
                    _ => Ok(ResponsePayload::Data { data: json!({ "domain": "DataFlow" }) }),
                }
            }
            _ => Ok(ResponsePayload::Data { data: json!({ "domain": "DataFlow" }) }),
        }
    }
}
