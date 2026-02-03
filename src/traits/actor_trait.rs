// src/traits/actor_trait.rs
// OLYMPUS v13 - OlympianActor Trait
// Interface base para todos los dioses del Olimpo

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actors::GodName;
use crate::actors::DivineDomain;
use crate::errors::ActorError;
use crate::traits::message::ActorMessage;
use crate::traits::message::ResponsePayload;

/// Heartbeat de un dios para Erinyes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodHeartbeat {
    pub god: GodName,
    pub status: ActorStatus,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub load: f64,
    pub memory_usage_mb: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActorStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Dead,
    Recovering,
}

/// Configuración de un actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorConfig {
    pub max_restarts: u32,
    pub restart_window_seconds: u64,
    pub heartbeat_interval_ms: u64,
    pub dead_letter_enabled: bool,
    pub persistence_enabled: bool,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_window_seconds: 30,
            heartbeat_interval_ms: 500,
            dead_letter_enabled: true,
            persistence_enabled: true,
        }
    }
}

/// Estado de salud de un actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub god: GodName,
    pub status: ActorStatus,
    pub uptime_seconds: u64,
    pub message_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub memory_usage_mb: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthStatus {
    pub fn healthy(god: GodName) -> Self {
        Self {
            god,
            status: ActorStatus::Healthy,
            uptime_seconds: 0,
            message_count: 0,
            error_count: 0,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn is_critical(&self) -> bool {
        self.status == ActorStatus::Dead || self.status == ActorStatus::Unhealthy
    }
}

/// Interface base para todos los dioses del Olimpo
#[async_trait]
pub trait OlympianActor: Send + Sync {
    /// Nombre del dios
    fn name(&self) -> GodName;

    /// Dominio del dios
    fn domain(&self) -> DivineDomain;

    /// Manejar mensaje entrante (estilo OTP GenServer)
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError>;

    /// Estado para persistencia
    fn persistent_state(&self) -> serde_json::Value;

    /// Cargar estado desde persistencia
    fn load_state(&mut self, state: &serde_json::Value) -> Result<(), ActorError>;

    /// Heartbeat para Erinyes
    fn heartbeat(&self) -> GodHeartbeat;

    /// Health check
    async fn health_check(&self) -> HealthStatus;

    /// Configuración del actor
    fn config(&self) -> Option<&ActorConfig>;

    /// Inicialización post-creación
    async fn initialize(&mut self) -> Result<(), ActorError>;

    /// Limpieza pre-destrucción
    async fn shutdown(&mut self) -> Result<(), ActorError>;

    /// Obtener estado actual del actor
    fn actor_state(&self) -> ActorState;
}

/// Estado interno del actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorState {
    pub god: GodName,
    pub status: ActorStatus,
    pub message_count: u64,
    pub error_count: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_message_time: chrono::DateTime<chrono::Utc>,
}

impl ActorState {
    pub fn new(god: GodName) -> Self {
        Self {
            god,
            status: ActorStatus::Healthy,
            message_count: 0,
            error_count: 0,
            start_time: chrono::Utc::now(),
            last_message_time: chrono::Utc::now(),
        }
    }
}

/// Wrapper para compartir estado del actor
pub type SharedActorState = Arc<RwLock<dyn OlympianActor>>;
