// src/errors/olympus_error.rs
// OLYMPUS v13 - Errores Generales del Sistema

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum OlympusError {
    #[error("System not initialized: {0}")]
    NotInitialized(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Actor not found: {0}")]
    ActorNotFound(String),

    #[error("Actor already exists: {0}")]
    ActorAlreadyExists(String),

    #[error("Actor panic: {0}")]
    ActorPanic(String),

    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Channel closed: {0}")]
    ChannelClosed(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Supervision error: {0}")]
    SupervisionError(String),

    #[error("Governance error: {0}")]
    GovernanceError(String),

    #[error("Supervisor error: {0}")]
    SupervisorError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for OlympusError {
    fn from(e: std::io::Error) -> Self {
        Self::Unknown(e.to_string())
    }
}

impl From<serde_json::Error> for OlympusError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidConfiguration(e.to_string())
    }
}
