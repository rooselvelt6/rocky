// src/actors/apollo/mod.rs
// OLYMPUS v13 - Apollo: Dios de los Eventos y Logging

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, ResponsePayload};
use crate::errors::ActorError;

pub mod events;
pub mod logging;
pub mod metrics;
pub mod queries;

#[derive(Debug, Clone)]
pub struct Apollo {
    name: GodName,
    state: ActorState,
    events: Arc<RwLock<Vec<super::EventPayload>>>,
}

impl Apollo {
    pub async fn new() -> Self {
        Self {
            name: GodName::Apollo,
            state: ActorState::new(GodName::Apollo),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl OlympianActor for Apollo {
    fn name(&self) -> GodName { GodName::Apollo }
    fn domain(&self) -> DivineDomain { DivineDomain::Events }
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> { Ok(ResponsePayload::Ack { message_id: msg.id }) }
    fn persistent_state(&self) -> serde_json::Value { serde_json::json!({}) }
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> { Ok(()) }
    fn heartbeat(&self) -> GodHeartbeat { unimplemented!() }
    async fn health_check(&self) -> HealthStatus { unimplemented!() }
    fn config(&self) -> Option<&ActorConfig> { None }
    async fn initialize(&mut self) -> Result<(), ActorError> { Ok(()) }
    async fn shutdown(&mut self) -> Result<(), ActorError> { Ok(()) }
    fn actor_state(&self) -> ActorState { self.state.clone() }
}

use serde::{Deserialize, Serialize};
