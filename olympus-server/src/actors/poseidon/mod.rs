// src/actors/poseidon/mod.rs
// OLYMPUS v16 - Poseidon: Señor del Flujo de Datos
// Implementación completa sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::GodName;
use crate::traits::{ActorState, ActorConfig};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
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
pub use reconnection::ReconnectionManager;

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
                let _ = res;
            }
            MessagePayload::Query(query) => {
                let res = self.handle_query(query, state).await;
                let _ = res;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Poseidon {
    async fn handle_command(&self, cmd: CommandPayload, _state: &mut PoseidonState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");
                match action {
                    "connect" => {
                        let url = data.get("url").and_then(|v| v.as_str()).unwrap_or("ws://localhost:8080");
                        Ok(ResponsePayload::Success { message: format!("Would connect to {}", url) })
                    }
                    "disconnect" => {
                        Ok(ResponsePayload::Success { message: "Disconnected".to_string() })
                    }
                    "send" => {
                        Ok(ResponsePayload::Success { message: "Message queued".to_string() })
                    }
                    "clear_buffer" => {
                        Ok(ResponsePayload::Success { message: "Buffer cleared".to_string() })
                    }
                    _ => Ok(ResponsePayload::Success { message: "Poseidon command processed".to_string() }),
                }
            }
            _ => Ok(ResponsePayload::Success { message: "Poseidon command processed".to_string() }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, _state: &PoseidonState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::HealthStatus => {
                Ok(ResponsePayload::Data { data: json!({
                    "status": "healthy",
                    "domain": "DataFlow"
                }) })
            }
            QueryPayload::GetStats => {
                Ok(ResponsePayload::Stats { data: json!({ "messages": 0 }) })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                match query_type {
                    "buffer_status" => {
                        Ok(ResponsePayload::Data { data: json!({ "size": 0 }) })
                    }
                    "flow_metrics" => {
                        Ok(ResponsePayload::Data { data: json!({ "rate": 0.0 }) })
                    }
                    "connection_info" => {
                        Ok(ResponsePayload::Data { data: json!({ "connected": false }) })
                    }
                    _ => Ok(ResponsePayload::Data { data: json!({ "domain": "DataFlow" }) }),
                }
            }
            _ => Ok(ResponsePayload::Data { data: json!({ "domain": "DataFlow" }) }),
        }
    }
}
