// src/actors/erinyes/recovery.rs
// OLYMPUS v13 - Erinyes Recovery Engine
// Motor de recuperación con max 3 restarts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

use super::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::errors::ActorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecord {
    pub actor: GodName,
    pub attempt: u32,
    pub strategy: RecoveryStrategy,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub success: bool,
    pub error: Option<String>,
}

impl RecoveryRecord {
    pub fn new(actor: GodName, attempt: u32, strategy: RecoveryStrategy) -> Self {
        Self {
            actor,
            attempt,
            strategy,
            started_at: Instant::now(),
            completed_at: None,
            success: false,
            error: None,
        }
    }
    
    pub fn complete(&mut self, success: bool, error: Option<String>) {
        self.completed_at = Some(Instant::now());
        self.success = success;
        self.error = error;
    }
    
    pub fn duration_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryEngine {
    records: Arc<RwLock<Vec<RecoveryRecord>>>,
    restart_counters: Arc<RwLock<HashMap<GodName, (u32, Instant)>>>,
    max_restarts: u32,
    restart_window_seconds: u64,
}

impl RecoveryEngine {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            restart_counters: Arc::new(RwLock::new(HashMap::new())),
            max_restarts: 3,
            restart_window_seconds: 30,
        }
    }
    
    pub async fn trigger_recovery(&self, actor: GodName, strategy: RecoveryStrategy) {
        let mut counters = self.restart_counters.write().await;
        let now = Instant::now();
        
        let (count, first_attempt) = counters.get(&actor).cloned().unwrap_or((0, now));
        
        // Check if within restart window
        if first_attempt.elapsed().as_secs() > self.restart_window_seconds {
            counters.insert(actor, (1, now));
        } else if count >= self.max_restarts {
            // Escalate to Zeus
            return;
        } else {
            counters.insert(actor, (count + 1, first_attempt));
        }
        
        let mut record = RecoveryRecord::new(actor, count + 1, strategy);
        
        // Perform recovery (in real implementation, would call actor.restart())
        let result = self.perform_recovery(actor.clone()).await;
        
        record.complete(result.is_ok(), result.err().map(|e| e.to_string()));
        
        let mut records = self.records.write().await;
        records.push(record);
    }
    
    async fn perform_recovery(&self, actor: GodName) -> Result<(), ActorError> {
        // In real implementation, would:
        // 1. Call actor.shutdown()
        // 2. Recreate actor instance
        // 3. Call actor.initialize()
        // 4. Update supervision tree
        Ok(())
    }
    
    pub async fn get_recovery_history(&self, actor: Option<GodName>) -> Vec<RecoveryRecord> {
        let records = self.records.read().await;
        match actor {
            Some(g) => records.iter().filter(|r| r.actor == g).cloned().collect(),
            None => records.clone(),
        }
    }
    
    pub async fn should_escalate(&self, actor: GodName) -> bool {
        let counters = self.restart_counters.read().await;
        if let Some((count, first_attempt)) = counters.get(&actor) {
            if *count >= self.max_restarts && first_attempt.elapsed().as_secs() <= self.restart_window_seconds {
                return true;
            }
        }
        false
    }
}
