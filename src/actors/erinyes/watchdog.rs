// src/actors/erinyes/watchdog.rs
// OLYMPUS v15 - Erinyes Watchdog
// Vigilancia continua del sistema con análisis predictivo

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{warn, error, debug};

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: WatchdogEventType,
    pub actor: Option<GodName>,
    pub message: String,
    pub severity: WatchdogSeverity,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchdogEventType {
    ActorDeath,
    ActorPanic,
    HighLoad,
    MemoryWarning,
    RecoveryStarted,
    RecoveryCompleted,
    RecoveryFailed,
    Escalation,
    CircuitBreakerOpened,
    CircuitBreakerClosed,
    HealthCheckFailed,
    SystemDegraded,
    SystemRecovered,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchdogSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for WatchdogSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for WatchdogEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathRecord {
    pub actor: GodName,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    pub recovery_attempts: u32,
    pub was_recovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorActivity {
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub message_count: u64,
    pub error_count: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Clone)]
pub struct Watchdog {
    events: Arc<RwLock<Vec<WatchdogEvent>>>,
    death_records: Arc<RwLock<HashMap<GodName, DeathRecord>>>,
    activity_log: Arc<RwLock<HashMap<GodName, VecDeque<ActorActivity>>>>,
    custom_handlers: Arc<RwLock<HashMap<WatchdogEventType, Vec<Box<dyn Fn(&WatchdogEvent) + Send + Sync>>>>>,
}

impl std::fmt::Debug for Watchdog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watchdog")
            .field("events", &"Arc<RwLock<Vec<WatchdogEvent>>>")
            .field("death_records", &"Arc<RwLock<HashMap<GodName, DeathRecord>>>")
            .field("activity_log", &"Arc<RwLock<HashMap<GodName, VecDeque<ActorActivity>>>>")
            .finish()
    }
}

impl Watchdog {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            death_records: Arc::new(RwLock::new(HashMap::new())),
            activity_log: Arc::new(RwLock::new(HashMap::new())),
            custom_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn report_event(
        &self,
        event_type: WatchdogEventType,
        actor: Option<GodName>,
        message: String,
        severity: WatchdogSeverity,
        metadata: Option<serde_json::Value>,
    ) {
        let event = WatchdogEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: event_type.clone(),
            actor,
            message: message.clone(),
            severity: severity.clone(),
            metadata: metadata.unwrap_or_default(),
        };
        
        // Store event
        let mut events = self.events.write().await;
        events.push(event.clone());
        
        // Keep only last 10000 events
        if events.len() > 10000 {
            events.remove(0);
        }
        drop(events);
        
        // Log based on severity
        match severity {
            WatchdogSeverity::Info => debug!("🐕 Watchdog: {:?} - {}", event_type, message),
            WatchdogSeverity::Warning => warn!("🐕 Watchdog: {:?} - {}", event_type, message),
            WatchdogSeverity::Error => error!("🐕 Watchdog: {:?} - {}", event_type, message),
            WatchdogSeverity::Critical => {
                error!("🚨 Watchdog CRITICAL: {:?} - {}", event_type, message);
            }
        }
        
        // Trigger custom handlers
        let handlers = self.custom_handlers.read().await;
        if let Some(handlers_list) = handlers.get(&event_type) {
            for handler in handlers_list {
                handler(&event);
            }
        }
    }
    
    pub async fn report_death(&self, actor: GodName, reason: String) {
        let _now = Instant::now();
        let mut records = self.death_records.write().await;
        
        let record = DeathRecord {
            actor: actor.clone(),
            timestamp: chrono::Utc::now(),
            detected_at: chrono::Utc::now(),
            reason: reason.clone(),
            recovery_attempts: 0,
            was_recovered: false,
        };
        
        records.insert(actor.clone(), record);
        drop(records);
        
        self.report_event(
            WatchdogEventType::ActorDeath,
            Some(actor),
            reason,
            WatchdogSeverity::Critical,
            None,
        ).await;
    }
    
    pub async fn report_panic(&self, actor: GodName, error: String, backtrace: Option<String>) {
        let metadata = serde_json::json!({
            "backtrace": backtrace,
            "error_type": "panic",
        });
        
        self.report_event(
            WatchdogEventType::ActorPanic,
            Some(actor),
            error,
            WatchdogSeverity::Critical,
            Some(metadata),
        ).await;
    }
    
    pub async fn mark_recovered(&self, actor: &GodName) {
        let mut records = self.death_records.write().await;
        
        if let Some(record) = records.get_mut(actor) {
            record.was_recovered = true;
            record.recovery_attempts += 1;
        }
        
        drop(records);
        
        self.report_event(
            WatchdogEventType::SystemRecovered,
            Some(actor.clone()),
            format!("Actor {:?} has recovered", actor),
            WatchdogSeverity::Info,
            None,
        ).await;
    }
    
    pub async fn report_activity(
        &self,
        actor: GodName,
        message_count: u64,
        error_count: u64,
        memory_usage_mb: f64,
        cpu_usage_percent: f64,
    ) {
        let activity = ActorActivity {
            last_seen: chrono::Utc::now(),
            message_count,
            error_count,
            memory_usage_mb,
            cpu_usage_percent,
        };
        
        let mut log = self.activity_log.write().await;
        let actor_log = log.entry(actor).or_insert_with(|| VecDeque::with_capacity(100));
        
        actor_log.push_back(activity);
        
        // Keep only last 100 entries
        if actor_log.len() > 100 {
            actor_log.pop_front();
        }
    }
    
    pub async fn check_system_health(&self) -> SystemHealth {
        let records = self.death_records.read().await;
        let events = self.events.read().await;
        let log = self.activity_log.read().await;
        
        let active_deaths = records.values().filter(|r| !r.was_recovered).count();
        let recent_errors = events
            .iter()
            .rev()
            .take(100)
            .filter(|e| e.severity == WatchdogSeverity::Error || e.severity == WatchdogSeverity::Critical)
            .count();
        
        let mut high_memory_actors = Vec::new();
        for (actor, activities) in log.iter() {
            if let Some(latest) = activities.back() {
                if latest.memory_usage_mb > 1000.0 { // > 1GB
                    high_memory_actors.push((actor.clone(), latest.memory_usage_mb));
                }
            }
        }
        
        let status = if active_deaths > 0 || recent_errors > 10 {
            SystemStatus::Critical
        } else if recent_errors > 5 {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };
        
        SystemHealth {
            status,
            active_deaths,
            recent_errors,
            high_memory_actors,
            total_events_24h: events.len(), // Simplified, should filter by time
        }
    }
    
    pub async fn get_recent_events(&self, limit: usize, event_type: Option<WatchdogEventType>) -> Vec<WatchdogEvent> {
        let events = self.events.read().await;
        
        let filtered: Vec<_> = match event_type {
            Some(et) => events.iter().filter(|e| e.event_type == et).cloned().collect(),
            None => events.clone(),
        };
        
        filtered.into_iter().rev().take(limit).collect()
    }
    
    pub async fn get_death_count(&self) -> usize {
        let records = self.death_records.read().await;
        records.len()
    }
    
    pub async fn get_active_deaths(&self) -> Vec<DeathRecord> {
        let records = self.death_records.read().await;
        records.values().filter(|r| !r.was_recovered).cloned().collect()
    }
    
    pub async fn get_death_records(&self, recovered_only: bool) -> Vec<DeathRecord> {
        let records = self.death_records.read().await;
        
        if recovered_only {
            records.values().filter(|r| r.was_recovered).cloned().collect()
        } else {
            records.values().cloned().collect()
        }
    }
    
    pub async fn get_activity_trend(&self, actor: &GodName, duration: Duration) -> Option<ActivityTrend> {
        let log = self.activity_log.read().await;
        
        if let Some(actor_log) = log.get(actor) {
            let cutoff = chrono::Utc::now() - chrono::Duration::from_std(duration).unwrap_or(chrono::Duration::zero());
            let recent: Vec<_> = actor_log.iter().filter(|a| a.last_seen > cutoff).collect();
            
            if recent.len() < 2 {
                return None;
            }
            
            let first = recent.first().unwrap();
            let last = recent.last().unwrap();
            
            let message_rate = if recent.len() > 1 {
                let time_span = (last.last_seen - first.last_seen).num_seconds() as f64;
                if time_span > 0.0 {
                    (last.message_count - first.message_count) as f64 / time_span
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            let error_rate = if recent.len() > 1 {
                let time_span = (last.last_seen - first.last_seen).num_seconds() as f64;
                if time_span > 0.0 {
                    (last.error_count - first.error_count) as f64 / time_span
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            let avg_memory = recent.iter().map(|a| a.memory_usage_mb).sum::<f64>() / recent.len() as f64;
            let avg_cpu = recent.iter().map(|a| a.cpu_usage_percent).sum::<f64>() / recent.len() as f64;
            
            Some(ActivityTrend {
                actor: actor.clone(),
                message_rate,
                error_rate,
                avg_memory_mb: avg_memory,
                avg_cpu_percent: avg_cpu,
                sample_count: recent.len(),
            })
        } else {
            None
        }
    }
    
    pub async fn clear_old_records(&self, max_age: chrono::Duration) {
        let mut records = self.death_records.write().await;
        let now = chrono::Utc::now();
        
        records.retain(|_, record| {
            now - record.timestamp < max_age || !record.was_recovered
        });
    }
    
    pub async fn register_event_handler<F>(&self, event_type: WatchdogEventType, handler: F)
    where
        F: Fn(&WatchdogEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.custom_handlers.write().await;
        handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }
    
    pub async fn export_events(&self, format: ExportFormat) -> String {
        let events = self.events.read().await;
        
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&*events).unwrap_or_default()
            }
            ExportFormat::Csv => {
                let mut csv = String::from("id,timestamp,event_type,actor,message,severity\n");
                for event in events.iter() {
                    csv.push_str(&format!(
                        "{},{},{:?},{:?},{},{}\n",
                        event.id,
                        event.timestamp,
                        event.event_type,
                        event.actor,
                        event.message.replace(',', ";"),
                        event.severity
                    ));
                }
                csv
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: SystemStatus,
    pub active_deaths: usize,
    pub recent_errors: usize,
    pub high_memory_actors: Vec<(GodName, f64)>,
    pub total_events_24h: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTrend {
    pub actor: GodName,
    pub message_rate: f64,      // messages per second
    pub error_rate: f64,        // errors per second
    pub avg_memory_mb: f64,
    pub avg_cpu_percent: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}
