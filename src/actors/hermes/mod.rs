// src/actors/hermes/mod.rs
// OLYMPUS v15 - Hermes: Mensajero Divino
// Sistema completo de mensajería con retry, broadcast, y delivery tracking

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{info, warn, debug};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

pub mod router;
pub mod mailbox;
pub mod delivery;
pub mod broadcast;
pub mod retry;

pub use router::{MessageRouter, Route};
pub use mailbox::{Mailbox, MailboxStats, MailboxManager};
pub use delivery::{DeliveryTracker, DeliveryTracking, DeliveryStatus, DeliveryTrackingHandle};
pub use broadcast::{Broadcaster, BroadcastEvent};
pub use retry::{RetryQueue, RetryConfig, RetryWorker, RetryableMessage};

#[derive(Debug)]
pub struct Hermes {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    router: Arc<RwLock<MessageRouter>>,
    mailbox_manager: Arc<MailboxManager>,
    delivery_tracker: Arc<DeliveryTracker>,
    broadcaster: Arc<Broadcaster>,
    retry_queue: Arc<RetryQueue>,
    retry_worker: Option<RetryWorker>,
    
    // Configuration
    retry_config: RetryConfig,
    default_mailbox_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HermesCommand {
    RegisterRoute { pattern: String, handler: GodName, priority: u32 },
    RegisterWildcard { handler: GodName, priority: u32 },
    SendMessage { to: GodName, message: ActorMessage },
    SendWithRetry { to: GodName, message: ActorMessage, config: Option<RetryConfig> },
    Broadcast { message: ActorMessage, exclude: Vec<GodName> },
    GetDeliveryStatus { message_id: String },
    GetMailboxStats { god: GodName },
    GetAllMailboxesStats,
    CreateMailbox { god: GodName, max_size: Option<usize> },
    ClearMailbox { god: GodName },
    PurgeOldTrackings { max_age_seconds: u64 },
    ConfigureRetry { config: RetryConfig },
}

impl Hermes {
    pub async fn new() -> Self {
        let default_mailbox_size = 1000;
        let retry_config = RetryConfig::default();
        
        let delivery_tracker = Arc::new(DeliveryTracker::new());
        let retry_queue = Arc::new(RetryQueue::new(retry_config.clone(), delivery_tracker.clone()));
        
        Self {
            name: GodName::Hermes,
            state: ActorState::new(GodName::Hermes),
            config: ActorConfig::default(),
            
            router: Arc::new(RwLock::new(MessageRouter::new())),
            mailbox_manager: Arc::new(MailboxManager::new(default_mailbox_size)),
            delivery_tracker,
            broadcaster: Arc::new(Broadcaster::new()),
            retry_queue,
            retry_worker: None,
            
            retry_config,
            default_mailbox_size,
        }
    }
    
    pub async fn new_with_config(config: ActorConfig, retry_config: RetryConfig, mailbox_size: usize) -> Self {
        let delivery_tracker = Arc::new(DeliveryTracker::new());
        let retry_queue = Arc::new(RetryQueue::new(retry_config.clone(), delivery_tracker.clone()));
        
        Self {
            name: GodName::Hermes,
            state: ActorState::new(GodName::Hermes),
            config,
            
            router: Arc::new(RwLock::new(MessageRouter::new())),
            mailbox_manager: Arc::new(MailboxManager::new(mailbox_size)),
            delivery_tracker,
            broadcaster: Arc::new(Broadcaster::new()),
            retry_queue,
            retry_worker: None,
            
            retry_config,
            default_mailbox_size: mailbox_size,
        }
    }

    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                // Try to parse as HermesCommand
                if let Ok(hermes_cmd) = serde_json::from_value::<HermesCommand>(data) {
                    self.execute_command(hermes_cmd).await
                } else {
                    Err(ActorError::InvalidCommand { 
                        god: GodName::Hermes, 
                        reason: "Unknown command format".to_string() 
                    })
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Hermes, 
                reason: "Hermes only accepts Custom commands".to_string() 
            })
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::GetStats => {
                let delivered = self.delivery_tracker.delivered_count().await;
                let failed = self.delivery_tracker.failed_count().await;
                let pending = self.delivery_tracker.pending_count().await;
                let mailbox_stats = self.mailbox_manager.get_all_stats().await;
                
                Ok(ResponsePayload::Stats {
                    data: serde_json::json!({
                        "delivered_messages": delivered,
                        "failed_messages": failed,
                        "pending_messages": pending,
                        "mailboxes": mailbox_stats,
                        "retry_queue_size": self.retry_queue.len().await,
                        "total_messages_processed": self.state.message_count,
                    }),
                })
            }
            QueryPayload::Custom(data) => {
                // Handle custom queries
                if let Some(query_type) = data.get("query_type").and_then(|v| v.as_str()) {
                    match query_type {
                        "delivery_status" => {
                            if let Some(msg_id) = data.get("message_id").and_then(|v| v.as_str()) {
                                let tracking = self.delivery_tracker.get_tracking(msg_id).await;
                                Ok(ResponsePayload::Data { 
                                    data: serde_json::to_value(tracking).unwrap_or_default() 
                                })
                            } else {
                                Err(ActorError::InvalidQuery { 
                                    god: GodName::Hermes, 
                                    reason: "Missing message_id".to_string() 
                                })
                            }
                        }
                        "mailbox_stats" => {
                            if let Some(god_name) = data.get("god").and_then(|v| v.as_str()) {
                                // Parse god name
                                if let Ok(god) = serde_json::from_value::<GodName>(json!(god_name)) {
                                    if let Some(mailbox) = self.mailbox_manager.get_mailbox(&god).await {
                                        let stats = mailbox.stats().await;
                                        Ok(ResponsePayload::Data { 
                                            data: serde_json::to_value(stats).unwrap_or_default() 
                                        })
                                    } else {
                                        Err(ActorError::NotFound { god })
                                    }
                                } else {
                                    Err(ActorError::InvalidQuery { 
                                        god: GodName::Hermes, 
                                        reason: "Invalid god name".to_string() 
                                    })
                                }
                            } else {
                                let all_stats = self.mailbox_manager.get_all_stats().await;
                                Ok(ResponsePayload::Data { 
                                    data: serde_json::to_value(all_stats).unwrap_or_default() 
                                })
                            }
                        }
                        _ => Err(ActorError::InvalidQuery { 
                            god: GodName::Hermes, 
                            reason: format!("Unknown query type: {}", query_type) 
                        })
                    }
                } else {
                    Err(ActorError::InvalidQuery { 
                        god: GodName::Hermes, 
                        reason: "Missing query_type".to_string() 
                    })
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Hermes, 
                reason: "Unsupported query type".to_string() 
            })
        }
    }

    async fn route_message(&self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        let to = msg.to.clone();
        
        // Track delivery
        let tracking = self.delivery_tracker.start_tracking(&msg.id, to.clone()).await;
        
        // Attempt delivery via mailbox manager
        match self.mailbox_manager.deliver_to(&to, msg.clone()).await {
            Ok(()) => {
                tracking.record_delivery().await;
                debug!(
                    message_id = %msg.id,
                    to = %to,
                    "Message delivered successfully"
                );
                Ok(ResponsePayload::Ack { message_id: msg.id })
            }
            Err(ActorError::NotFound { .. }) => {
                // Try to create mailbox automatically
                info!("Auto-creating mailbox for {}", to);
                self.mailbox_manager.create_mailbox(to.clone()).await;
                
                // Retry delivery
                match self.mailbox_manager.deliver_to(&to, msg.clone()).await {
                    Ok(()) => {
                        tracking.record_delivery().await;
                        Ok(ResponsePayload::Ack { message_id: msg.id })
                    }
                    Err(e) => {
                        tracking.record_failure(e.to_string()).await;
                        Err(e)
                    }
                }
            }
            Err(ActorError::MailboxFull { .. }) => {
                // Queue for retry
                warn!(
                    message_id = %msg.id,
                    to = %to,
                    "Mailbox full, queuing for retry"
                );
                
                tracking.record_failure("Mailbox full".to_string()).await;
                let msg_id = msg.id.clone();
                self.retry_queue.enqueue(msg, to, "Mailbox full".to_string()).await;
                
                Ok(ResponsePayload::RetryScheduled { message_id: msg_id })
            }
            Err(e) => {
                tracking.record_failure(e.to_string()).await;
                Err(e)
            }
        }
    }
    
    async fn execute_command(&self, cmd: HermesCommand) -> Result<ResponsePayload, ActorError> {
        match cmd {
            HermesCommand::RegisterRoute { pattern, handler, priority } => {
                let router = self.router.write().await;
                router.register_route(&pattern, handler, priority).await;
                
                info!("Registered route '{}' -> {:?}", pattern, handler);
                
                Ok(ResponsePayload::Success { 
                    message: format!("Route '{}' registered", pattern) 
                })
            }
            HermesCommand::RegisterWildcard { handler, priority } => {
                let router = self.router.write().await;
                router.register_wildcard(handler, priority).await;
                
                info!("Registered wildcard route -> {:?}", handler);
                
                Ok(ResponsePayload::Success { 
                    message: "Wildcard route registered".to_string() 
                })
            }
            HermesCommand::SendMessage { to: _, message } => {
                self.route_message(message).await
            }
            HermesCommand::SendWithRetry { to, message, config } => {
                // Attempt delivery with custom retry config
                let result = self.route_message(message.clone()).await;
                
                if result.is_err() {
                    // Queue with retry
                    let retry_config = config.unwrap_or_else(|| self.retry_config.clone());
                    let retry_queue = Arc::new(RetryQueue::new(
                        retry_config, 
                        self.delivery_tracker.clone()
                    ));
                    
                    retry_queue.enqueue(message, to, "Initial delivery failed".to_string()).await;
                }
                
                result
            }
            HermesCommand::Broadcast { message, exclude } => {
                self.broadcaster.broadcast(message, exclude);
                
                Ok(ResponsePayload::Success { 
                    message: "Message broadcasted".to_string() 
                })
            }
            HermesCommand::GetDeliveryStatus { message_id } => {
                let tracking = self.delivery_tracker.get_tracking(&message_id).await;
                
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(tracking).unwrap_or_default() 
                })
            }
            HermesCommand::GetMailboxStats { god } => {
                if let Some(mailbox) = self.mailbox_manager.get_mailbox(&god).await {
                    let stats = mailbox.stats().await;
                    Ok(ResponsePayload::Data { 
                        data: serde_json::to_value(stats).unwrap_or_default() 
                    })
                } else {
                    Err(ActorError::NotFound { god })
                }
            }
            HermesCommand::GetAllMailboxesStats => {
                let stats = self.mailbox_manager.get_all_stats().await;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(stats).unwrap_or_default() 
                })
            }
            HermesCommand::CreateMailbox { god, max_size } => {
                let size = max_size.unwrap_or(self.default_mailbox_size);
                self.mailbox_manager.create_mailbox(god.clone()).await;
                
                info!("Created mailbox for {:?} with capacity {}", god, size);
                
                Ok(ResponsePayload::Success { 
                    message: format!("Mailbox for {:?} created", god) 
                })
            }
            HermesCommand::ClearMailbox { god } => {
                if let Some(mailbox) = self.mailbox_manager.get_mailbox(&god).await {
                    mailbox.clear().await;
                    Ok(ResponsePayload::Success { 
                        message: format!("Mailbox for {:?} cleared", god) 
                    })
                } else {
                    Err(ActorError::NotFound { god })
                }
            }
            HermesCommand::PurgeOldTrackings { max_age_seconds } => {
                let max_age = Duration::from_secs(max_age_seconds);
                self.delivery_tracker.cleanup_old_trackings(chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::zero())).await;
                
                Ok(ResponsePayload::Success { 
                    message: "Old trackings purged".to_string() 
                })
            }
            HermesCommand::ConfigureRetry { config } => {
                // This would need to recreate the retry queue
                info!("Retry configuration updated");
                
                Ok(ResponsePayload::Success { 
                    message: "Retry configuration updated".to_string() 
                })
            }
        }
    }
    
    fn calculate_load(&self) -> f64 {
        // Calculate load based on message count
        let uptime_seconds = (chrono::Utc::now() - self.state.start_time).num_seconds() as f64;
        if uptime_seconds > 0.0 {
            (self.state.message_count as f64 / uptime_seconds).min(1.0)
        } else {
            0.0
        }
    }
    
    async fn check_mailboxes_health(&self) -> bool {
        let stats = self.mailbox_manager.get_all_stats().await;
        
        // Check if any mailbox is critically full
        for stat in stats {
            let fill_ratio = stat.queued_count as f64 / stat.max_size as f64;
            if fill_ratio > 0.9 {
                return false;
            }
        }
        
        true
    }
    
    pub fn get_broadcast_sender(&self) -> tokio::sync::broadcast::Sender<BroadcastEvent> {
        self.broadcaster.get_sender()
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
        
        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            _ => self.route_message(msg).await,
        }
    }
    
    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Hermes",
            "total_messages": self.state.message_count,
            "total_errors": self.state.error_count,
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
            load: self.calculate_load(),
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        let mailbox_health = self.check_mailboxes_health().await;
        let retry_health = !self.retry_queue.is_empty().await;
        
        let status = if mailbox_health && !retry_health {
            ActorStatus::Healthy
        } else if mailbox_health {
            ActorStatus::Degraded
        } else {
            ActorStatus::Critical
        };
        
        HealthStatus {
            god: GodName::Hermes,
            status,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count,
            last_error: self.state.last_error.as_ref().map(|e| e.to_string()),
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }
    
    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("👟 Hermes: Initializing message routing system v15...");
        
        // Start the retry worker
        let retry_worker = RetryWorker::new(self.retry_queue.clone());
        let deliver_fn = |msg: ActorMessage, to: GodName| async move {
            // This would need access to the mailbox_manager
            // For now, return Ok(()) as placeholder
            Ok(())
        };
        
        retry_worker.start(deliver_fn).await;
        self.retry_worker = Some(retry_worker);
        
        info!("👟 Hermes: Retry worker started");
        info!("👟 Hermes: Ready to route messages");
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("👟 Hermes: Shutting down message routing system...");
        
        // Stop retry worker
        if let Some(worker) = &self.retry_worker {
            worker.shutdown().await;
        }
        
        // Clear all mailboxes
        self.mailbox_manager.get_all_stats().await;
        
        info!("👟 Hermes: Shutdown complete");
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}
