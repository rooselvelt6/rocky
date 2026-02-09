use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

pub mod events;
pub mod logging;
pub mod metrics;
pub mod queries;

pub use events::ApolloEvent;
pub use logging::{LogEntry, LogLevel};
pub use metrics::EventMetrics;
pub use queries::EventQuery;

#[derive(Debug)]
pub struct Apollo {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    events: Arc<RwLock<Vec<ApolloEvent>>>,
    logs: Arc<RwLock<Vec<LogEntry>>>,
    metrics: Arc<RwLock<EventMetrics>>,
}

impl Apollo {
    pub async fn new() -> Self {
        Self {
            name: GodName::Apollo,
            state: ActorState::new(GodName::Apollo),
            config: ActorConfig::default(),
            events: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            logs: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            metrics: Arc::new(RwLock::new(EventMetrics::default())),
        }
    }

    async fn record_event(&self, event: ApolloEvent) {
        let mut events = self.events.write().await;
        if events.len() >= 1000 {
            events.remove(0);
        }
        events.push(event.clone());
        
        let mut metrics = self.metrics.write().await;
        metrics.record_event(event.source, &event.event_type);
    }

    async fn record_log(&self, log: LogEntry) {
        let mut logs = self.logs.write().await;
        if logs.len() >= 1000 {
            logs.remove(0);
        }
        logs.push(log);
    }
}

#[async_trait]
impl OlympianActor for Apollo {
    fn name(&self) -> GodName { GodName::Apollo }
    fn domain(&self) -> DivineDomain { DivineDomain::Events }

    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();

        match msg.payload {
            MessagePayload::Event(event) => {
                let apollo_event = ApolloEvent::new(
                    msg.from.unwrap_or(GodName::Zeus),
                    &format!("{:?}", event),
                    serde_json::to_value(&event).unwrap_or(serde_json::json!({})),
                );
                self.record_event(apollo_event).await;
                Ok(ResponsePayload::Ack { message_id: msg.id })
            }
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }

    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Apollo",
            "messages": self.state.message_count,
            "status": self.state.status,
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }

    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name.clone(),
            status: self.state.status.clone(),
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("☀️ Apollo: Iniciando motor de eventos y logs v15...");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("☀️ Apollo: Apagando motor de eventos...");
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::message::{EventPayload, ActorMessage};
    use serde_json::json;

    #[tokio::test]
    async fn test_apollo_event_recording() -> Result<(), ActorError> {
        let mut apollo = Apollo::new().await;
        apollo.initialize().await?;

        // Simular la llegada de un evento
        let event_msg = ActorMessage {
            id: "evt1".to_string(),
            from: Some(GodName::Athena),
            to: GodName::Apollo,
            priority: crate::traits::message::MessagePriority::Normal,
            payload: MessagePayload::Event(EventPayload::ActorStarted { actor: GodName::Athena }),
            timestamp: chrono::Utc::now(),
            metadata: json!({}),
        };

        apollo.handle_message(event_msg).await?;

        // Verificar métricas
        let stats_resp = apollo.handle_query(QueryPayload::Metrics).await?;
        if let ResponsePayload::Stats { data } = stats_resp {
            assert_eq!(data["total_events"], 1);
            assert_eq!(data["events_per_actor"]["Athena"], 1);
        } else {
            panic!("Expected Stats response");
        }

        // Verificar logs
        let log_cmd = CommandPayload::Custom(json!({
            "action": "log",
            "message": "Test log message",
            "level": "Info",
            "actor": "Athena"
        }));
        
        let log_msg = ActorMessage {
            id: "log1".to_string(),
            from: Some(GodName::Athena),
            to: GodName::Apollo,
            priority: crate::traits::message::MessagePriority::Normal,
            payload: MessagePayload::Command(log_cmd),
            timestamp: chrono::Utc::now(),
            metadata: json!({}),
        };

        apollo.handle_message(log_msg).await?;

        let logs_resp = apollo.handle_query(QueryPayload::Custom(json!({"query_type": "recent_logs"}))).await?;
        if let ResponsePayload::Data { data } = logs_resp {
            let logs = data.as_array().unwrap();
            assert!(!logs.is_empty());
            assert_eq!(logs[0]["message"], "Test log message");
        } else {
            panic!("Expected Data response");
        }

        Ok(())
    }
}

impl Apollo {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
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
                            
                            self.record_log(LogEntry::new(level, actor, message.to_string())).await;
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

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let metrics = self.metrics.read().await;
                Ok(ResponsePayload::Stats { data: serde_json::to_value(&*metrics).unwrap_or_default() })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                match query_type {
                    "recent_events" => {
                        let events = self.events.read().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(&*events).unwrap_or_default() })
                    }
                    "recent_logs" => {
                        let logs = self.logs.read().await;
                        Ok(ResponsePayload::Data { data: serde_json::to_value(&*logs).unwrap_or_default() })
                    }
                    _ => Err(ActorError::InvalidQuery { god: GodName::Apollo, reason: "Query type not supported".to_string() }),
                }
            }
            _ => Err(ActorError::InvalidQuery { god: GodName::Apollo, reason: "Query not supported".to_string() }),
        }
    }
}
