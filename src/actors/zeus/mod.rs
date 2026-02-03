// src/actors/zeus/mod.rs
// OLYMPUS v13 - Zeus: Gobernador Supremo
// Supervisor del Olimpo con thunderstrike y métricas

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

use crate::actors::{GodName, DivineDomain, OlympusState, OlympusMetrics, SystemStatus};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, EventPayload, ResponsePayload};
use crate::traits::message::RecoveryStrategy;
use crate::traits::supervisor_trait::{Supervisor, SupervisionTree, SupervisedActor, ActorSupervisionStatus};
use crate::errors::ActorError;

pub mod thunder;
pub mod supervisor;
pub mod metrics;
pub mod governance;
pub mod config;

pub use thunder::Thunderbolt;
pub use supervisor::SupervisionManager;
pub use metrics::ZeusMetrics;
pub use governance::GovernanceController;
pub use config::ZeusConfig;

#[derive(Debug, Clone)]
pub struct Zeus {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Zeus components
    thunderbolt: Arc<Thunderbolt>,
    supervision_manager: Arc<RwLock<SupervisionManager>>,
    metrics: Arc<RwLock<ZeusMetrics>>,
    governance: Arc<GovernanceController>,
    
    // Channels
    command_rx: mpsc::Receiver<ZeusCommand>,
    event_tx: broadcast::Sender<ZeusEvent>,
    
    // Children
    children: Arc<RwLock<Vec<GodName>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZeusCommand {
    MountOlympus,
    StartActor { actor: GodName },
    StopActor { actor: GodName, reason: String },
    RestartActor { actor: GodName },
    EmergencyShutdown { reason: String },
    GetMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZeusEvent {
    OlympusMounted { gods: Vec<GodName> },
    ActorStarted { actor: GodName },
    ActorStopped { actor: GodName, reason: String },
    ActorRecovered { actor: GodName, attempt: u32 },
    EmergencyShutdown { reason: String },
    MetricsUpdate { metrics: OlympusMetrics },
}

impl Zeus {
    pub async fn new(config: ZeusConfig) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            name: GodName::Zeus,
            state: ActorState::new(GodName::Zeus),
            config: ActorConfig::default(),
            
            thunderbolt: Arc::new(Thunderbolt::new(event_tx.clone())),
            supervision_manager: Arc::new(RwLock::new(SupervisionManager::new())),
            metrics: Arc::new(RwLock::new(ZeusMetrics::new())),
            governance: Arc::new(GovernanceController::new()),
            
            command_rx,
            event_tx,
            
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Mount the entire Olympus
    pub async fn mount_olympus(&mut self) -> Result<(), ActorError> {
        info!("⚡ Zeus: Mounting Olympus v13...");
        
        // Initialize metrics
        self.metrics.write().await.start_time = chrono::Utc::now();
        
        // Start self-evaluation cycle
        self.start_self_evaluation().await;
        
        info!("⚡ Zeus: Olympus mounted successfully");
        Ok(())
    }
    
    /// Self-evaluation cycle
    async fn start_self_evaluation(&self) {
        let metrics = self.metrics.clone();
        let supervision = self.supervision_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                
                let health = supervision.read().await.get_olympic_health();
                let mut m = metrics.write().await;
                
                m.last_health_check = chrono::Utc::now();
                m.healthy_actors = health.healthy_count;
                m.dead_actors = health.dead_count;
                
                if health.is_critical() {
                    error!("⚡ Zeus: Olympic health is CRITICAL: {:?}", health);
                }
            }
        });
    }
    
    /// Broadcast thunderstrike
    pub fn thunderstrike(&self, event: ZeusEvent) {
        let _ = self.thunderbolt.broadcast(event);
    }
}

#[async_trait]
impl OlympianActor for Zeus {
    fn name(&self) -> GodName {
        GodName::Zeus
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Governance
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
            "name": "Zeus",
            "uptime_seconds": (chrono::Utc::now() - self.state.start_time).num_seconds(),
            "metrics": self.metrics.read().await.clone(),
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Zeus,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Zeus,
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
        self.mount_olympus().await
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        self.thunderstrike(ZeusEvent::EmergencyShutdown { reason: "System shutdown".to_string() });
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Zeus {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::StartActor { actor } => {
                self.children.write().await.push(actor);
                self.thunderstrike(ZeusEvent::ActorStarted { actor });
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            CommandPayload::StopActor { actor, reason } => {
                self.children.write().await.retain(|&g| g != actor);
                self.thunderstrike(ZeusEvent::ActorStopped { actor, reason });
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            CommandPayload::EmergencyShutdown { reason } => {
                self.shutdown().await?;
                self.thunderstrike(ZeusEvent::EmergencyShutdown { reason });
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unknown command".to_string(), code: 400 }),
        }
    }
    
    async fn handle_query(&self, _query: crate::traits::message::QueryPayload) -> Result<ResponsePayload, ActorError> {
        let metrics = self.metrics.read().await.clone();
        Ok(ResponsePayload::Data { data: serde_json::to_value(metrics).unwrap() })
    }
    
    async fn handle_event(&mut self, _event: EventPayload) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
    }
}
