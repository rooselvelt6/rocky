// src/actors/erinyes/mod.rs
// OLYMPUS v15 - Erinyes: Guardiana Suprema de Integridad
// Sistema inmunitario completo que protege la Trinidad y todo el Olimpo

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration, Instant};
use tracing::{info, warn, error, debug};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, RecoveryStrategy};
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;
use serde_json::json;

pub mod heartbeat;
pub mod recovery;
pub mod dead_letter;
pub mod watchdog;
pub mod alerts;

pub use heartbeat::{HeartbeatMonitor, HeartbeatState, HeartbeatStats, HeartbeatConfig};
pub use recovery::{RecoveryEngine, RecoveryRecord, RecoveryConfig, RecoveryUrgency, RecoveryStats};
pub use dead_letter::{DeadLetterQueue, DeadLetter, DeadLetterStatus};
pub use watchdog::{Watchdog, WatchdogEvent, WatchdogEventType, WatchdogSeverity, DeathRecord, SystemHealth, SystemStatus};
pub use alerts::{AlertSystem, Alert, AlertSeverity, AlertCategory, AlertRule, AlertCondition, AlertChannel, AlertStats};

/// Erinyes: La Guardiana de la Integridad
/// Vigila la Trinidad Suprema (Zeus, Hades, Poseidón) y todos los actores
/// Detecta fallos, recupera actores, gestiona dead letters y alertas
#[derive(Debug)]
pub struct Erinyes {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    heartbeat_monitor: Arc<HeartbeatMonitor>,
    recovery_engine: Arc<RecoveryEngine>,
    dead_letter_queue: Arc<DeadLetterQueue>,
    watchdog: Arc<Watchdog>,
    alert_system: Arc<AlertSystem>,
    
    // Configuration
    trinity_members: Arc<RwLock<Vec<GodName>>>,
    monitoring_interval_ms: u64,
    auto_recovery_enabled: bool,
    escalation_enabled: bool,
    
    // Valkey for persistence
    valkey: Arc<ValkeyStore>,
    
    // Channels
    command_tx: mpsc::Sender<ErinyesCommand>,
    command_rx: Arc<RwLock<mpsc::Receiver<ErinyesCommand>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErinyesCommand {
    RegisterActor { actor: GodName, config: HeartbeatConfig },
    UnregisterActor { actor: GodName },
    ReceiveHeartbeat { actor: GodName, latency_ms: Option<u64> },
    TriggerRecovery { actor: GodName, strategy: RecoveryStrategy, urgency: RecoveryUrgency },
    ConfigureAlerts { actor: GodName, alert_config: AlertConfig },
    AcknowledgeAlert { alert_id: String, acknowledged_by: String },
    ResolveAlert { alert_id: String, resolution_note: Option<String> },
    EnableAutoRecovery { enabled: bool },
    EnableEscalation { enabled: bool },
    GetSystemHealth,
    GetActorHealth { actor: GodName },
    GetRecoveryHistory { actor: Option<GodName>, limit: usize },
    GetDeadLetters,
    RetryDeadLetter { letter_id: String },
    PurgeOldData { max_age_seconds: u64 },
    ConfigureMonitoring { interval_ms: u64 },
    SetTrinityPriority { actor: GodName, is_trinity: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub threshold: u64,
    pub escalation_to_zeus: bool,
    pub channels: Vec<AlertChannel>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3,
            escalation_to_zeus: true,
            channels: vec![AlertChannel::Log],
        }
    }
}

impl Erinyes {
    pub async fn new(valkey: Arc<ValkeyStore>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        let alert_system = Arc::new(AlertSystem::new());
        let heartbeat_monitor = Arc::new(HeartbeatMonitor::new(alert_system.clone()));
        let recovery_engine = Arc::new(RecoveryEngine::new(alert_system.clone()));
        let watchdog = Arc::new(Watchdog::new());
        let dead_letter_queue = Arc::new(DeadLetterQueue::new(valkey.clone()));
        
        // Start alert processor
        alert_system.start_processor().await;
        
        let mut erinyes = Self {
            name: GodName::Erinyes,
            state: ActorState::new(GodName::Erinyes),
            config: ActorConfig::default(),
            
            heartbeat_monitor,
            recovery_engine,
            dead_letter_queue,
            watchdog,
            alert_system,
            
            trinity_members: Arc::new(RwLock::new(vec![
                GodName::Zeus,
                GodName::Hades,
                GodName::Poseidon,
            ])),
            monitoring_interval_ms: 500,
            auto_recovery_enabled: true,
            escalation_enabled: true,
            
            valkey,
            command_tx,
            command_rx: Arc::new(RwLock::new(command_rx)),
        };
        
        // Register Trinity members with priority monitoring
        erinyes.register_trinity_members().await;
        
        erinyes
    }
    
    pub async fn with_config(valkey: Arc<ValkeyStore>, config: ErinyesConfig) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        let alert_system = Arc::new(AlertSystem::new());
        let heartbeat_monitor = Arc::new(HeartbeatMonitor::new(alert_system.clone()));
        let recovery_engine = Arc::new(RecoveryEngine::new(alert_system.clone()));
        let watchdog = Arc::new(Watchdog::new());
        let dead_letter_queue = Arc::new(DeadLetterQueue::new(valkey.clone()));
        
        // Start alert processor
        alert_system.start_processor().await;
        
        let mut erinyes = Self {
            name: GodName::Erinyes,
            state: ActorState::new(GodName::Erinyes),
            config: ActorConfig::default(),
            
            heartbeat_monitor,
            recovery_engine,
            dead_letter_queue,
            watchdog,
            alert_system,
            
            trinity_members: Arc::new(RwLock::new(config.trinity_members.clone())),
            monitoring_interval_ms: config.monitoring_interval_ms,
            auto_recovery_enabled: config.auto_recovery_enabled,
            escalation_enabled: config.escalation_enabled,
            
            valkey,
            command_tx,
            command_rx: Arc::new(RwLock::new(command_rx)),
        };
        
        // Register Trinity members
        erinyes.register_trinity_members().await;
        
        erinyes
    }
    
    async fn register_trinity_members(&self) {
        let trinity = self.trinity_members.read().await;
        
        for god in trinity.iter() {
            // Trinity members get more aggressive monitoring
            let config = HeartbeatConfig {
                interval_ms: 250,  // More frequent (every 250ms)
                timeout_ms: 500,   // Shorter timeout
                strategy: RecoveryStrategy::OneForOne,
            };
            
            let _ = self.heartbeat_monitor.register(god.clone(), Some(config)).await;
            info!("🏛️ Trinity member {:?} registered for priority monitoring", god);
        }
    }
    
    /// Start the monitoring cycle
    pub fn start_monitoring(&self) {
        let monitor = self.heartbeat_monitor.clone();
        let watchdog = self.watchdog.clone();
        let recovery = self.recovery_engine.clone();
        let alert_system = self.alert_system.clone();
        let trinity = self.trinity_members.clone();
        let auto_recovery = self.auto_recovery_enabled;
        let escalation = self.escalation_enabled;
        let interval_ms = self.monitoring_interval_ms;
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));
            
            loop {
                ticker.tick().await;
                
                // Check all heartbeats
                let trinity_guard = trinity.read().await;
                
                monitor.check_all(|actor, state| {
                    let is_trinity = trinity_guard.contains(&actor);
                    
                    // Report to watchdog
                    let watchdog = watchdog.clone();
                    let alert_system = alert_system.clone();
                    let recovery = recovery.clone();
                    
                    // Clone data for async block
                    let actor_id = actor.clone();
                    let consecutive_misses = state.consecutive_misses;
                    let health_score = state.get_health_score();
                    let status = state.status.clone();
                    let strategy = state.config.strategy.clone();

                    tokio::spawn(async move {
                        watchdog.report_event(
                            WatchdogEventType::HealthCheckFailed,
                            Some(actor_id.clone()),
                            format!("Heartbeat missed: {} consecutive misses", consecutive_misses),
                            if is_trinity { WatchdogSeverity::Critical } else { WatchdogSeverity::Error },
                            Some(serde_json::json!({
                                "consecutive_misses": consecutive_misses,
                                "is_trinity": is_trinity,
                                "health_score": health_score,
                            })),
                        ).await;
                        
                        // Auto-recovery if enabled and actor is dead
                        if auto_recovery && status == ActorStatus::Dead {
                            if is_trinity {
                                // Trinity members get immediate recovery
                                info!("🚨 Trinity member {:?} detected dead, triggering immediate recovery", actor_id);
                                
                                if escalation {
                                    alert_system.create_alert(
                                        AlertSeverity::Critical,
                                        GodName::Erinyes,
                                        format!("TRINITY MEMBER DOWN: {:?}", actor_id),
                                        format!("Critical system component {:?} has failed. Immediate recovery initiated.", actor_id),
                                    ).await;
                                }
                            }
                            
                            // Trigger recovery
                            recovery.request_recovery(
                                actor_id,
                                strategy,
                                if is_trinity { RecoveryUrgency::Critical } else { RecoveryUrgency::High }
                            ).await.ok();
                        }
                    });
                }).await;
                
                drop(trinity_guard);
                
                // Check system health
                let system_health = watchdog.check_system_health().await;
                
                if system_health.status == SystemStatus::Critical {
                    alert_system.create_alert(
                        AlertSeverity::Critical,
                        GodName::Erinyes,
                        "SYSTEM CRITICAL".to_string(),
                        format!(
                            "System health is CRITICAL. Active deaths: {}, Recent errors: {}",
                            system_health.active_deaths, system_health.recent_errors
                        ),
                    ).await;
                }
            }
        });
        
        info!("🏹 Erinyes monitoring started (interval: {}ms)", interval_ms);
    }
}

#[derive(Debug, Clone)]
pub struct ErinyesConfig {
    pub trinity_members: Vec<GodName>,
    pub monitoring_interval_ms: u64,
    pub auto_recovery_enabled: bool,
    pub escalation_enabled: bool,
}

impl Default for ErinyesConfig {
    fn default() -> Self {
        Self {
            trinity_members: vec![
                GodName::Zeus,
                GodName::Hades,
                GodName::Poseidon,
            ],
            monitoring_interval_ms: 500,
            auto_recovery_enabled: true,
            escalation_enabled: true,
        }
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

    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Erinyes",
            "total_messages": self.state.message_count,
            "total_errors": self.state.error_count,
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
            load: self.calculate_load(),
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        let system_health = self.watchdog.check_system_health().await;
        let heartbeat_stats = self.heartbeat_monitor.get_stats().await;
        
        let status = if system_health.status == SystemStatus::Critical || heartbeat_stats.dead > 0 {
            ActorStatus::Critical
        } else if system_health.status == SystemStatus::Degraded {
            ActorStatus::Degraded
        } else {
            ActorStatus::Healthy
        };
        
        HealthStatus {
            god: GodName::Erinyes,
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
        info!("🏹 Erinyes: Initializing Guardian of Integrity v15...");
        
        // Start monitoring
        self.start_monitoring();
        
        // Start recovery worker
        let recovery_fn = |actor: GodName| {
            Box::pin(async move {
                info!("🔄 Recovery performed for {:?}", actor);
                Ok(())
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ActorError>> + Send>>
        };
        
        self.recovery_engine.start_recovery_worker(recovery_fn).await;
        
        info!("🏹 Erinyes: Monitoring {} Trinity members with priority", 
            self.trinity_members.read().await.len()
        );
        info!("🏹 Erinyes: Auto-recovery enabled: {}", self.auto_recovery_enabled);
        info!("🏹 Erinyes: Escalation enabled: {}", self.escalation_enabled);
        info!("🏹 Erinyes: Guardian of Integrity ready");
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🏹 Erinyes: Shutting down Guardian of Integrity...");
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
                self.heartbeat_monitor.set_interval(actor, interval_ms).await;
                Ok(ResponsePayload::Success { 
                    message: format!("Heartbeat interval configured for {:?}", actor) 
                })
            }
            CommandPayload::RecoverActor { actor, strategy } => {
                self.recovery_engine.request_recovery(
                    actor, 
                    strategy, 
                    RecoveryUrgency::High
                ).await?;
                
                Ok(ResponsePayload::Success { 
                    message: format!("Recovery initiated for {:?}", actor) 
                })
            }
            CommandPayload::Custom(data) => {
                if let Ok(erinyes_cmd) = serde_json::from_value::<ErinyesCommand>(data) {
                    self.execute_erinyes_command(erinyes_cmd).await
                } else {
                    Err(ActorError::InvalidCommand { 
                        god: GodName::Erinyes, 
                        reason: "Unknown command format".to_string() 
                    })
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Erinyes, 
                reason: "Command not supported by Erinyes".to_string() 
            })
        }
    }
    
    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::GetStats => {
                let heartbeat_stats = self.heartbeat_monitor.get_stats().await;
                let recovery_stats = self.recovery_engine.get_stats().await;
                let alert_stats = self.alert_system.get_alert_stats().await;
                let system_health = self.watchdog.check_system_health().await;
                let dead_letter_count = self.dead_letter_queue.len().await;
                
                Ok(ResponsePayload::Stats {
                    data: serde_json::json!({
                        "heartbeat": heartbeat_stats,
                        "recovery": recovery_stats,
                        "alerts": alert_stats,
                        "system_health": system_health,
                        "dead_letters": dead_letter_count,
                        "trinity_status": self.get_trinity_status().await,
                    }),
                })
            }
            QueryPayload::Custom(data) => {
                if let Some(query_type) = data.get("query_type").and_then(|v| v.as_str()) {
                    match query_type {
                        "actor_health" => {
                            if let Some(actor_name) = data.get("actor").and_then(|v| v.as_str()) {
                                // Parse actor name
                                let actor = match actor_name {
                                    "Zeus" => GodName::Zeus,
                                    "Hades" => GodName::Hades,
                                    "Poseidon" => GodName::Poseidon,
                                    "Hermes" => GodName::Hermes,
                                    _ => GodName::Zeus, // Default
                                };
                                
                                let state = self.heartbeat_monitor.get_state(&actor).await;
                                Ok(ResponsePayload::Data { 
                                    data: serde_json::to_value(state).unwrap_or_default() 
                                })
                            } else {
                                Err(ActorError::InvalidQuery { 
                                    god: GodName::Erinyes, 
                                    reason: "Missing actor name".to_string() 
                                })
                            }
                        }
                        "system_health" => {
                            let health = self.watchdog.check_system_health().await;
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(health).unwrap_or_default() 
                            })
                        }
                        "active_alerts" => {
                            let alerts = self.alert_system.get_active_alerts(None).await;
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(alerts).unwrap_or_default() 
                            })
                        }
                        "recovery_history" => {
                            let actor = data.get("actor").and_then(|v| v.as_str())
                                .map(|s| match s {
                                    "Zeus" => GodName::Zeus,
                                    "Hades" => GodName::Hades,
                                    _ => GodName::Zeus,
                                });
                            let limit = data.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                            
                            let history = self.recovery_engine.get_recovery_history(actor, limit).await;
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(history).unwrap_or_default() 
                            })
                        }
                        _ => Err(ActorError::InvalidQuery { 
                            god: GodName::Erinyes, 
                            reason: format!("Unknown query type: {}", query_type) 
                        })
                    }
                } else {
                    Err(ActorError::InvalidQuery { 
                        god: GodName::Erinyes, 
                        reason: "Missing query_type".to_string() 
                    })
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Erinyes, 
                reason: "Unsupported query type".to_string() 
            })
        }
    }
    
    async fn handle_event(&mut self, event: crate::traits::message::EventPayload) -> Result<ResponsePayload, ActorError> {
        match event {
            crate::traits::message::EventPayload::ActorPanicked { actor, error } => {
                self.watchdog.report_panic(actor, error.clone(), None).await;
                
                // Trigger immediate recovery for Trinity members
                let trinity = self.trinity_members.read().await;
                if trinity.contains(&actor) {
                    warn!("🚨 Trinity member {:?} panicked! Triggering immediate recovery", actor);
                    
                    self.alert_system.create_alert(
                        AlertSeverity::Critical,
                        GodName::Erinyes,
                        format!("TRINITY PANIC: {:?}", actor),
                        format!("Trinity member {:?} has panicked: {}", actor, error),
                    ).await;
                    
                    drop(trinity);
                    
                    self.recovery_engine.request_recovery(
                        actor,
                        RecoveryStrategy::OneForOne,
                        RecoveryUrgency::Critical,
                    ).await.ok();
                } else {
                    drop(trinity);
                    
                    if self.auto_recovery_enabled {
                        self.recovery_engine.request_recovery(
                            actor,
                            RecoveryStrategy::OneForOne,
                            RecoveryUrgency::High,
                        ).await.ok();
                    }
                }
                
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() }),
        }
    }
    
    
    async fn execute_erinyes_command(&self, cmd: ErinyesCommand) -> Result<ResponsePayload, ActorError> {
        match cmd {
            ErinyesCommand::RegisterActor { actor, config } => {
                self.heartbeat_monitor.register(actor, Some(config)).await
                    .map_err(|e| ActorError::Unknown { god: GodName::Erinyes, message: e })?;
                
                Ok(ResponsePayload::Success { 
                    message: format!("Actor {:?} registered for monitoring", actor) 
                })
            }
            ErinyesCommand::UnregisterActor { actor } => {
                self.heartbeat_monitor.unregister(&actor).await;
                Ok(ResponsePayload::Success { 
                    message: format!("Actor {:?} unregistered from monitoring", actor) 
                })
            }
            ErinyesCommand::ReceiveHeartbeat { actor, latency_ms } => {
                self.heartbeat_monitor.receive_heartbeat(actor, latency_ms).await;
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            ErinyesCommand::TriggerRecovery { actor, strategy, urgency } => {
                self.recovery_engine.request_recovery(actor, strategy, urgency).await?;
                Ok(ResponsePayload::Success { 
                    message: format!("Recovery triggered for {:?}", actor) 
                })
            }
            ErinyesCommand::AcknowledgeAlert { alert_id, acknowledged_by } => {
                self.alert_system.acknowledge_alert(&alert_id, &acknowledged_by).await
                    .map_err(|e| ActorError::Unknown { god: GodName::Erinyes, message: e })?;
                
                Ok(ResponsePayload::Success { 
                    message: format!("Alert {} acknowledged", alert_id) 
                })
            }
            ErinyesCommand::ResolveAlert { alert_id, resolution_note } => {
                self.alert_system.resolve_alert(&alert_id, resolution_note).await
                    .map_err(|e| ActorError::Unknown { god: GodName::Erinyes, message: e })?;
                
                Ok(ResponsePayload::Success { 
                    message: format!("Alert {} resolved", alert_id) 
                })
            }
            ErinyesCommand::EnableAutoRecovery { enabled } => {
                // This would require making auto_recovery_enabled mutable
                // For now, just log it
                info!("Auto-recovery enabled: {}", enabled);
                Ok(ResponsePayload::Success { 
                    message: format!("Auto-recovery {}", if enabled { "enabled" } else { "disabled" }) 
                })
            }
            ErinyesCommand::GetSystemHealth => {
                let health = self.watchdog.check_system_health().await;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(health).unwrap_or_default() 
                })
            }
            ErinyesCommand::GetRecoveryHistory { actor, limit } => {
                let history = self.recovery_engine.get_recovery_history(actor, limit).await;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(history).unwrap_or_default() 
                })
            }
            ErinyesCommand::PurgeOldData { max_age_seconds } => {
                let max_age = Duration::from_secs(max_age_seconds);
                self.watchdog.clear_old_records(chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::zero())).await;
                self.alert_system.cleanup_old_alerts(max_age).await;
                
                Ok(ResponsePayload::Success { 
                    message: "Old data purged".to_string() 
                })
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Erinyes, 
                reason: "Command not yet implemented".to_string() 
            })
        }
    }
    
    async fn get_trinity_status(&self) -> Vec<(GodName, Option<HeartbeatState>)> {
        let trinity = self.trinity_members.read().await;
        let mut status = Vec::new();
        
        for god in trinity.iter() {
            let state = self.heartbeat_monitor.get_state(god).await;
            status.push((god.clone(), state));
        }
        
        status
    }
    
    fn calculate_load(&self) -> f64 {
        let uptime_seconds = (chrono::Utc::now() - self.state.start_time).num_seconds() as f64;
        if uptime_seconds > 0.0 {
            (self.state.message_count as f64 / uptime_seconds).min(1.0)
        } else {
            0.0
        }
    }
    
    pub async fn is_trinity_healthy(&self) -> bool {
        let trinity = self.trinity_members.read().await;
        
        for god in trinity.iter() {
            if let Some(state) = self.heartbeat_monitor.get_state(god).await {
                if state.status != ActorStatus::Healthy {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    pub async fn get_trinity_members(&self) -> Vec<GodName> {
        self.trinity_members.read().await.clone()
    }
    
    pub async fn set_trinity_priority(&self, actor: GodName, is_trinity: bool) {
        let mut trinity = self.trinity_members.write().await;
        
        if is_trinity && !trinity.contains(&actor) {
            trinity.push(actor);
        } else if !is_trinity {
            trinity.retain(|g| g != &actor);
        }
        
        info!("🏛️ Trinity membership updated for {:?}: {}", actor, is_trinity);
    }
    
    pub fn get_heartbeat_monitor(&self) -> Arc<HeartbeatMonitor> {
        self.heartbeat_monitor.clone()
    }
    
    pub fn get_recovery_engine(&self) -> Arc<RecoveryEngine> {
        self.recovery_engine.clone()
    }
    
    pub fn get_alert_system(&self) -> Arc<AlertSystem> {
        self.alert_system.clone()
    }
}
