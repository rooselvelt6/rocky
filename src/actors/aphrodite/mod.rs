// src/actors/aphrodite/mod.rs
// OLYMPUS v13 - Aphrodite: Diosa de la Belleza y UI

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, ResponsePayload};
use crate::errors::ActorError;

pub mod theme;
pub mod components;
pub mod animations;
pub mod accessibility;

#[derive(Debug, Clone)]
pub struct Aphrodite {
    name: GodName,
    state: ActorState,
    current_theme: Arc<RwLock<Theme>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub error: String,
    pub success: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub base_size: u16,
    pub scale_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
}

impl Aphrodite {
    pub async fn new() -> Self {
        Self {
            name: GodName::Aphrodite,
            state: ActorState::new(GodName::Aphrodite),
            current_theme: Arc::new(RwLock::new(Theme::default())),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: ThemeColors {
                primary: "#3b82f6".to_string(),
                secondary: "#8b5cf6".to_string(),
                accent: "#ec4899".to_string(),
                background: "#ffffff".to_string(),
                surface: "#f3f4f6".to_string(),
                error: "#ef4444".to_string(),
                success: "#22c55e".to_string(),
                warning: "#f59e0b".to_string(),
            },
            typography: ThemeTypography {
                font_family: "Inter".to_string(),
                base_size: 16,
                scale_ratio: 1.25,
            },
            spacing: ThemeSpacing {
                xs: "0.25rem".to_string(),
                sm: "0.5rem".to_string(),
                md: "1rem".to_string(),
                lg: "1.5rem".to_string(),
                xl: "2rem".to_string(),
            },
        }
    }
}

#[async_trait]
impl OlympianActor for Aphrodite {
    fn name(&self) -> GodName { GodName::Aphrodite }
    fn domain(&self) -> DivineDomain { DivineDomain::UI }
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> { Ok(ResponsePayload::Ack { message_id: msg.id }) }
    async fn persistent_state(&self) -> serde_json::Value { serde_json::json!({}) }
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> { Ok(()) }
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name(),
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.1,
            memory_usage_mb: 25.0,
            uptime_seconds: 0,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus::healthy(self.name())
    }
    fn config(&self) -> Option<&ActorConfig> { None }
    async fn initialize(&mut self) -> Result<(), ActorError> { Ok(()) }
    async fn shutdown(&mut self) -> Result<(), ActorError> { Ok(()) }
    fn actor_state(&self) -> ActorState { self.state.clone() }
}

use serde::{Deserialize, Serialize};
