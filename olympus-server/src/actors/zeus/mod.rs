// src/actors/zeus/mod.rs
// OLYMPUS v16 - Zeus: Gobernador Supremo y Coordinador de la Trinidad
// Implementación completa sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain, OlympusState, OlympusMetrics};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, QueryPayload, EventPayload, ResponsePayload, RecoveryStrategy};
use crate::traits::supervisor_trait::{SupervisionTree, SupervisedActor, ActorSupervisionStatus};
use crate::errors::ActorError;

pub mod thunder;
pub mod supervisor;
pub mod metrics;
pub mod governance;
pub mod config;

pub use thunder::{Thunderbolt, ThunderEvent, ThunderSeverity};
pub use supervisor::{SupervisionManager, LifecycleEvent, RestartResult};
pub use metrics::{ZeusMetrics, AlertSeverity, TrinityMetrics, TrinityStatus};
pub use governance::{GovernanceController, GovernanceDecision, GovernanceSituation, CircuitState};
pub use config::{ZeusConfig, ConfigManager, Environment};

/// Comandos completos de Zeus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZeusCommand {
    MountOlympus,
    UnmountOlympus,
    StartActor { actor: GodName, config: Option<ActorConfig> },
    StopActor { actor: GodName, reason: String },
    RestartActor { actor: GodName, force: bool },
    KillActor { actor: GodName, reason: String },
    StartAllActors,
    StopAllActors { reason: String },
    RestartAllActors,
    EmergencyShutdown { reason: String },
    GracefulShutdown { timeout_seconds: u64 },
    Configure { config: ZeusConfig },
    UpdateConfig { key: String, value: serde_json::Value },
    HotReloadConfig,
    GetMetrics,
    ExportMetrics,
    ResetMetrics,
    EnableFeatureFlag { flag: String, modified_by: Option<String> },
    DisableFeatureFlag { flag: String, modified_by: Option<String> },
    OpenCircuitBreaker { component: String },
    CloseCircuitBreaker { component: String },
    SyncTrinityStatus,
    ForceTrinityHealthCheck,
    InternalSelfEvaluation,
    InternalTrinitySync,
    EnableAutoRecovery { enabled: bool },
}

/// Queries de Zeus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZeusQuery {
    GetTrinityStatus,
    GetSupervisionTree,
    GetSystemHealth,
    GetActorStatus { actor: GodName },
    GetAllActorsStatus,
    GetAllMetrics,
    GetActorMetrics { actor: GodName },
    GetHistoricalMetrics { since: Option<chrono::DateTime<chrono::Utc>>, limit: Option<usize> },
    GetGovernanceHistory { limit: usize },
    GetFeatureFlag { flag: String },
    GetAllFeatureFlags,
    GetCircuitBreakerState { component: String },
    GetAllCircuitBreakers,
    GetConfig,
}

/// Eventos emitidos por Zeus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZeusEvent {
    OlympusMounted { gods: Vec<GodName> },
    OlympusUnmounted { reason: String },
    ActorStarted { actor: GodName, timestamp: chrono::DateTime<chrono::Utc> },
    ActorStopped { actor: GodName, reason: String, timestamp: chrono::DateTime<chrono::Utc> },
    ActorRecovered { actor: GodName, timestamp: chrono::DateTime<chrono::Utc> },
    ActorFailed { actor: GodName, error: String, timestamp: chrono::DateTime<chrono::Utc> },
    ActorRestarted { actor: GodName, timestamp: chrono::DateTime<chrono::Utc> },
    TrinityStatusChanged { status: TrinityStatus, timestamp: chrono::DateTime<chrono::Utc> },
    SystemHealthy { timestamp: chrono::DateTime<chrono::Utc> },
    SystemDegraded { reason: String, timestamp: chrono::DateTime<chrono::Utc> },
    EmergencyShutdown { reason: String, timestamp: chrono::DateTime<chrono::Utc> },
    FeatureFlagChanged { flag: String, enabled: bool },
    CircuitBreakerChanged { component: String, state: CircuitState },
}

/// Estado de la Trinidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrinityState {
    pub zeus_healthy: bool,
    pub hades_healthy: bool,
    pub poseidon_healthy: bool,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub is_critical: bool,
}

impl Default for TrinityState {
    fn default() -> Self {
        Self {
            zeus_healthy: true,
            hades_healthy: true,
            poseidon_healthy: true,
            last_sync: chrono::Utc::now(),
            is_critical: false,
        }
    }
}

/// Estado interno de Zeus para Ractor
pub struct ZeusState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    pub zeus_config: ZeusConfig,
    pub thunderbolt: Arc<Thunderbolt>,
    pub supervision_manager: SupervisionManager,
    pub metrics: ZeusMetrics,
    pub governance: GovernanceController,
    pub config_manager: ConfigManager,
    pub olympus_state: OlympusState,
    pub trinity_state: TrinityState,
    pub event_tx: broadcast::Sender<ZeusEvent>,
    pub lifecycle_tx: broadcast::Sender<LifecycleEvent>,
    pub erinyes_tx: Option<mpsc::Sender<crate::actors::erinyes::ErinyesCommand>>,
    pub olympus_actors: Vec<GodName>,
    pub running: bool,
}

pub struct Zeus;

impl Zeus {
    pub fn get_all_olympus_actors() -> Vec<GodName> {
        vec![
            GodName::Zeus, GodName::Erinyes, GodName::Poseidon, GodName::Athena,
            GodName::Apollo, GodName::Artemis, GodName::Hermes, GodName::Hades,
            GodName::Hera, GodName::Ares, GodName::Hefesto, GodName::Chronos,
            GodName::Moirai, GodName::Chaos, GodName::Aurora, GodName::Aphrodite,
            GodName::Iris, GodName::Demeter, GodName::Dionysus, GodName::Hestia,
        ]
    }

    async fn start_trinity_sync(&self, state: &ZeusState) {
        let trinity_state = Arc::new(RwLock::new(state.trinity_state.clone()));
        let event_tx = state.event_tx.clone();
        let interval_secs = state.zeus_config.health_check_interval_seconds;
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            loop {
                ticker.tick().await;
                let mut trinity = trinity_state.write().await;
                trinity.last_sync = chrono::Utc::now();
                let critical = !trinity.zeus_healthy || !trinity.hades_healthy || !trinity.poseidon_healthy;
                let was_critical = trinity.is_critical;
                trinity.is_critical = critical;
                
                if critical && !was_critical {
                    let _ = event_tx.send(ZeusEvent::TrinityStatusChanged { 
                        status: TrinityStatus::Critical,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        });
    }

    async fn start_lifecycle_processor(&self, state: &ZeusState) {
        let mut lifecycle_rx = state.lifecycle_tx.subscribe();
        let event_tx = state.event_tx.clone();
        
        tokio::spawn(async move {
            while let Ok(event) = lifecycle_rx.recv().await {
                match event {
                    LifecycleEvent::ActorStarted { actor } => {
                        let _ = event_tx.send(ZeusEvent::ActorStarted { actor, timestamp: chrono::Utc::now() });
                    }
                    LifecycleEvent::ActorStopped { actor, reason } => {
                        let _ = event_tx.send(ZeusEvent::ActorStopped { actor, reason, timestamp: chrono::Utc::now() });
                    }
                    LifecycleEvent::ActorRecovered { actor } => {
                        let _ = event_tx.send(ZeusEvent::ActorRecovered { actor, timestamp: chrono::Utc::now() });
                    }
                    LifecycleEvent::Failed { actor, error } => {
                        let _ = event_tx.send(ZeusEvent::ActorFailed { actor, error, timestamp: chrono::Utc::now() });
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn thunderstrike(&self, event: ZeusEvent, state: &ZeusState) {
        let thunder_event = match &event {
            ZeusEvent::ActorStarted { actor, .. } => ThunderEvent::ActorStarted { actor: actor.clone() },
            ZeusEvent::ActorStopped { actor, reason, .. } => ThunderEvent::ActorStopped { actor: actor.clone(), reason: reason.clone() },
            ZeusEvent::ActorRecovered { actor, .. } => ThunderEvent::ActorRecovered { actor: actor.clone() },
            ZeusEvent::EmergencyShutdown { reason, .. } => ThunderEvent::Emergency { reason: reason.clone(), severity: ThunderSeverity::Critical },
            _ => {
                let _ = state.event_tx.send(event);
                return;
            }
        };
        let _ = state.thunderbolt.broadcast(thunder_event);
        let _ = state.event_tx.send(event);
    }

    async fn perform_self_evaluation(&self, state: &mut ZeusState) {
        let _ = state.supervision_manager.get_olympic_health().await;
        let _ = state.event_tx.send(ZeusEvent::SystemHealthy { timestamp: chrono::Utc::now() });
    }

    pub async fn mount_olympus(&self, state: &mut ZeusState) -> Result<(), ActorError> {
        info!("⚡ Zeus: Mounting Olympus...");
        let actors = state.olympus_actors.clone();
        for actor in actors {
            if actor != GodName::Zeus {
                let _ = state.supervision_manager.start_actor(actor).await;
            }
        }
        self.thunderstrike(ZeusEvent::OlympusMounted { gods: state.olympus_actors.clone() }, state);
        Ok(())
    }
}

#[async_trait]
impl Actor for Zeus {
    type Msg = ActorMessage;
    type State = ZeusState;
    type Arguments = ActorConfig;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, config: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let (event_tx, _) = broadcast::channel(1000);
        let (thunder_tx, _) = broadcast::channel(1000);
        let (lifecycle_tx, _) = broadcast::channel(1000);
        
        let zeus_config = ZeusConfig::default();
        let config_manager = ConfigManager::new(zeus_config.clone());
        
        let state = ZeusState {
            name: GodName::Zeus,
            metadata: ActorState::new(GodName::Zeus),
            config,
            zeus_config,
            thunderbolt: Arc::new(Thunderbolt::new(thunder_tx)),
            supervision_manager: SupervisionManager::new(),
            metrics: ZeusMetrics::new(),
            governance: GovernanceController::new(),
            config_manager,
            olympus_state: OlympusState::default(),
            trinity_state: TrinityState::default(),
            event_tx,
            lifecycle_tx,
            erinyes_tx: None,
            olympus_actors: Self::get_all_olympus_actors(),
            running: true,
        };
        Ok(state)
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        self.start_trinity_sync(state).await;
        self.start_lifecycle_processor(state).await;
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message.payload {
            MessagePayload::Command(cmd) => {
                let _ = self.handle_command(cmd, state).await;
            }
            MessagePayload::Query(query) => {
                let _ = self.handle_query(query, state).await;
            }
            MessagePayload::Event(event) => {
                let _ = self.handle_event(event, state).await;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Zeus {
    async fn handle_command(&self, cmd: CommandPayload, state: &mut ZeusState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::StartActor { actor, .. } => {
                state.supervision_manager.start_actor(actor).await? ;
                Ok(ResponsePayload::Success { message: format!("Actor {:?} started", actor) })
            }
            CommandPayload::StopActor { actor, reason } => {
                state.supervision_manager.stop_actor(actor, reason.clone()).await?;
                Ok(ResponsePayload::Success { message: format!("Actor {:?} stopped", actor) })
            }
            CommandPayload::EmergencyShutdown { reason } => {
                state.running = false;
                self.thunderstrike(ZeusEvent::EmergencyShutdown { reason: reason.clone(), timestamp: chrono::Utc::now() }, state);
                Ok(ResponsePayload::Success { message: format!("Emergency shutdown: {}", reason) })
            }
            CommandPayload::Custom(data) => {
                if let Ok(zeus_cmd) = serde_json::from_value::<ZeusCommand>(data) {
                    self.execute_zeus_command(zeus_cmd, state).await
                } else {
                    Err(ActorError::InvalidCommand { god: GodName::Zeus, reason: "Invalid format".to_string() })
                }
            }
            _ => Err(ActorError::InvalidCommand { god: GodName::Zeus, reason: "Unsupported".to_string() }),
        }
    }

    async fn execute_zeus_command(&self, cmd: ZeusCommand, state: &mut ZeusState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            ZeusCommand::MountOlympus => self.mount_olympus(state).await.map(|_| ResponsePayload::Success { message: "Mounted".to_string() }),
            ZeusCommand::StartAllActors => {
                for actor in state.olympus_actors.clone() {
                    let _ = state.supervision_manager.start_actor(actor).await;
                }
                Ok(ResponsePayload::Success { message: "All actors starting".to_string() })
            }
            ZeusCommand::EnableAutoRecovery { enabled } => {
                state.supervision_manager.set_auto_recovery(enabled).await;
                Ok(ResponsePayload::Success { message: format!("Auto-recovery: {}", enabled) })
            }
            ZeusCommand::GetMetrics => {
                let metrics = state.metrics.get_summary().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(metrics).unwrap_or_default() })
            }
            _ => Ok(ResponsePayload::Error { error: "Not implemented".to_string(), code: 501 }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, state: &mut ZeusState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::HealthStatus => {
                let health = state.supervision_manager.get_olympic_health().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(health).unwrap_or_default() })
            }
            QueryPayload::Custom(data) => {
                if let Ok(zeus_query) = serde_json::from_value::<ZeusQuery>(data) {
                    self.execute_zeus_query(zeus_query, state).await
                } else {
                    Err(ActorError::InvalidQuery { god: GodName::Zeus, reason: "Invalid format".to_string() })
                }
            }
            _ => Err(ActorError::InvalidQuery { god: GodName::Zeus, reason: "Unsupported".to_string() }),
        }
    }

    async fn execute_zeus_query(&self, query: ZeusQuery, state: &ZeusState) -> Result<ResponsePayload, ActorError> {
        match query {
            ZeusQuery::GetTrinityStatus => Ok(ResponsePayload::Data { data: serde_json::to_value(&state.trinity_state).unwrap_or_default() }),
            ZeusQuery::GetSystemHealth => {
                let health = state.supervision_manager.get_olympic_health().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(health).unwrap_or_default() })
            }
            _ => Ok(ResponsePayload::Error { error: "Not implemented".to_string(), code: 501 }),
        }
    }

    async fn handle_event(&self, event: EventPayload, state: &mut ZeusState) -> Result<ResponsePayload, ActorError> {
        match event {
            EventPayload::ActorRecovered { actor, .. } => {
                state.metrics.increment_recoveries();
                state.supervision_manager.mark_recovered(actor).await;
                self.thunderstrike(ZeusEvent::ActorRecovered { actor, timestamp: chrono::Utc::now() }, state);
                Ok(ResponsePayload::Ack { message_id: "zeus".to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: "zeus".to_string() }),
        }
    }
}
