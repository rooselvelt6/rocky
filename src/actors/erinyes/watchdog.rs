// src/actors/erinyes/watchdog.rs
// OLYMPUS v13 - Erinyes Watchdog
// Vigilancia continua del sistema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: WatchdogEventType,
    pub actor: Option<GodName>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchdogEventType {
    ActorDeath,
    ActorPanic,
    HighLoad,
    MemoryWarning,
    RecoveryStarted,
    RecoveryCompleted,
    RecoveryFailed,
    Escalation,
}

#[derive(Debug, Clone)]
pub struct Watchdog {
    events: Arc<RwLock<Vec<WatchdogEvent>>>,
    death_records: Arc<RwLock<HashMap<GodName, DeathRecord>>>,
    last_activity: Arc<RwLock<HashMap<GodName, Instant>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathRecord {
    pub actor: GodName,
    pub timestamp: Instant,
    pub reason: String,
    pub recovery_attempts: u32,
}

impl Watchdog {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            death_records: Arc::new(RwLock::new(HashMap::new())),
            last_activity: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn report_death(&self, actor: GodName, reason: String) {
        let mut records = self.death_records.write().await;
        records.insert(actor, DeathRecord {
            actor,
            timestamp: Instant::now(),
            reason,
            recovery_attempts: 0,
        });
        
        let mut events = self.events.write().await;
        events.push(WatchdogEvent {
            timestamp: chrono::Utc::now(),
            event_type: WatchdogEventType::ActorDeath,
            actor: Some(actor),
            message: reason,
        });
    }
    
    pub async fn report_panic(&self, actor: &GodName, error: &str) {
        let mut events = self.events.write().await;
        events.push(WatchdogEvent {
            timestamp: chrono::Utc::now(),
            event_type: WatchdogEventType::ActorPanic,
            actor: Some(*actor),
            message: error.to_string(),
        });
    }
    
    pub async fn report_activity(&self, actor: GodName) {
        let mut activity = self.last_activity.write().await;
        activity.insert(actor, Instant::now());
    }
    
    pub async fn get_recent_events(&self, limit: usize) -> Vec<WatchdogEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }
    
    pub async fn get_death_count(&self) -> usize {
        self.death_records.read().await.len()
    }
    
    pub async fn get_death_records(&self) -> Vec<DeathRecord> {
        self.death_records.read().await.values().cloned().collect()
    }
}
