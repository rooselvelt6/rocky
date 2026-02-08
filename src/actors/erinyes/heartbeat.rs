// src/actors/erinyes/heartbeat.rs
// OLYMPUS v15 - Erinyes Heartbeat Monitor
// Monitoreo avanzado cada 500ms de todos los dioses con análisis de tendencias

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

use super::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::traits::actor_trait::ActorStatus;
use crate::actors::erinyes::alerts::{AlertSeverity, AlertSystem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub interval_ms: u64,
    pub timeout_ms: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            timeout_ms: 1500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatState {
    pub actor: GodName,
    pub last_seen: Instant,
    pub status: ActorStatus,
    pub consecutive_misses: u32,
    pub consecutive_successes: u32,
    pub config: HeartbeatConfig,
    pub history: VecDeque<HeartbeatRecord>,
    pub total_heartbeats: u64,
    pub missed_heartbeats: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRecord {
    pub timestamp: Instant,
    pub latency_ms: u64,
    pub status: ActorStatus,
}

impl HeartbeatState {
    pub fn new(actor: GodName, config: HeartbeatConfig) -> Self {
        Self {
            actor,
            last_seen: Instant::now(),
            status: ActorStatus::Healthy,
            consecutive_misses: 0,
            consecutive_successes: 0,
            config,
            history: VecDeque::with_capacity(100),
            total_heartbeats: 0,
            missed_heartbeats: 0,
        }
    }
    
    pub fn is_timed_out(&self) -> bool {
        self.last_seen.elapsed().as_millis() as u64 > self.config.timeout_ms
    }
    
    pub fn get_timeout_remaining_ms(&self) -> u64 {
        let elapsed = self.last_seen.elapsed().as_millis() as u64;
        if elapsed >= self.config.timeout_ms {
            0
        } else {
            self.config.timeout_ms - elapsed
        }
    }
    
    pub fn mark_missed(&mut self) {
        self.consecutive_misses += 1;
        self.consecutive_successes = 0;
        self.missed_heartbeats += 1;
        
        // Update status based on consecutive misses
        if self.consecutive_misses >= 5 {
            self.status = ActorStatus::Dead;
        } else if self.consecutive_misses >= 3 {
            self.status = ActorStatus::Critical;
        } else if self.consecutive_misses >= 1 {
            self.status = ActorStatus::Unhealthy;
        }
        
        // Add to history
        self.add_history_record(ActorStatus::Unhealthy, 0);
    }
    
    pub fn mark_received(&mut self, latency_ms: u64) {
        self.last_seen = Instant::now();
        self.consecutive_misses = 0;
        self.consecutive_successes += 1;
        self.total_heartbeats += 1;
        
        // Update status
        if self.consecutive_successes >= 3 {
            self.status = ActorStatus::Healthy;
        } else {
            self.status = ActorStatus::Recovering;
        }
        
        // Add to history
        self.add_history_record(self.status.clone(), latency_ms);
    }
    
    fn add_history_record(&mut self, status: ActorStatus, latency_ms: u64) {
        let record = HeartbeatRecord {
            timestamp: Instant::now(),
            latency_ms,
            status,
        };
        
        self.history.push_back(record);
        
        // Keep only last 100 records
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }
    
    pub fn get_average_latency_ms(&self) -> u64 {
        if self.history.is_empty() {
            return 0;
        }
        
        let total: u64 = self.history.iter().map(|r| r.latency_ms).sum();
        total / self.history.len() as u64
    }
    
    pub fn get_health_score(&self) -> f64 {
        if self.total_heartbeats == 0 {
            return 0.0;
        }
        
        let success_rate = (self.total_heartbeats - self.missed_heartbeats) as f64 
            / self.total_heartbeats as f64;
        
        // Latency factor (lower is better)
        let avg_latency = self.get_average_latency_ms();
        let latency_factor = if avg_latency > 0 {
            (100.0 / (avg_latency as f64 + 100.0)).min(1.0)
        } else {
            1.0
        };
        
        (success_rate * 0.7 + latency_factor * 0.3) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    actors: Arc<RwLock<HashMap<GodName, HeartbeatState>>>,
    default_config: HeartbeatConfig,
    alert_system: Arc<AlertSystem>,
}

impl HeartbeatMonitor {
    pub fn new(alert_system: Arc<AlertSystem>) -> Self {
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            default_config: HeartbeatConfig::default(),
            alert_system,
        }
    }
    
    pub async fn register(&self, actor: GodName, config: Option<HeartbeatConfig>) -> Result<(), String> {
        let mut actors = self.actors.write().await;
        
        if actors.contains_key(&actor) {
            return Err(format!("Actor {:?} already registered", actor));
        }
        
        let cfg = config.unwrap_or_else(|| self.default_config.clone());
        actors.insert(actor.clone(), HeartbeatState::new(actor.clone(), cfg));
        
        info!("🫀 Heartbeat monitor registered for {:?}", actor);
        Ok(())
    }
    
    pub async fn unregister(&self, actor: &GodName) {
        let mut actors = self.actors.write().await;
        actors.remove(actor);
        info!("🫀 Heartbeat monitor unregistered for {:?}", actor);
    }
    
    pub async fn receive_heartbeat(&self, actor: GodName, latency_ms: Option<u64>) {
        let mut actors = self.actors.write().await;
        
        if let Some(state) = actors.get_mut(&actor) {
            state.mark_received(latency_ms.unwrap_or(0));
            debug!("💓 Heartbeat received from {:?}", actor);
        } else {
            // Auto-register if not found
            drop(actors);
            let _ = self.register(actor.clone(), None).await;
            
            // Mark heartbeat
            let mut actors = self.actors.write().await;
            if let Some(state) = actors.get_mut(&actor) {
                state.mark_received(latency_ms.unwrap_or(0));
            }
        }
    }
    
    pub async fn check_all(&self, on_missed: impl Fn(GodName, &HeartbeatState)) {
        let actors = self.actors.read().await;
        let timed_out: Vec<(GodName, HeartbeatState)> = actors
            .iter()
            .filter(|(_, state)| state.is_timed_out())
            .map(|(actor, state)| (actor.clone(), state.clone()))
            .collect();
        drop(actors);
        
        for (actor, _) in timed_out {
            let mut actors = self.actors.write().await;
            if let Some(state) = actors.get_mut(&actor) {
                let previous_status = state.status.clone();
                state.mark_missed();
                
                warn!(
                    "💔 Heartbeat missed for {:?} (consecutive: {})",
                    actor, state.consecutive_misses
                );
                
                // Trigger callback
                let state_clone = state.clone();
                drop(actors);
                on_missed(actor.clone(), &state_clone);
                
                // Create alert if status degraded
                if state_clone.status == ActorStatus::Critical && previous_status != ActorStatus::Critical {
                    self.alert_system.create_alert(
                        AlertSeverity::Critical,
                        GodName::Erinyes,
                        format!("Actor {:?} is CRITICAL", actor),
                        format!(
                            "Actor {:?} has missed {} consecutive heartbeats. Status: {:?}",
                            actor, state_clone.consecutive_misses, state_clone.status
                        ),
                    ).await;
                }
            }
        }
    }
    
    pub async fn set_interval(&self, actor: GodName, interval_ms: u64) {
        let mut actors = self.actors.write().await;
        if let Some(state) = actors.get_mut(&actor) {
            state.config.interval_ms = interval_ms;
            state.config.timeout_ms = interval_ms + 100; // Timeout = interval + buffer
            info!("🫀 Heartbeat interval updated for {:?}: {}ms", actor, interval_ms);
        }
    }
    
    pub async fn set_timeout(&self, actor: GodName, timeout_ms: u64) {
        let mut actors = self.actors.write().await;
        if let Some(state) = actors.get_mut(&actor) {
            state.config.timeout_ms = timeout_ms;
            info!("🫀 Heartbeat timeout updated for {:?}: {}ms", actor, timeout_ms);
        }
    }
    
    pub async fn get_state(&self, actor: &GodName) -> Option<HeartbeatState> {
        let actors = self.actors.read().await;
        actors.get(actor).cloned()
    }
    
    pub async fn monitored_count(&self) -> usize {
        let actors = self.actors.read().await;
        actors.len()
    }
    
    pub async fn get_all_states(&self) -> HashMap<GodName, HeartbeatState> {
        let actors = self.actors.read().await;
        actors.clone()
    }
    
    pub async fn get_stats(&self) -> HeartbeatStats {
        let actors = self.actors.read().await;
        let mut healthy = 0;
        let mut recovering = 0;
        let mut unhealthy = 0;
        let mut critical = 0;
        let mut dead = 0;
        let mut total_health_score = 0.0;
        let mut total_latency = 0u64;
        let mut count_with_latency = 0;
        
        for (_, state) in actors.iter() {
            match state.status {
                ActorStatus::Healthy => healthy += 1,
                ActorStatus::Recovering => recovering += 1,
                ActorStatus::Unhealthy => unhealthy += 1,
                ActorStatus::Critical => critical += 1,
                ActorStatus::Dead => dead += 1,
            }
            
            total_health_score += state.get_health_score();
            let latency = state.get_average_latency_ms();
            if latency > 0 {
                total_latency += latency;
                count_with_latency += 1;
            }
        }
        
        let avg_health_score = if actors.is_empty() {
            0.0
        } else {
            total_health_score / actors.len() as f64
        };
        
        let avg_latency = if count_with_latency > 0 {
            total_latency / count_with_latency as u64
        } else {
            0
        };
        
        HeartbeatStats {
            total: actors.len(),
            healthy,
            recovering,
            unhealthy,
            critical,
            dead,
            avg_health_score,
            avg_latency_ms: avg_latency,
        }
    }
    
    pub async fn get_unhealthy_actors(&self) -> Vec<(GodName, HeartbeatState)> {
        let actors = self.actors.read().await;
        actors
            .iter()
            .filter(|(_, state)| state.status != ActorStatus::Healthy)
            .map(|(actor, state)| (actor.clone(), state.clone()))
            .collect()
    }
    
    pub async fn is_healthy(&self, actor: &GodName) -> bool {
        let actors = self.actors.read().await;
        actors
            .get(actor)
            .map(|state| state.status == ActorStatus::Healthy)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatStats {
    pub total: usize,
    pub healthy: usize,
    pub recovering: usize,
    pub unhealthy: usize,
    pub critical: usize,
    pub dead: usize,
    pub avg_health_score: f64,
    pub avg_latency_ms: u64,
}

impl HeartbeatStats {
    pub fn healthy_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.healthy as f64 / self.total as f64) * 100.0
        }
    }
    
    pub fn degraded_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            ((self.unhealthy + self.critical + self.recovering) as f64 / self.total as f64) * 100.0
        }
    }
}
