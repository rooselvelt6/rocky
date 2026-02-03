// src/actors/erinyes/mod.rs
// OLYMPUS v13 - Erinyes: Guardián de Integridad
// Sistema inmunitario con heartbeat, recovery y dead letter

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration, Instant};
use tracing::{info, warn, error};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload};
use crate::traits::message::RecoveryStrategy;
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;

pub mod heartbeat;
pub mod recovery;
pub mod dead_letter;
pub mod watchdog;
pub mod alerts;

pub use heartbeat::HeartbeatMonitor;
pub use recovery::RecoveryEngine;
pub use dead_letter::DeadLetterQueue;
pub use watchdog::Watchdog;
pub use alerts::AlertSystem;

#[derive(Debug, Clone)]
pub struct Erinyes {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    heartbeat_monitor: Arc<RwLock<HeartbeatMonitor>>,
    recovery_engine: Arc<RwLock<RecoveryEngine>>,
    dead_letter_queue: Arc<DeadLetterQueue>,
    watchdog: Arc<Watchdog>,
    alert_system: Arc<AlertSystem>,
    
    // Valkey for dead letter persistence
    valkey: Arc<ValkeyStore>,
    
    // Channel for commands
    command_rx: mpsc::Receiver<ErinyesCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErinyesCommand {
    RegisterActor { actor: GodName, config: HeartbeatConfig },
    UnregisterActor { actor: GodName },
    ReceiveHeartbeat { actor: GodName },
    TriggerRecovery { actor: GodName },
    ConfigureAlerts { actor: GodName, alert_config: AlertConfig },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub strategy: RecoveryStrategy,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 500,
            timeout_ms: 600,
            strategy: RecoveryStrategy::OneForOne,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub threshold: u64,
    pub escalation_to_zeus: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3,
            escalation_to_zeus: true,
        }
    }
}

impl Erinyes {
    pub async fn new(valkey: Arc<ValkeyStore>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        Self {
            name: GodName::Erinyes,
            state: ActorState::new(GodName::Erinyes),
            config: ActorConfig::default(),
            
            heartbeat_monitor: Arc::new(RwLock::new(HeartbeatMonitor::new())),
            recovery_engine: Arc::new(RwLock::new(RecoveryEngine::new())),
            dead_letter_queue: Arc::new(DeadLetterQueue::new(valkey.clone())),
            watchdog: Arc::new(Watchdog::new()),
            alert_system: Arc::new(AlertSystem::new()),
            
            valkey,
            command_rx,
        }
    }
    
    /// Start the heartbeat monitoring cycle
    pub fn start_monitoring(&self) {
        let monitor = self.heartbeat_monitor.clone();
        let watchdog = self.watchdog.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                monitor.read().await.check_all(&watchdog).await;
            }
        });
    }
}

#[async_trait]
impl OlympianActor for Erinyes {
    fn name(&self) -> GodName {
        GodName::Erinyes
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Integrity
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
            "name": "Erinyes",
            "monitored_actors": self.heartbeat_monitor.read().await.monitored_count(),
            "dead_letters": self.dead_letter_queue.len().await,
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Erinyes,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Erinyes,
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
        self.start_monitoring();
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Erinyes {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::ConfigureHeartbeat { actor, interval_ms } => {
                self.heartbeat_monitor.write().await.set_interval(actor, interval_ms);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            CommandPayload::RecoverActor { actor, strategy } => {
                self.recovery_engine.write().await.trigger_recovery(actor, strategy).await;
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unknown command".to_string(), code: 400 }),
        }
    }
    
    async fn handle_query(&self, _query: crate::traits::message::QueryPayload) -> Result<ResponsePayload, ActorError> {
        let stats = self.heartbeat_monitor.read().await.get_stats().await;
        Ok(ResponsePayload::Data { data: serde_json::to_value(stats).unwrap() })
    }
    
    async fn handle_event(&mut self, event: crate::traits::message::EventPayload) -> Result<ResponsePayload, ActorError> {
        match event {
            crate::traits::message::EventPayload::ActorPanicked { actor, error } => {
                self.watchdog.write().await.report_panic(&actor, &error);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() }),
        }
    }
}
