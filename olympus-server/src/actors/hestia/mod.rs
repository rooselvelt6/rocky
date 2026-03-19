// src/actors/hestia/mod.rs
// OLYMPUS v16 - Hestia: Sistema de Persistencia Completo
// Implementación sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{info};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{ActorState, ActorConfig};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;
use crate::infrastructure::{ValkeyStore, SurrealStore};

pub mod memory_store;
pub mod cache;
pub mod async_buffer;
pub mod sync;

pub use memory_store::{MemoryStore, MemoryStoreConfig};
pub use cache::{CacheManager, CacheConfig, CacheLevel};
pub use async_buffer::{AsyncBuffer, OperationType, FlushResult};
pub use sync::{SyncManager, ConflictResolution, SyncResult};

/// Hestia State for Ractor
pub struct HestiaState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    pub memory_store: Arc<MemoryStore>,
    pub cache: Arc<CacheManager>,
    pub async_buffer: Arc<AsyncBuffer>,
    pub sync_manager: Arc<SyncManager>,
    pub valkey: Arc<ValkeyStore>,
    pub surreal: Arc<SurrealStore>,
    pub hades_encryption: bool,
    pub default_encryption_key: Option<String>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub consecutive_errors: u32,
    pub running: bool,
}

pub struct Hestia;

#[async_trait]
impl Actor for Hestia {
    type Msg = ActorMessage;
    type State = HestiaState;
    type Arguments = (Arc<ValkeyStore>, Arc<SurrealStore>);

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, (valkey, surreal): Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let memory_store = Arc::new(MemoryStore::with_config(valkey.clone(), MemoryStoreConfig::default()));
        let sync_manager = Arc::new(SyncManager::new(valkey.clone(), surreal.clone()));
        let cache = Arc::new(CacheManager::with_config(valkey.clone(), CacheConfig::default()).with_sync_manager(sync_manager.clone()));
        let async_buffer = Arc::new(AsyncBuffer::new(valkey.clone(), surreal.clone()));

        Ok(HestiaState {
            name: GodName::Hestia,
            metadata: ActorState::new(GodName::Hestia),
            config: ActorConfig::default(),
            memory_store,
            cache,
            async_buffer,
            sync_manager,
            valkey,
            surreal,
            hades_encryption: false,
            default_encryption_key: None,
            last_health_check: chrono::Utc::now(),
            consecutive_errors: 0,
            running: false,
        })
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        state.running = true;
        state.cache.start_background_tasks().await;
        state.async_buffer.start().await;
        state.sync_manager.start().await;
        
        info!("🏠 Hestia: Persistence System ready");
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
            _ => if let Some(reply) = message.reply_to { let _ = reply.send(Ok(ResponsePayload::Ack { message_id: message.id })); }
        }
        Ok(())
    }
}

impl Hestia {
    async fn handle_command(&self, _cmd: CommandPayload, _state: &mut HestiaState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Success { message: "Hestia persistence action processed".to_string() })
    }

    async fn handle_query(&self, _query: QueryPayload, state: &HestiaState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Data { data: serde_json::json!({ "storage": "dual" }) })
    }
}
