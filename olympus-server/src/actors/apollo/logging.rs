use serde::{Deserialize, Serialize};
use crate::actors::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub actor: GodName,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

impl LogEntry {
    pub fn new(level: LogLevel, actor: GodName, message: String) -> Self {
        Self {
            level,
            actor,
            message,
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}
