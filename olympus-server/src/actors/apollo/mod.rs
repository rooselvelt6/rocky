// src/actors/apollo/mod.rs
// OLYMPUS v16 - Apollo: Dios de las Artes y Eventos
// Implementación sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use serde::{Deserialize, Serialize};
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, EventPayload};
use crate::errors::ActorError;

pub mod events;
pub mod logging;
pub mod metrics;
pub mod queries;

pub use events::ApolloEvent;
pub use logging::{LogEntry, LogLevel};
pub use metrics::EventMetrics;

/// Apollo State for Ractor
pub struct ApolloState {
    pub name: GodName,
    pub metadata: ActorState,
    pub config: ActorConfig,
    pub events: Vec<ApolloEvent>,
    pub logs: Vec<LogEntry>,
    pub metrics: EventMetrics,
}

pub struct Apollo;

#[async_trait]
impl Actor for Apollo {
    type Msg = ActorMessage;
    type State = ApolloState;
    type Arguments = ActorConfig;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, config: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(ApolloState {
            name: GodName::Apollo,
            metadata: ActorState::new(GodName::Apollo),
            config,
            events: Vec::with_capacity(1000),
            logs: Vec::with_capacity(1000),
            metrics: EventMetrics::default(),
        })
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message.payload {
            MessagePayload::Event(event) => {
                let apollo_event = ApolloEvent::new(
                    message.from.unwrap_or(GodName::Zeus),
                    &format!("{:?}", event),
                    serde_json::to_value(&event).unwrap_or(serde_json::json!({})),
                );
                self.record_event(apollo_event, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(Ok(ResponsePayload::Ack { message_id: message.id })); }
            }
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

impl Apollo {
    async fn record_event(&self, event: ApolloEvent, state: &mut ApolloState) {
        if state.events.len() >= 1000 {
            state.events.remove(0);
        }
        state.events.push(event.clone());
        state.metrics.record_event(event.source, &event.event_type);
    }

    async fn record_log(&self, log: LogEntry, state: &mut ApolloState) {
        if state.logs.len() >= 1000 {
            state.logs.remove(0);
        }
        state.logs.push(log);
    }

    async fn handle_command(&self, cmd: CommandPayload, state: &mut ApolloState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                if let Some(action) = data.get("action").and_then(|v| v.as_str()) {
                    match action {
                        "log" => {
                            let message = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
                            let level_str = data.get("level").and_then(|v| v.as_str()).unwrap_or("Info");
                            let level = match level_str {
                                "Debug" => LogLevel::Debug,
                                "Warn" => LogLevel::Warn,
                                "Error" => LogLevel::Error,
                                "Critical" => LogLevel::Critical,
                                _ => LogLevel::Info,
                            };
                            let actor = data.get("actor")
                                .and_then(|v| serde_json::from_value::<GodName>(v.clone()).ok())
                                .unwrap_or(GodName::Zeus);
                            
                            self.record_log(LogEntry::new(level, actor, message.to_string()), state).await;
                            Ok(ResponsePayload::Success { message: "Log recorded".to_string() })
                        }
                        _ => Err(ActorError::InvalidCommand { god: GodName::Apollo, reason: format!("Action '{}' not supported", action) }),
                    }
                } else {
                    Err(ActorError::InvalidCommand { god: GodName::Apollo, reason: "Missing action".to_string() })
                }
            }
            _ => Err(ActorError::InvalidCommand { god: GodName::Apollo, reason: "Command not supported".to_string() }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, state: &ApolloState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                Ok(ResponsePayload::Stats { data: serde_json::to_value(&state.metrics).unwrap_or_default() })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                match query_type {
                    "recent_events" => {
                        Ok(ResponsePayload::Data { data: serde_json::to_value(&state.events).unwrap_or_default() })
                    }
                    "recent_logs" => {
                        Ok(ResponsePayload::Data { data: serde_json::to_value(&state.logs).unwrap_or_default() })
                    }
                    _ => Err(ActorError::InvalidQuery { god: GodName::Apollo, reason: "Query type not supported".to_string() }),
                }
            }
            _ => Err(ActorError::InvalidQuery { god: GodName::Apollo, reason: "Query not supported".to_string() }),
        }
    }
}
