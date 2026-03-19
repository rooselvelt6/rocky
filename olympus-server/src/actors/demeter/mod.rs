// src/actors/demeter/mod.rs
// OLYMPUS v16 - Demeter: Diosa de la Agricultura y Recursos
// Implementación sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use tracing::info;
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{ActorState, ActorConfig};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

pub mod resources;
pub mod alerts;

pub use resources::{ResourceSnapshot, ResourceType, ResourceMetrics};
pub use alerts::{AlertThreshold, AlertLevel, ResourceAlert};

/// Demeter State for Ractor
pub struct DemeterState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    pub metrics_history: Vec<ResourceSnapshot>,
    pub thresholds: Vec<AlertThreshold>,
    pub active_alerts: Vec<ResourceAlert>,
    pub monitoring: bool,
    pub sample_interval_secs: u64,
}

pub struct Demeter;

#[async_trait]
impl Actor for Demeter {
    type Msg = ActorMessage;
    type State = DemeterState;
    type Arguments = ActorConfig;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, config: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(DemeterState {
            name: GodName::Demeter,
            metadata: ActorState::new(GodName::Demeter),
            config,
            metrics_history: Vec::with_capacity(1000),
            thresholds: Self::default_thresholds(),
            active_alerts: Vec::new(),
            monitoring: false,
            sample_interval_secs: 30,
        })
    }

    async fn post_start(&self, _myself: ActorRef<Self::Msg>, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        state.monitoring = true;
        info!("🌾 Demeter: Resource Monitoring System ready");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message.payload {
            MessagePayload::Command(cmd) => {
                let res = self.handle_command(cmd, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            MessagePayload::Query(query) => {
                let res = self.handle_query(query, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            _ => if let Some(reply) = message.reply_to { let _ = reply.send(Ok(ResponsePayload::Ack { message_id: message.id })); }
        }
        Ok(())
    }
}

impl Demeter {
    fn default_thresholds() -> Vec<AlertThreshold> {
        vec![
            AlertThreshold::new(ResourceType::Cpu, 0.80, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Cpu, 0.95, AlertLevel::Critical),
            AlertThreshold::new(ResourceType::Memory, 0.80, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Memory, 0.95, AlertLevel::Critical),
            AlertThreshold::new(ResourceType::Storage, 0.85, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Storage, 0.98, AlertLevel::Critical),
        ]
    }

    async fn handle_command(&self, _cmd: CommandPayload, _state: &mut DemeterState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Success { message: "Demeter resource action processed".to_string() })
    }

    async fn handle_query(&self, _query: QueryPayload, state: &DemeterState) -> Result<ResponsePayload, ActorError> {
        Ok(ResponsePayload::Data { data: serde_json::json!({ "alerts_active": state.active_alerts.len() }) })
    }
}
