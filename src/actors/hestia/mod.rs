// src/actors/hestia/mod.rs
// OLYMPUS v13 - Hestia: Persistencia y Memoria
// Almacenamiento dual (Valkey + SurrealDB)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::info;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload};
use crate::traits::persistable::{Persistable, PersistenceError};
use crate::infrastructure::{ValkeyStore, SurrealStore};
use crate::errors::ActorError;

pub mod memory_store;
pub mod async_buffer;
pub mod cache;
pub mod sync;

pub use memory_store::MemoryStore;
pub use async_buffer::AsyncBuffer;
pub use cache::CacheManager;
pub use sync::SyncManager;

#[derive(Debug, Clone)]
pub struct Hestia {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    memory_store: Arc<MemoryStore>,
    async_buffer: Arc<AsyncBuffer>,
    cache: Arc<CacheManager>,
    sync_manager: Arc<SyncManager>,
    
    // Infrastructure
    valkey: Arc<ValkeyStore>,
    surreal: Arc<SurrealStore>,
    
    // Channel for commands
    command_rx: mpsc::Receiver<HestiaCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HestiaCommand {
    Save { key: String, value: serde_json::Value },
    Load { key: String },
    Delete { key: String },
    Flush,
    Sync,
}

impl Hestia {
    pub async fn new(valkey: Arc<ValkeyStore>, surreal: Arc<SurrealStore>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        Self {
            name: GodName::Hestia,
            state: ActorState::new(GodName::Hestia),
            config: ActorConfig::default(),
            
            memory_store: Arc::new(MemoryStore::new(valkey.clone())),
            async_buffer: Arc::new(AsyncBuffer::new(valkey.clone(), surreal.clone())),
            cache: Arc::new(CacheManager::new(valkey.clone())),
            sync_manager: Arc::new(SyncManager::new(valkey.clone(), surreal.clone())),
            
            valkey,
            surreal,
            command_rx,
        }
    }
}

#[async_trait]
impl OlympianActor for Hestia {
    fn name(&self) -> GodName {
        GodName::Hestia
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Persistence
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
            "name": "Hestia",
            "cached_items": self.cache.size().await,
            "buffered_items": self.async_buffer.len().await,
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Hestia,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Hestia,
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
        info!("🏠 Hestia: Initializing persistence system...");
        self.sync_manager.start().await;
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        self.async_buffer.flush().await;
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Hestia {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Shutdown => {
                self.shutdown().await?;
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unknown command".to_string(), code: 400 }),
        }
    }
    
    async fn handle_query(&self, _query: crate::traits::message::QueryPayload) -> Result<ResponsePayload, ActorError> {
        let stats = self.cache.size().await;
        Ok(ResponsePayload::Data { data: serde_json::json!({ "cached": stats }) })
    }
    
    async fn handle_event(&mut self, _event: crate::traits::message::EventPayload) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
    }
}
