// src/actors/erinyes/mod.rs
// OLYMPUS v16 - Erinyes: Guardiana Suprema de Integridad
// Implementación completa sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, RecoveryStrategy};
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;

pub mod heartbeat;
pub mod recovery;
pub mod dead_letter;
pub mod watchdog;
pub mod alerts;

pub use heartbeat::{HeartbeatMonitor, HeartbeatState, HeartbeatConfig};
pub use recovery::{RecoveryEngine, RecoveryUrgency};
pub use dead_letter::DeadLetterQueue;
pub use watchdog::{Watchdog, WatchdogEventType, WatchdogSeverity, SystemStatus};
pub use alerts::{AlertSystem, AlertSeverity, AlertChannel};

/// Erinyes Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErinyesCommand {
    RegisterActor { actor: GodName, config: HeartbeatConfig },
    UnregisterActor { actor: GodName },
    ReceiveHeartbeat { actor: GodName, latency_ms: Option<u64> },
    TriggerRecovery { actor: GodName, strategy: RecoveryStrategy, urgency: RecoveryUrgency },
    EnableAutoRecovery { enabled: bool },
    GetSystemHealth,
    AcknowledgeAlert { alert_id: String, acknowledged_by: String },
    ResolveAlert { alert_id: String, resolution_note: Option<String> },
}

/// Erinyes State for Ractor
pub struct ErinyesState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    
    pub heartbeat_monitor: Arc<HeartbeatMonitor>,
    pub recovery_engine: Arc<RecoveryEngine>,
    pub dead_letter_queue: Arc<DeadLetterQueue>,
    pub watchdog: Arc<Watchdog>,
    pub alert_system: Arc<AlertSystem>,
    
    pub trinity_members: Vec<GodName>,
    pub monitoring_interval_ms: u64,
    pub auto_recovery_enabled: bool,
    pub escalation_enabled: bool,
    
    pub valkey: Arc<ValkeyStore>,
}

pub struct Erinyes;

impl Erinyes {
    async fn register_trinity_members(&self, state: &ErinyesState) {
        for god in &state.trinity_members {
            let config = HeartbeatConfig {
                interval_ms: 250,
                timeout_ms: 500,
                strategy: RecoveryStrategy::OneForOne,
            };
            let _ = state.heartbeat_monitor.register(god.clone(), Some(config)).await;
        }
    }

    async fn start_monitoring(&self, state: &ErinyesState) {
        let monitor = state.heartbeat_monitor.clone();
        let watchdog = state.watchdog.clone();
        let recovery = state.recovery_engine.clone();
        let alert_system = state.alert_system.clone();
        let trinity = state.trinity_members.clone();
        let auto_recovery = state.auto_recovery_enabled;
        let escalation = state.escalation_enabled;
        let interval_ms = state.monitoring_interval_ms;
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));
            loop {
                ticker.tick().await;
                monitor.check_all(|actor, s| {
                    let is_trinity = trinity.contains(&actor);
                    let watchdog = watchdog.clone();
                    let alert_system = alert_system.clone();
                    let recovery = recovery.clone();
                    let actor_id = actor.clone();
                    let status = s.status.clone();
                    let strategy = s.config.strategy.clone();
                    let misses = s.consecutive_misses;

                    tokio::spawn(async move {
                        watchdog.report_event(
                            WatchdogEventType::HealthCheckFailed,
                            Some(actor_id.clone()),
                            format!("Health check failed for {:?}", actor_id),
                            if is_trinity { WatchdogSeverity::Critical } else { WatchdogSeverity::Error },
                            None,
                        ).await;

                        if auto_recovery && status == ActorStatus::Dead {
                            if is_trinity && escalation {
                                let _ = alert_system.create_alert(
                                    AlertSeverity::Critical,
                                    GodName::Erinyes,
                                    format!("TRINITY MEMBER DOWN: {:?}", actor_id),
                                    "Critical level escalation".to_string(),
                                ).await;
                            }
                            let _ = recovery.request_recovery(
                                actor_id,
                                strategy,
                                if is_trinity { RecoveryUrgency::Critical } else { RecoveryUrgency::High }
                            ).await;
                        }
                    });
                }).await;
            }
        });
    }
}

#[async_trait]
impl Actor for Erinyes {
    type Msg = ActorMessage;
    type State = ErinyesState;
    type Arguments = (ActorConfig, Arc<ValkeyStore>);

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let (config, valkey) = args;
        let alert_system = Arc::new(AlertSystem::new());
        let heartbeat_monitor = Arc::new(HeartbeatMonitor::new(alert_system.clone()));
        let recovery_engine = Arc::new(RecoveryEngine::new(alert_system.clone()));
        let watchdog = Arc::new(Watchdog::new());
        let dead_letter_queue = Arc::new(DeadLetterQueue::new(valkey.clone()));

        alert_system.start_processor().await;

        let state = ErinyesState {
            name: GodName::Erinyes,
            metadata: ActorState::new(GodName::Erinyes),
            config,
            heartbeat_monitor,
            recovery_engine,
            dead_letter_queue,
            watchdog,
            alert_system,
            trinity_members: vec![GodName::Zeus, GodName::Hades, GodName::Poseidon],
            monitoring_interval_ms: 500,
            auto_recovery_enabled: true,
            escalation_enabled: true,
            valkey,
        };
        Ok(state)
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        self.register_trinity_members(state).await;
        self.start_monitoring(state).await;
        
        let recovery_fn = |actor: GodName| {
            Box::pin(async move {
                info!("🔄 Recovery performed for {:?}", actor);
                Ok(())
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ActorError>> + Send>>
        };
        state.recovery_engine.start_recovery_worker(recovery_fn).await;
        
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
            MessagePayload::Event(event) => {
                let _ = self.handle_event(event, state).await;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Erinyes {
    async fn handle_command(&self, cmd: CommandPayload, state: &mut ErinyesState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                if let Ok(erinyes_cmd) = serde_json::from_value::<ErinyesCommand>(data) {
                    self.execute_erinyes_command(erinyes_cmd, state).await
                } else {
                    Err(ActorError::InvalidCommand { god: GodName::Erinyes, reason: "Invalid format".to_string() })
                }
            }
            _ => Err(ActorError::InvalidCommand { god: GodName::Erinyes, reason: "Unsupported".to_string() }),
        }
    }

    async fn execute_erinyes_command(&self, cmd: ErinyesCommand, state: &mut ErinyesState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            ErinyesCommand::RegisterActor { actor, config } => {
                state.heartbeat_monitor.register(actor, Some(config)).await
                    .map_err(|e| ActorError::Unknown { god: GodName::Erinyes, message: e })?;
                Ok(ResponsePayload::Success { message: format!("Monitoring actor {:?}", actor) })
            }
            ErinyesCommand::ReceiveHeartbeat { actor, latency_ms } => {
                state.heartbeat_monitor.receive_heartbeat(actor, latency_ms).await;
                Ok(ResponsePayload::Ack { message_id: "erinyes".to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Not implemented".to_string(), code: 501 }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, state: &ErinyesState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::HealthStatus => {
                let health = state.watchdog.check_system_health().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(health).unwrap_or_default() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unsupported query".to_string(), code: 400 }),
        }
    }

    async fn handle_event(&self, event: crate::traits::message::EventPayload, state: &mut ErinyesState) -> Result<ResponsePayload, ActorError> {
        match event {
            crate::traits::message::EventPayload::ActorPanicked { actor, error } => {
                state.watchdog.report_panic(actor, error.clone(), None).await;
                if state.auto_recovery_enabled {
                    let _ = state.recovery_engine.request_recovery(actor, RecoveryStrategy::OneForOne, RecoveryUrgency::High).await;
                }
                Ok(ResponsePayload::Ack { message_id: "erinyes".to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: "erinyes".to_string() }),
        }
    }
}
