// src/actors/erinyes/heartbeat.rs
// OLYMPUS v13 - Erinyes Heartbeat Monitor
// Monitoreo cada 500ms de todos los dioses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

use super::GodName;
use super::HeartbeatConfig;
use crate::traits::message::RecoveryStrategy;
use crate::traits::actor_trait::ActorStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatState {
    pub actor: GodName,
    pub last_seen: Instant,
    pub status: ActorStatus,
    pub consecutive_misses: u32,
    pub config: HeartbeatConfig,
}

impl HeartbeatState {
    pub fn new(actor: GodName, config: HeartbeatConfig) -> Self {
        Self {
            actor,
            last_seen: Instant::now(),
            status: ActorStatus::Healthy,
            consecutive_misses: 0,
            config,
        }
    }
    
    pub fn is_timed_out(&self) -> bool {
        self.last_seen.elapsed().as_millis() as u64 > self.config.timeout_ms
    }
    
    pub fn mark_missed(&mut self) {
        self.consecutive_misses += 1;
        if self.consecutive_misses >= 3 {
            self.status = ActorStatus::Dead;
        } else if self.consecutive_misses >= 1 {
            self.status = ActorStatus::Unhealthy;
        }
    }
    
    pub fn mark_received(&mut self) {
        self.last_seen = Instant::now();
        self.consecutive_misses = 0;
        self.status = ActorStatus::Healthy;
    }
}

#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    actors: Arc<RwLock<HashMap<GodName, HeartbeatState>>>,
    default_config: HeartbeatConfig,
}

impl HeartbeatMonitor {
    pub fn new() -> Self {
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            default_config: HeartbeatConfig::default(),
        }
    }
    
    pub async fn register(&self, actor: GodName, config: Option<HeartbeatConfig>) {
        let mut actors = self.actors.write().await;
        actors.insert(actor, HeartbeatState::new(actor, config.unwrap_or_else(|| self.default_config.clone())));
    }
    
    pub async fn unregister(&self, actor: &GodName) {
        let mut actors = self.actors.write().await;
        actors.remove(actor);
    }
    
    pub async fn receive_heartbeat(&self, actor: GodName) {
        let mut actors = self.actors.write().await;
        if let Some(state) = actors.get_mut(&actor) {
            state.mark_received();
        }
    }
    
    pub async fn check_all(&self, watchdog: &super::Watchdog) {
        let actors = self.actors.read().await;
        for (actor, state) in actors.iter() {
            if state.is_timed_out() {
                drop(actors);
                let mut actors = self.actors.write().await;
                if let Some(s) = actors.get_mut(actor) {
                    s.mark_missed();
                    if s.status == ActorStatus::Dead {
                        watchdog.report_death(*actor, "Heartbeat timeout".to_string()).await;
                    }
                }
                let actors = self.actors.read().await;
            }
        }
    }
    
    pub async fn set_interval(&self, actor: GodName, interval_ms: u64) {
        let mut actors = self.actors.write().await;
        if let Some(state) = actors.get_mut(&actor) {
            state.config.interval_ms = interval_ms;
        }
    }
    
    pub fn monitored_count(&self) -> usize {
        self.actors.blocking_read().len()
    }
    
    pub async fn get_stats(&self) -> HeartbeatStats {
        let actors = self.actors.read().await;
        let healthy = actors.values().filter(|s| s.status == ActorStatus::Healthy).count();
        let unhealthy = actors.values().filter(|s| s.status == ActorStatus::Unhealthy).count();
        let dead = actors.values().filter(|s| s.status == ActorStatus::Dead).count();
        
        HeartbeatStats {
            total: actors.len(),
            healthy,
            unhealthy,
            dead,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatStats {
    pub total: usize,
    pub healthy: usize,
    pub unhealthy: usize,
    pub dead: usize,
}
