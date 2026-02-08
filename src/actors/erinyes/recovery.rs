// src/actors/erinyes/recovery.rs
// OLYMPUS v15 - Erinyes Recovery Engine
// Motor de recuperación avanzado con múltiples estrategias

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn, error, debug};

use super::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::errors::ActorError;
use crate::actors::erinyes::alerts::{AlertSeverity, AlertSystem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecord {
    pub id: String,
    pub actor: GodName,
    pub attempt: u32,
    pub strategy: RecoveryStrategy,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub metadata: serde_json::Value,
}

impl RecoveryRecord {
    pub fn new(actor: GodName, attempt: u32, strategy: RecoveryStrategy) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            actor,
            attempt,
            strategy,
            started_at: chrono::Utc::now(),
            completed_at: None,
            success: false,
            error: None,
            duration_ms: 0,
            metadata: serde_json::json!({}),
        }
    }
    
    pub fn complete(&mut self, success: bool, error: Option<String>, duration_ms: u64) {
        self.completed_at = Some(chrono::Utc::now());
        self.success = success;
        self.error = error;
        self.duration_ms = duration_ms;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub max_restarts: u32,
    pub restart_window_seconds: u64,
    pub backoff_base_ms: u64,
    pub backoff_max_ms: u64,
    pub backoff_multiplier: f64,
    pub enable_circuit_breaker: bool,
    pub circuit_breaker_threshold: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_window_seconds: 60,
            backoff_base_ms: 1000,
            backoff_max_ms: 30000,
            backoff_multiplier: 2.0,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RestartWindow {
    pub count: u32,
    pub first_attempt: Instant,
    pub last_attempt: Instant,
}

#[derive(Debug, Clone)]
pub struct RecoveryEngine {
    records: Arc<RwLock<Vec<RecoveryRecord>>>,
    restart_counters: Arc<RwLock<HashMap<GodName, RestartWindow>>>,
    config: RecoveryConfig,
    circuit_breaker: Arc<RwLock<HashMap<GodName, bool>>>,
    alert_system: Arc<AlertSystem>,
    recovery_tx: mpsc::Sender<RecoveryRequest>,
    recovery_rx: Arc<RwLock<mpsc::Receiver<RecoveryRequest>>>,
}

#[derive(Debug, Clone)]
pub struct RecoveryRequest {
    pub actor: GodName,
    pub strategy: RecoveryStrategy,
    pub urgency: RecoveryUrgency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryUrgency {
    Low,
    Medium,
    High,
    Critical,
}

impl RecoveryEngine {
    pub fn new(alert_system: Arc<AlertSystem>) -> Self {
        let (recovery_tx, recovery_rx) = mpsc::channel(100);
        
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            restart_counters: Arc::new(RwLock::new(HashMap::new())),
            config: RecoveryConfig::default(),
            circuit_breaker: Arc::new(RwLock::new(HashMap::new())),
            alert_system,
            recovery_tx,
            recovery_rx: Arc::new(RwLock::new(recovery_rx)),
        }
    }
    
    pub fn with_config(alert_system: Arc<AlertSystem>, config: RecoveryConfig) -> Self {
        let (recovery_tx, recovery_rx) = mpsc::channel(100);
        
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            restart_counters: Arc::new(RwLock::new(HashMap::new())),
            config,
            circuit_breaker: Arc::new(RwLock::new(HashMap::new())),
            alert_system,
            recovery_tx,
            recovery_rx: Arc::new(RwLock::new(recovery_rx)),
        }
    }
    
    pub async fn start_recovery_worker(&self, recovery_fn: impl Fn(GodName) -> futures::future::BoxFuture<'static, Result<(), ActorError>> + Send + Sync + 'static) {
        let rx = self.recovery_rx.clone();
        let counters = self.restart_counters.clone();
        let records = self.records.clone();
        let circuit_breaker = self.circuit_breaker.clone();
        let alert_system = self.alert_system.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut rx = rx.write().await;
            
            while let Some(request) = rx.recv().await {
                let actor = request.actor;
                let strategy = request.strategy;
                
                // Check circuit breaker
                if config.enable_circuit_breaker {
                    let cb = circuit_breaker.read().await;
                    if cb.get(&actor).copied().unwrap_or(false) {
                        warn!("Circuit breaker open for {:?}, skipping recovery", actor);
                        continue;
                    }
                }
                
                // Perform recovery
                let start = Instant::now();
                let mut record = RecoveryRecord::new(actor.clone(), 0, strategy.clone());
                
                info!("🔄 Starting recovery for {:?} with strategy {:?}", actor, strategy);
                
                match recovery_fn(actor.clone()).await {
                    Ok(()) => {
                        let duration = start.elapsed().as_millis() as u64;
                        record.complete(true, None, duration);
                        
                        info!("✅ Recovery succeeded for {:?} in {}ms", actor, duration);
                        
                        // Reset restart counter
                        let mut counters = counters.write().await;
                        counters.remove(&actor);
                        
                        // Close circuit breaker if open
                        let mut cb = circuit_breaker.write().await;
                        cb.insert(actor.clone(), false);
                    }
                    Err(e) => {
                        let duration = start.elapsed().as_millis() as u64;
                        let error_msg = e.to_string();
                        record.complete(false, Some(error_msg.clone()), duration);
                        
                        warn!("❌ Recovery failed for {:?}: {}", actor, error_msg);
                        
                        // Update restart counter
                        let mut counters = counters.write().await;
                        let now = Instant::now();
                        
                        let window = counters.entry(actor.clone()).or_insert(RestartWindow {
                            count: 0,
                            first_attempt: now,
                            last_attempt: now,
                        });
                        
                        window.count += 1;
                        window.last_attempt = now;
                        
                        // Check if we should escalate
                        if window.count >= config.circuit_breaker_threshold {
                            let mut cb = circuit_breaker.write().await;
                            cb.insert(actor.clone(), true);
                            
                            alert_system.create_alert(
                                AlertSeverity::Critical,
                                GodName::Erinyes,
                                format!("Circuit breaker opened for {:?}", actor),
                                format!(
                                    "Actor {:?} has failed recovery {} times. Circuit breaker is now open.",
                                    actor, window.count
                                ),
                            ).await;
                        }
                    }
                }
                
                // Store record
                let mut records = records.write().await;
                records.push(record);
                
                // Keep only last 1000 records
                if records.len() > 1000 {
                    records.remove(0);
                }
            }
        });
    }
    
    pub async fn request_recovery(&self, actor: GodName, strategy: RecoveryStrategy, urgency: RecoveryUrgency) -> Result<(), ActorError> {
        // Check if should escalate to Zeus
        if self.should_escalate(&actor).await {
            return Err(ActorError::RecoveryFailed {
                god: actor,
                message: "Max restarts exceeded, escalation required".to_string(),
                attempts: self.get_restart_count(&actor).await,
            });
        }
        
        let request = RecoveryRequest { actor, strategy, urgency };
        
        self.recovery_tx
            .send(request)
            .await
            .map_err(|_| ActorError::Unknown { 
                god: GodName::Erinyes, 
                message: "Failed to queue recovery request".to_string() 
            })?;
        
        Ok(())
    }
    
    pub async fn trigger_recovery(&self, actor: GodName, strategy: RecoveryStrategy) {
        let _ = self.request_recovery(actor, strategy, RecoveryUrgency::High).await;
    }
    
    pub async fn get_restart_count(&self, actor: &GodName) -> u32 {
        let counters = self.restart_counters.read().await;
        counters.get(actor).map(|w| w.count).unwrap_or(0)
    }
    
    pub async fn should_escalate(&self, actor: &GodName) -> bool {
        let counters = self.restart_counters.read().await;
        
        if let Some(window) = counters.get(actor) {
            if window.count >= self.config.max_restarts {
                let elapsed = window.first_attempt.elapsed().as_secs();
                if elapsed <= self.config.restart_window_seconds {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub async fn reset_restart_counter(&self, actor: &GodName) {
        let mut counters = self.restart_counters.write().await;
        counters.remove(actor);
    }
    
    pub async fn is_circuit_breaker_open(&self, actor: &GodName) -> bool {
        if !self.config.enable_circuit_breaker {
            return false;
        }
        
        let cb = self.circuit_breaker.read().await;
        cb.get(actor).copied().unwrap_or(false)
    }
    
    pub async fn close_circuit_breaker(&self, actor: &GodName) {
        let mut cb = self.circuit_breaker.write().await;
        cb.insert(actor.clone(), false);
        
        // Also reset restart counter
        self.reset_restart_counter(actor).await;
        
        info!("🔓 Circuit breaker manually closed for {:?}", actor);
    }
    
    pub async fn get_recovery_history(&self, actor: Option<GodName>, limit: usize) -> Vec<RecoveryRecord> {
        let records = self.records.read().await;
        let filtered: Vec<RecoveryRecord> = match actor {
            Some(g) => records.iter().filter(|r| r.actor == g).cloned().collect(),
            None => records.clone(),
        };
        
        filtered.into_iter().rev().take(limit).collect()
    }
    
    pub async fn get_success_rate(&self, actor: Option<GodName>) -> f64 {
        let records = self.records.read().await;
        
        let relevant: Vec<_> = match actor {
            Some(g) => records.iter().filter(|r| r.actor == g).collect(),
            None => records.iter().collect(),
        };
        
        if relevant.is_empty() {
            return 0.0;
        }
        
        let successful = relevant.iter().filter(|r| r.success).count();
        (successful as f64 / relevant.len() as f64) * 100.0
    }
    
    pub async fn get_stats(&self) -> RecoveryStats {
        let records = self.records.read().await;
        let counters = self.restart_counters.read().await;
        
        let total = records.len();
        let successful = records.iter().filter(|r| r.success).count();
        let failed = total - successful;
        let avg_duration = if total > 0 {
            records.iter().map(|r| r.duration_ms).sum::<u64>() / total as u64
        } else {
            0
        };
        
        RecoveryStats {
            total_attempts: total,
            successful,
            failed,
            success_rate: if total > 0 { (successful as f64 / total as f64) * 100.0 } else { 0.0 },
            avg_recovery_duration_ms: avg_duration,
            actors_in_recovery: counters.len(),
            circuit_breakers_open: counters.iter().filter(|(_, w)| w.count >= self.config.circuit_breaker_threshold).count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub total_attempts: usize,
    pub successful: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub avg_recovery_duration_ms: u64,
    pub actors_in_recovery: usize,
    pub circuit_breakers_open: usize,
}

use futures::future::BoxFuture;
