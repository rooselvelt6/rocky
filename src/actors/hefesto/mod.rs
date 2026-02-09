// src/actors/hefesto/mod.rs
// OLYMPUS v13 - Hefesto: Dios de la Forja y Configuración

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, ResponsePayload};
use crate::errors::ActorError;

pub mod config;
pub mod validation;
pub mod backup;
pub mod migration;

#[derive(Debug, Clone)]
pub struct Hefesto {
    name: GodName,
    state: ActorState,
    configs: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
}

impl Hefesto {
    pub async fn new() -> Self {
        Self {
            name: GodName::Hefesto,
            state: ActorState::new(GodName::Hefesto),
            configs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl OlympianActor for Hefesto {
    fn name(&self) -> GodName { GodName::Hefesto }
    fn domain(&self) -> DivineDomain { DivineDomain::Configuration }
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> { Ok(ResponsePayload::Ack { message_id: msg.id }) }
    async fn persistent_state(&self) -> serde_json::Value { serde_json::json!({}) }
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> { Ok(()) }
    fn heartbeat(&self) -> GodHeartbeat { unimplemented!() }
    async fn health_check(&self) -> HealthStatus { unimplemented!() }
    fn config(&self) -> Option<&ActorConfig> { None }
    async fn initialize(&mut self) -> Result<(), ActorError> { Ok(()) }
    async fn shutdown(&mut self) -> Result<(), ActorError> { Ok(()) }
    fn actor_state(&self) -> ActorState { self.state.clone() }
}

use serde::{Deserialize, Serialize};
