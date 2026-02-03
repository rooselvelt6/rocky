// src/actors/zeus/config.rs
// OLYMPUS v13 - Zeus Configuration
// Configuración del Gobernador Supremo

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeusConfig {
    pub heartbeat_interval_ms: u64,
    pub max_restarts: u32,
    pub restart_window_seconds: u64,
    pub self_evaluation_interval_seconds: u64,
    pub emergency_shutdown_timeout_seconds: u64,
    pub metrics_retention_hours: u64,
    pub governance_enabled: bool,
    pub auto_recovery_enabled: bool,
}

impl Default for ZeusConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: 500,
            max_restarts: 3,
            restart_window_seconds: 30,
            self_evaluation_interval_seconds: 5,
            emergency_shutdown_timeout_seconds: 10,
            metrics_retention_hours: 24,
            governance_enabled: true,
            auto_recovery_enabled: true,
        }
    }
}

impl ZeusConfig {
    pub fn from_env() -> Self {
        Self {
            heartbeat_interval_ms: std::env::var("ZEUS_HEARTBEAT_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),
            max_restarts: std::env::var("ZEUS_MAX_RESTARTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            restart_window_seconds: std::env::var("ZEUS_RESTART_WINDOW_S")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            self_evaluation_interval_seconds: std::env::var("ZEUS_EVAL_INTERVAL_S")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            emergency_shutdown_timeout_seconds: std::env::var("ZEUS_SHUTDOWN_TIMEOUT_S")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            metrics_retention_hours: std::env::var("ZEUS_METRICS_RETENTION_H")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            governance_enabled: std::env::var("ZEUS_GOVERNANCE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            auto_recovery_enabled: std::env::var("ZEUS_AUTO_RECOVERY_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}
