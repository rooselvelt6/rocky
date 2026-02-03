// src/errors/persistence_error.rs
// OLYMPUS v13 - Errores de Persistencia

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum PersistenceError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: u32, got: u32 },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    #[error("Buffer full, cannot store more transactions")]
    BufferFull,

    #[error("Valkey error: {0}")]
    ValkeyError(String),

    #[error("SurrealDB error: {0}")]
    SurrealError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<String> for PersistenceError {
    fn from(e: String) -> Self {
        Self::Unknown(e)
    }
}
