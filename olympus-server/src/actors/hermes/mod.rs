// src/actors/hermes/mod.rs
// OLYMPUS v16 - Hermes: Mensajero Divino
// Implementación sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{info, warn, debug};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

pub mod router;
pub mod mailbox;
pub mod delivery;
pub mod broadcast;
pub mod retry;

pub use router::MessageRouter;
pub use mailbox::MailboxManager;
pub use delivery::DeliveryTracker;
pub use broadcast::{Broadcaster, BroadcastEvent};
pub use retry::{RetryQueue, RetryConfig, RetryWorker};

/// Hermes State for Ractor
pub struct HermesState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    
    pub router: Arc<RwLock<MessageRouter>>,
    pub mailbox_manager: Arc<MailboxManager>,
    pub delivery_tracker: Arc<DeliveryTracker>,
    pub broadcaster: Arc<Broadcaster>,
    pub retry_queue: Arc<RetryQueue>,
    pub retry_worker: Option<RetryWorker>,
    
    pub retry_config: RetryConfig,
    pub default_mailbox_size: usize,
}

pub struct Hermes;

#[async_trait]
impl Actor for Hermes {
    type Msg = ActorMessage;
    type State = HermesState;
    type Arguments = ActorConfig;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, config: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let default_mailbox_size = 1000;
        let retry_config = RetryConfig::default();
        let delivery_tracker = Arc::new(DeliveryTracker::new());
        let retry_queue = Arc::new(RetryQueue::new(retry_config.clone(), delivery_tracker.clone()));

        Ok(HermesState {
            name: GodName::Hermes,
            metadata: ActorState::new(GodName::Hermes),
            config,
            router: Arc::new(RwLock::new(MessageRouter::new())),
            mailbox_manager: Arc::new(MailboxManager::new(default_mailbox_size)),
            delivery_tracker,
            broadcaster: Arc::new(Broadcaster::new()),
            retry_queue,
            retry_worker: None,
            retry_config,
            default_mailbox_size,
        })
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        info!("👟 Hermes: Divine Messenger System ready");
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

impl Hermes {
    async fn handle_command(&self, _cmd: CommandPayload, _state: &mut HermesState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Success { message: "Hermes routing command processed".to_string() })
    }

    async fn handle_query(&self, _query: QueryPayload, _state: &HermesState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Data { data: json!({ "mailboxes": 0 }) })
    }
}
