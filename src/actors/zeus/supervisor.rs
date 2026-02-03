// src/actors/zeus/supervisor.rs
// OLYMPUS v13 - Zeus Supervision Manager
// Árbol de supervisión jerárquica

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

use super::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::traits::supervisor_trait::{SupervisionTree, SupervisedActor, ActorSupervisionStatus};

/// Manager del árbol de supervisión
#[derive(Debug, Clone)]
pub struct SupervisionManager {
    actors: Arc<RwLock<HashMap<GodName, SupervisedActor>>>,
    strategies: Arc<RwLock<HashMap<GodName, RecoveryStrategy>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympicHealth {
    pub healthy_count: usize,
    pub dead_count: usize,
    pub degraded_count: usize,
    pub is_critical: bool,
}

impl SupervisionManager {
    pub fn new() -> Self {
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register_actor(&mut self, actor: GodName, strategy: RecoveryStrategy) {
        let mut actors = self.actors.write().await;
        actors.insert(actor, SupervisedActor {
            name: actor,
            status: ActorSupervisionStatus::Running,
            restarts: 0,
            last_restart: None,
            strategy: strategy.clone(),
            children: Vec::new(),
        });
        
        let mut strategies = self.strategies.write().await;
        strategies.insert(actor, strategy);
    }
    
    pub async fn unregister_actor(&mut self, actor: &GodName) {
        let mut actors = self.actors.write().await;
        actors.remove(actor);
    }
    
    pub async fn update_status(&self, actor: GodName, status: ActorSupervisionStatus) {
        let mut actors = self.actors.write().await;
        if let Some(a) = actors.get_mut(&actor) {
            a.status = status;
        }
    }
    
    pub async fn increment_restarts(&self, actor: GodName) -> u32 {
        let mut actors = self.actors.write().await;
        if let Some(a) = actors.get_mut(&actor) {
            a.restarts += 1;
            a.last_restart = Some(Utc::now());
            a.status = ActorSupervisionStatus::Recovering;
            a.restarts
        } else {
            0
        }
    }
    
    pub async fn get_tree(&self) -> SupervisionTree {
        let actors = self.actors.read().await;
        let children: Vec<SupervisedActor> = actors.values().cloned().collect();
        
        SupervisionTree {
            root: SupervisedActor {
                name: GodName::Zeus,
                status: ActorSupervisionStatus::Running,
                restarts: 0,
                last_restart: None,
                strategy: RecoveryStrategy::OneForOne,
                children: actors.keys().cloned().collect(),
            },
            children,
            total_actors: actors.len(),
            healthy_actors: actors.values().filter(|a| a.status == ActorSupervisionStatus::Running).count(),
            dead_actors: actors.values().filter(|a| a.status == ActorSupervisionStatus::Dead).count(),
        }
    }
    
    pub async fn get_health(&self) -> OlympicHealth {
        let actors = self.actors.read().await;
        let healthy = actors.values().filter(|a| a.status == ActorSupervisionStatus::Running).count();
        let dead = actors.values().filter(|a| a.status == ActorSupervisionStatus::Dead).count();
        let degraded = actors.values().filter(|a| a.status == ActorSupervisionStatus::Degraded).count();
        
        OlympicHealth {
            healthy_count: healthy,
            dead_count: dead,
            degraded_count: degraded,
            is_critical: dead > 0 || healthy == 0,
        }
    }
    
    pub async fn get_strategy(&self, actor: GodName) -> Option<RecoveryStrategy> {
        let strategies = self.strategies.read().await;
        strategies.get(&actor).cloned()
    }
}
