use serde::{Deserialize, Serialize};
use crate::traits::message::MessagePriority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApolloEvent {
    pub id: String,
    pub source: crate::actors::GodName,
    pub event_type: String,
    pub priority: MessagePriority,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApolloEvent {
    pub fn new(source: crate::actors::GodName, event_type: &str, data: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            event_type: event_type.to_string(),
            priority: MessagePriority::Normal,
            data,
            metadata: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        }
    }
}
