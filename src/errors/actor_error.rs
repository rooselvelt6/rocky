// src/errors/actor_error.rs
// OLYMPUS v13 - Errores de Actores

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::actors::GodName;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ActorError {
    #[error("Actor {god} panic: {message}")]
    Panic {
        god: GodName,
        message: String,
        backtrace: Option<String>,
    },

    #[error("Actor {god} timeout: {message}")]
    Timeout { god: GodName, message: String },

    #[error("Actor {god} not found")]
    NotFound { god: GodName },

    #[error("Actor {god} already running")]
    AlreadyRunning { god: GodName },

    #[error("Actor {god} not running")]
    NotRunning { god: GodName },

    #[error("Invalid message for actor {god}: {message}")]
    InvalidMessage { god: GodName, message: String },

    #[error("State error in actor {god}: {message}")]
    StateError { god: GodName, message: String },

    #[error("Recovery failed for actor {god}: {message}")]
    RecoveryFailed {
        god: GodName,
        message: String,
        attempts: u32,
    },

    #[error("Health check failed for actor {god}: {message}")]
    HealthCheckFailed { god: GodName, message: String },

    #[error("Configuration error for actor {god}: {message}")]
    ConfigurationError { god: GodName, message: String },

    #[error("Serialization error in actor {god}: {message}")]
    SerializationError { god: GodName, message: String },

    #[error("Unknown error in actor {god}: {message}")]
    Unknown { god: GodName, message: String },
}

impl ActorError {
    pub fn panic(god: GodName, message: &str) -> Self {
        Self::Panic {
            god,
            message: message.to_string(),
            backtrace: None,
        }
    }

    pub fn not_found(god: GodName) -> Self {
        Self::NotFound { god }
    }

    pub fn invalid_message(god: GodName, message: &str) -> Self {
        Self::InvalidMessage {
            god,
            message: message.to_string(),
        }
    }
}
