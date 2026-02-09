// src/actors/zeus/supervisor.rs
// OLYMPUS v15 - Zeus Supervision Manager
// Árbol de supervisión jerárquica con estrategias de recovery avanzadas

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration, Instant};
use chrono::Utc;
use tracing::{info, warn, error, debug};

use super::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::traits::supervisor_trait::{SupervisionTree, SupervisedActor, ActorSupervisionStatus};
use crate::errors::ActorError;

/// Manager completo del árbol de supervisión
#[derive(Debug, Clone)]
pub struct SupervisionManager {
    // Actores supervisados
    actors: Arc<RwLock<HashMap<GodName, SupervisedActor>>>,
    // Estrategias de recovery por actor
    strategies: Arc<RwLock<HashMap<GodName, RecoveryStrategy>>>,
    // Dependencias padre-hijo
    dependencies: Arc<RwLock<HashMap<GodName, Vec<GodName>>>>, // padre -> hijos
    reverse_deps: Arc<RwLock<HashMap<GodName, GodName>>>, // hijo -> padre
    // Historial de reinicios para límites temporales
    restart_history: Arc<RwLock<HashMap<GodName, VecDeque<chrono::DateTime<chrono::Utc>>>>>,
    // Configuración de límites
    max_restarts: u32,
    restart_window_seconds: u64,
    // Canales para comunicación
    lifecycle_tx: mpsc::Sender<LifecycleEvent>,
    // Auto-recovery habilitado
    auto_recovery: Arc<RwLock<bool>>,
}

/// Eventos del ciclo de vida de actores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    Registered { actor: GodName, parent: Option<GodName>, strategy: RecoveryStrategy },
    ActorStarted { actor: GodName },
    ActorStopped { actor: GodName, reason: String },
    Restarted { actor: GodName, attempt: u32 },
    Failed { actor: GodName, error: String },
    ActorRecovered { actor: GodName },
    Unregistered { actor: GodName },
}

/// Estado de salud del Olimpo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympicHealth {
    pub healthy_count: usize,
    pub dead_count: usize,
    pub degraded_count: usize,
    pub recovering_count: usize,
    pub starting_count: usize,
    pub total_actors: usize,
    pub critical_actors_down: Vec<GodName>,
    pub is_critical: bool,
    pub health_percentage: f64,
}

impl OlympicHealth {
    pub fn is_critical(&self) -> bool {
        self.is_critical || self.dead_count > 0 || self.health_percentage < 50.0
    }
    
    pub fn is_healthy(&self) -> bool {
        self.dead_count == 0 && self.degraded_count == 0 && self.health_percentage >= 90.0
    }
}

/// Información de dependencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub parent: Option<GodName>,
    pub children: Vec<GodName>,
    pub depth: usize,
}

impl SupervisionManager {
    pub fn new() -> Self {
        let (lifecycle_tx, _) = mpsc::channel(1000);
        
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_deps: Arc::new(RwLock::new(HashMap::new())),
            restart_history: Arc::new(RwLock::new(HashMap::new())),
            max_restarts: 3,
            restart_window_seconds: 30,
            lifecycle_tx,
            auto_recovery: Arc::new(RwLock::new(true)),
        }
    }
    
    pub fn with_config(max_restarts: u32, restart_window_seconds: u64) -> Self {
        let (lifecycle_tx, _) = mpsc::channel(1000);
        
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_deps: Arc::new(RwLock::new(HashMap::new())),
            restart_history: Arc::new(RwLock::new(HashMap::new())),
            max_restarts,
            restart_window_seconds,
            lifecycle_tx,
            auto_recovery: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Registra un actor en el árbol de supervisión
    pub async fn register_actor(
        &self, 
        actor: GodName, 
        parent: Option<GodName>,
        strategy: RecoveryStrategy
    ) -> Result<(), ActorError> {
        let mut actors = self.actors.write().await;
        
        if actors.contains_key(&actor) {
            return Err(ActorError::AlreadyRunning { god: actor });
        }
        
        let supervised = SupervisedActor {
            name: actor.clone(),
            status: ActorSupervisionStatus::Starting,
            restarts: 0,
            last_restart: None,
            strategy: strategy.clone(),
            children: Vec::new(),
        };
        
        actors.insert(actor.clone(), supervised);
        drop(actors);
        
        // Registrar estrategia
        let mut strategies = self.strategies.write().await;
        strategies.insert(actor.clone(), strategy.clone());
        drop(strategies);
        
        // Registrar dependencias
        if let Some(ref parent_name) = parent {
            let mut deps = self.dependencies.write().await;
            deps.entry(parent_name.clone()).or_insert_with(Vec::new).push(actor.clone());
            drop(deps);
            
            let mut rev_deps = self.reverse_deps.write().await;
            rev_deps.insert(actor.clone(), parent_name.clone());
            drop(rev_deps);
        }
        
        // Inicializar historial de reinicios
        let mut history = self.restart_history.write().await;
        history.insert(actor.clone(), VecDeque::new());
        drop(history);
        
        // Notificar evento
        let _ = self.lifecycle_tx.send(LifecycleEvent::Registered { 
            actor, 
            parent, 
            strategy 
        }).await;
        
        info!("⚡ Zeus: Actor {:?} registered with strategy {:?}", actor, strategy);
        Ok(())
    }
    
    /// Inicia un actor
    pub async fn start_actor(&self, actor: GodName) -> Result<(), ActorError> {
        let mut actors = self.actors.write().await;
        
        if let Some(a) = actors.get_mut(&actor) {
            a.status = ActorSupervisionStatus::Running;
            info!("⚡ Zeus: Actor {:?} started", actor);
        } else {
            return Err(ActorError::NotFound { god: actor });
        }
        
        drop(actors);
        let _ = self.lifecycle_tx.send(LifecycleEvent::ActorStarted { actor }).await;
        Ok(())
    }
    
    /// Detiene un actor
    pub async fn stop_actor(&self, actor: GodName, reason: String) -> Result<(), ActorError> {
        let mut actors = self.actors.write().await;
        
        if let Some(a) = actors.get_mut(&actor) {
            a.status = ActorSupervisionStatus::Stopping;
            info!("⚡ Zeus: Actor {:?} stopping: {}", actor, reason);
        } else {
            return Err(ActorError::NotFound { god: actor });
        }
        
        drop(actors);
        let _ = self.lifecycle_tx.send(LifecycleEvent::ActorStopped { actor, reason }).await;
        Ok(())
    }
    
    /// Reinicia un actor con manejo de límites
    pub async fn restart_actor(&self, actor: GodName) -> Result<RestartResult, ActorError> {
        // Verificar límites de reinicio
        if !self.can_restart(actor).await {
            return Ok(RestartResult::MaxRestartsExceeded);
        }
        
        let mut actors = self.actors.write().await;
        
        if let Some(a) = actors.get_mut(&actor) {
            a.status = ActorSupervisionStatus::Recovering;
            a.restarts += 1;
            a.last_restart = Some(Utc::now());
            
            let attempt = a.restarts;
            let strategy = a.strategy.clone();
            
            drop(actors);
            
            // Registrar en historial
            self.record_restart(actor).await;
            
            // Aplicar estrategia de recovery
            let affected_actors = match strategy {
                RecoveryStrategy::OneForOne => vec![actor],
                RecoveryStrategy::OneForAll => self.get_all_siblings(actor).await,
                RecoveryStrategy::RestForOne => self.get_rest_of_chain(actor).await,
                RecoveryStrategy::Escalate => vec![actor], // Se escala a Zeus
            };
            
            let _ = self.lifecycle_tx.send(LifecycleEvent::Restarted { actor, attempt }).await;
            
            info!("⚡ Zeus: Actor {:?} restarting (attempt {}, strategy {:?})", 
                actor, attempt, strategy);
            
            Ok(RestartResult::Success { affected_actors, attempt })
        } else {
            Err(ActorError::NotFound { god: actor })
        }
    }
    
    /// Desregistra un actor
    pub async fn unregister_actor(&self, actor: GodName) -> Result<(), ActorError> {
        let mut actors = self.actors.write().await;
        
        if actors.remove(&actor).is_none() {
            return Err(ActorError::NotFound { god: actor });
        }
        drop(actors);
        
        // Limpiar dependencias
        let mut deps = self.dependencies.write().await;
        for children in deps.values_mut() {
            children.retain(|&c| c != actor);
        }
        deps.remove(&actor);
        drop(deps);
        
        let mut rev_deps = self.reverse_deps.write().await;
        rev_deps.remove(&actor);
        drop(rev_deps);
        
        // Limpiar historial
        let mut history = self.restart_history.write().await;
        history.remove(&actor);
        drop(history);
        
        // Limpiar estrategia
        let mut strategies = self.strategies.write().await;
        strategies.remove(&actor);
        drop(strategies);
        
        let _ = self.lifecycle_tx.send(LifecycleEvent::Unregistered { actor }).await;
        
        info!("⚡ Zeus: Actor {:?} unregistered", actor);
        Ok(())
    }
    
    /// Actualiza el estado de un actor
    pub async fn update_status(&self, actor: GodName, status: ActorSupervisionStatus) {
        let mut actors = self.actors.write().await;
        
        if let Some(a) = actors.get_mut(&actor) {
            let old_status = a.status.clone();
            a.status = status.clone();
            
            debug!("⚡ Zeus: Actor {:?} status changed: {:?} -> {:?}", 
                actor, old_status, status);
        }
    }
    
    /// Marca un actor como fallido y aplica recovery
    pub async fn mark_failed(&self, actor: GodName, error: String) -> Result<RecoveryAction, ActorError> {
        let auto_recovery = *self.auto_recovery.read().await;
        
        self.update_status(actor, ActorSupervisionStatus::Failed).await;
        
        let _ = self.lifecycle_tx.send(LifecycleEvent::Failed { 
            actor, 
            error: error.clone() 
        }).await;
        
        error!("⚡ Zeus: Actor {:?} failed: {}", actor, error);
        
        if auto_recovery {
            let result = self.restart_actor(actor).await?;
            
            match result {
                RestartResult::Success { affected_actors, attempt } => {
                    return Ok(RecoveryAction::Restart { 
                        actors: affected_actors, 
                        attempt 
                    });
                }
                RestartResult::MaxRestartsExceeded => {
                    self.update_status(actor, ActorSupervisionStatus::Dead).await;
                    return Ok(RecoveryAction::Escalate { 
                        reason: "Max restarts exceeded".to_string() 
                    });
                }
            }
        }
        
        Ok(RecoveryAction::NoAction)
    }
    
    /// Marca un actor como recuperado
    pub async fn mark_recovered(&self, actor: GodName) {
        self.update_status(actor, ActorSupervisionStatus::Running).await;
        let _ = self.lifecycle_tx.send(LifecycleEvent::ActorRecovered { actor }).await;
        info!("⚡ Zeus: Actor {:?} recovered", actor);
    }
    
    /// Verifica si un actor puede ser reiniciado (límites temporales)
    async fn can_restart(&self, actor: GodName) -> bool {
        let history = self.restart_history.read().await;
        
        if let Some(restarts) = history.get(&actor) {
            let now = Utc::now();
            let window = chrono::Duration::seconds(self.restart_window_seconds as i64);
            
            // Contar reinicios en la ventana temporal
            let recent_restarts = restarts.iter()
                .filter(|&&time| now - time < window)
                .count();
            
            recent_restarts < self.max_restarts as usize
        } else {
            true
        }
    }
    
    /// Registra un reinicio en el historial
    async fn record_restart(&self, actor: GodName) {
        let mut history = self.restart_history.write().await;
        
        if let Some(restarts) = history.get_mut(&actor) {
            restarts.push_back(Utc::now());
            
            // Mantener solo los últimos 10 reinicios
            while restarts.len() > 10 {
                restarts.pop_front();
            }
        }
    }
    
    /// Obtiene todos los actores hermanos (mismo padre)
    async fn get_all_siblings(&self, actor: GodName) -> Vec<GodName> {
        let rev_deps = self.reverse_deps.read().await;
        
        if let Some(parent) = rev_deps.get(&actor) {
            let parent_name = *parent;
            drop(rev_deps);
            
            let deps = self.dependencies.read().await;
            if let Some(children) = deps.get(&parent_name) {
                return children.clone();
            }
        }
        
        vec![actor]
    }
    
    /// Obtiene el resto de la cadena (el actor y todos los posteriores)
    async fn get_rest_of_chain(&self, actor: GodName) -> Vec<GodName> {
        let mut result = vec![actor];
        
        // Obtener todos los descendientes
        let deps = self.dependencies.read().await;
        self.collect_descendants(&deps, actor, &mut result);
        
        result
    }
    
    fn collect_descendants(&self, deps: &HashMap<GodName, Vec<GodName>>, parent: GodName, result: &mut Vec<GodName>) {
        if let Some(children) = deps.get(&parent) {
            for child in children {
                if !result.contains(child) {
                    result.push(*child);
                    self.collect_descendants(deps, *child, result);
                }
            }
        }
    }
    
    /// Obtiene el árbol de supervisión completo
    pub async fn get_tree(&self) -> SupervisionTree {
        let actors = self.actors.read().await;
        let deps = self.dependencies.read().await;
        let rev_deps = self.reverse_deps.read().await;
        
        let children: Vec<SupervisedActor> = actors.values().cloned().collect();
        
        let healthy_count = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Running)
            .count();
        
        let dead_count = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Dead)
            .count();
        
        SupervisionTree {
            root: SupervisedActor {
                name: GodName::Zeus,
                status: ActorSupervisionStatus::Running,
                restarts: 0,
                last_restart: None,
                strategy: RecoveryStrategy::OneForOne,
                children: deps.get(&GodName::Zeus).cloned().unwrap_or_default(),
            },
            children,
            total_actors: actors.len(),
            healthy_actors: healthy_count,
            dead_actors: dead_count,
        }
    }
    
    /// Obtiene la salud del Olimpo
    pub async fn get_olympic_health(&self) -> OlympicHealth {
        let actors = self.actors.read().await;
        
        let healthy = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Running)
            .count();
        
        let dead = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Dead)
            .count();
        
        let degraded = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Failed || a.status == ActorSupervisionStatus::Recovering)
            .count();
        
        let recovering = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Recovering)
            .count();
        
        let starting = actors.values()
            .filter(|a| a.status == ActorSupervisionStatus::Starting)
            .count();
        
        let critical_actors: Vec<GodName> = actors.values()
            .filter(|a| {
                let is_critical_actor = matches!(a.name, 
                    GodName::Zeus | GodName::Hades | GodName::Poseidon | 
                    GodName::Erinyes | GodName::Hermes | GodName::Hestia
                );
                is_critical_actor && a.status != ActorSupervisionStatus::Running
            })
            .map(|a| a.name)
            .collect();
        
        let total = actors.len();
        let health_percentage = if total > 0 {
            (healthy as f64 / total as f64) * 100.0
        } else {
            100.0
        };
        
        OlympicHealth {
            healthy_count: healthy,
            dead_count: dead,
            degraded_count: degraded,
            recovering_count: recovering,
            starting_count: starting,
            total_actors: total,
            critical_actors_down: critical_actors.clone(),
            is_critical: !critical_actors.is_empty() || dead > 0,
            health_percentage,
        }
    }
    
    /// Obtiene información de dependencias
    pub async fn get_dependencies(&self, actor: GodName) -> DependencyInfo {
        let deps = self.dependencies.read().await;
        let rev_deps = self.reverse_deps.read().await;
        
        let parent = rev_deps.get(&actor).cloned();
        let children = deps.get(&actor).cloned().unwrap_or_default();
        
        // Calcular profundidad en el árbol
        let mut depth = 0;
        let mut current = actor;
        while let Some(p) = rev_deps.get(&current) {
            depth += 1;
            current = *p;
        }
        
        DependencyInfo {
            parent,
            children,
            depth,
        }
    }
    
    /// Obtiene la estrategia de recovery de un actor
    pub async fn get_strategy(&self, actor: GodName) -> Option<RecoveryStrategy> {
        let strategies = self.strategies.read().await;
        strategies.get(&actor).cloned()
    }
    
    /// Establece la estrategia de recovery
    pub async fn set_strategy(&self, actor: GodName, strategy: RecoveryStrategy) -> Result<(), ActorError> {
        let mut strategies = self.strategies.write().await;
        
        if !strategies.contains_key(&actor) {
            return Err(ActorError::NotFound { god: actor });
        }
        
        strategies.insert(actor, strategy.clone());
        drop(strategies);
        
        // Actualizar en el actor también
        let mut actors = self.actors.write().await;
        if let Some(a) = actors.get_mut(&actor) {
            a.strategy = strategy;
        }
        
        Ok(())
    }
    
    /// Obtiene la lista de actores afectados por una estrategia
    pub async fn get_affected_actors(&self, actor: GodName) -> Vec<GodName> {
        let strategy = self.get_strategy(actor).await.unwrap_or(RecoveryStrategy::OneForOne);
        
        match strategy {
            RecoveryStrategy::OneForOne => vec![actor],
            RecoveryStrategy::OneForAll => self.get_all_siblings(actor).await,
            RecoveryStrategy::RestForOne => self.get_rest_of_chain(actor).await,
            RecoveryStrategy::Escalate => vec![actor],
        }
    }
    
    /// Habilita/deshabilita auto-recovery
    pub async fn set_auto_recovery(&self, enabled: bool) {
        let mut auto = self.auto_recovery.write().await;
        *auto = enabled;
        info!("⚡ Zeus: Auto-recovery {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Verifica si el auto-recovery está habilitado
    pub async fn is_auto_recovery_enabled(&self) -> bool {
        *self.auto_recovery.read().await
    }
    
    /// Obtiene el canal de eventos del ciclo de vida
    pub fn get_lifecycle_channel(&self) -> mpsc::Sender<LifecycleEvent> {
        self.lifecycle_tx.clone()
    }
    
    /// Lista todos los actores con su estado
    pub async fn list_actors(&self) -> Vec<(GodName, ActorSupervisionStatus)> {
        let actors = self.actors.read().await;
        actors.iter().map(|(name, actor)| (*name, actor.status.clone())).collect()
    }
    
    /// Obtiene actores por estado
    pub async fn get_actors_by_status(&self, status: ActorSupervisionStatus) -> Vec<GodName> {
        let actors = self.actors.read().await;
        actors.values()
            .filter(|a| a.status == status)
            .map(|a| a.name)
            .collect()
    }
    
    /// Inicia monitoreo continuo del árbol
    pub fn start_tree_monitor(&self, interval_secs: u64) {
        let actors = self.actors.clone();
        let history = self.restart_history.clone();
        let max_restarts = self.max_restarts;
        let window_secs = self.restart_window_seconds;
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            
            loop {
                ticker.tick().await;
                
                let actors_guard = actors.read().await;
                let dead_count = actors_guard.values()
                    .filter(|a| a.status == ActorSupervisionStatus::Dead)
                    .count();
                
                if dead_count > 0 {
                    warn!("⚡ Zeus: {} actors are dead in supervision tree", dead_count);
                }
                
                // Limpiar historial antiguo
                drop(actors_guard);
                let mut history_guard = history.write().await;
                let now = Utc::now();
                let window = chrono::Duration::seconds(window_secs as i64 * 2); // Doble ventana para limpieza
                
                for restarts in history_guard.values_mut() {
                    while let Some(front) = restarts.front() {
                        if now - *front > window {
                            restarts.pop_front();
                        } else {
                            break;
                        }
                    }
                }
            }
        });
    }
}

/// Resultado de un reinicio
#[derive(Debug, Clone)]
pub enum RestartResult {
    Success { affected_actors: Vec<GodName>, attempt: u32 },
    MaxRestartsExceeded,
}

/// Acción de recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    NoAction,
    Restart { actors: Vec<GodName>, attempt: u32 },
    Escalate { reason: String },
}

impl Default for SupervisionManager {
    fn default() -> Self {
        Self::new()
    }
}
