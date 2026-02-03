// src/actors/hermes/mod.rs
// OLYMPUS v13 - Hermes: Mensajero Divino
// Routing de mensajes entre dioses

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::info;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload};
use crate::errors::ActorError;

pub mod router;
pub mod mailbox;
pub mod delivery;
pub mod broadcast;

pub use router::MessageRouter;
pub use mailbox::Mailbox;
pub use delivery::DeliveryTracker;
pub use broadcast::Broadcaster;

#[derive(Debug, Clone)]
pub struct Hermes {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    router: Arc<RwLock<MessageRouter>>,
    mailboxes: Arc<RwLock<HashMap<GodName, Mailbox>>>,
    delivery_tracker: Arc<DeliveryTracker>,
    broadcaster: Arc<Broadcaster>,
    
    // Channel for commands
    command_rx: mpsc::Receiver<HermesCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HermesCommand {
    RegisterRoute { pattern: String, handler: GodName },
    SendMessage { to: GodName, message: ActorMessage },
    Broadcast { message: ActorMessage, exclude: Vec<GodName> },
    GetDeliveryStatus { message_id: String },
}

impl Hermes {
    pub async fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        Self {
            name: GodName::Hermes,
            state: ActorState::new(GodName::Hermes),
            config: ActorConfig::default(),
            
            router: Arc::new(RwLock::new(MessageRouter::new())),
            mailboxes: Arc::new(RwLock::new(HashMap::new())),
            delivery_tracker: Arc::new(DeliveryTracker::new()),
            broadcaster: Arc::new(Broadcaster::new()),
            
            command_rx,
        }
    }
}

#[async_trait]
impl OlympianActor for Hermes {
    fn name(&self) -> GodName {
        GodName::Hermes
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Messaging
    }
    
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();
        
        // Route the message
        let result = self.route_message(msg).await;
        result
    }
    
    fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Hermes",
            "routes": self.router.read().await.route_count(),
            "delivered": self.delivery_tracker.delivered_count(),
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Hermes,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Hermes,
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
        info!("👟 Hermes: Initializing message routing system...");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Hermes {
    async fn route_message(&self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        let to = msg.to.clone();
        
        // Track delivery
        let tracking = self.delivery_tracker.start_tracking(&msg.id, to.clone());
        
        // Deliver to mailbox
        let mailboxes = self.mailboxes.read().await;
        if let Some(mailbox) = mailboxes.get(&to) {
            mailbox.deliver(msg).await;
            tracking.record_delivery();
            Ok(ResponsePayload::Ack { message_id: msg.id })
        } else {
            // Mailbox not found, return error
            tracking.record_failure("Mailbox not found".to_string());
            Err(ActorError::NotFound { god: to })
        }
    }
}

use std::collections::HashMap;
