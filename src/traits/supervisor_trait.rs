// src/traits/supervisor_trait.rs
// OLYMPUS v13 - Supervisor Trait
// Interface para Zeus y Erinyes

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::actors::GodName;
use crate::traits::message::RecoveryStrategy;
use crate::errors::OlympusError;

/// Estado del árbol de supervisión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionTree {
    pub root: SupervisedActor,
    pub children: Vec<SupervisedActor>,
    pub total_actors: usize,
    pub healthy_actors: usize,
    pub dead_actors: usize,
}

/// Actor supervisado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisedActor {
    pub name: GodName,
    pub status: ActorSupervisionStatus,
    pub restarts: u32,
    pub last_restart: Option<chrono::DateTime<chrono::Utc>>,
    pub strategy: RecoveryStrategy,
    pub children: Vec<GodName>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActorSupervisionStatus {
    Running,
    Starting,
    Stopping,
    Dead,
    Recovering,
    Failed,
}

/// Interface para supervisores (Zeus, Erinyes)
#[async_trait]
pub trait Supervisor: Send + Sync {
    /// Iniciar un actor hijo
    async fn start_child(&mut self, god: GodName) -> Result<(), SupervisorError>;

    /// Detener un actor hijo
    async fn stop_child(&mut self, god: GodName, reason: String) -> Result<(), SupervisorError>;

    /// Reiniciar un actor hijo
    async fn restart_child(&mut self, god: GodName) -> Result<(), SupervisorError>;

    /// Obtener lista de hijos
    fn children(&self) -> Vec<GodName>;

    /// Obtener estado del árbol de supervisión
    fn supervision_tree(&self) -> SupervisionTree;

    /// Configurar estrategia de recuperación para un actor
    fn set_recovery_strategy(&mut self, god: GodName, strategy: RecoveryStrategy);

    /// Obtener estrategia de recuperación de un actor
    fn get_recovery_strategy(&self, god: GodName) -> Option<RecoveryStrategy>;
}

/// Interface para actores que pueden ser supervisados
#[async_trait]
pub trait Supervisable: Send + Sync {
    /// Iniciar el actor
    async fn start(&mut self) -> Result<(), SupervisorError>;

    /// Detener el actor
    async fn stop(&mut self) -> Result<(), SupervisorError>;

    /// Reiniciar el actor
    async fn restart(&mut self) -> Result<(), SupervisorError>;

    /// Obtener estado actual
    fn status(&self) -> ActorSupervisionStatus;

    /// Configurar supervisor
    fn set_supervisor(&mut self, supervisor: GodName);
}

/// Métricas de supervisión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionMetrics {
    pub total_restarts: u64,
    pub successful_restarts: u64,
    pub failed_restarts: u64,
    pub escalations_to_zeus: u64,
    pub average_recovery_time_ms: u64,
    pub last_recovery_time_ms: u64,
}
