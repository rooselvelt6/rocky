// src/traits/persistable.rs
// OLYMPUS v13 - Persistable Trait
// Interface para persistencia de datos

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Interface para actores que pueden persistir su estado
#[async_trait]
pub trait Persistable: Send + Sync {
    /// Guardar estado
    async fn save(&self) -> Result<serde_json::Value, PersistenceError>;

    /// Cargar estado
    async fn load(&mut self, state: &serde_json::Value) -> Result<(), PersistenceError>;

    /// Obtener clave de persistencia
    fn persistence_key(&self) -> String;

    /// Versión del esquema de persistencia
    fn schema_version(&self) -> u32;
}

/// Transacción de persistencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceTransaction {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: TransactionStatus,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Cola de transacciones pendientes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransactions {
    pub transactions: Vec<PersistenceTransaction>,
    pub max_size: usize,
    pub current_size: usize,
}

impl PendingTransactions {
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: Vec::new(),
            max_size,
            current_size: 0,
        }
    }

    pub fn push(&mut self, transaction: PersistenceTransaction) {
        if self.current_size >= self.max_size {
            self.transactions.remove(0);
        }
        self.transactions.push(transaction);
        self.current_size = self.transactions.len();
    }

    pub fn pop(&mut self) -> Option<PersistenceTransaction> {
        self.transactions.pop()
    }

    pub fn len(&self) -> usize {
        self.current_size
    }

    pub fn is_empty(&self) -> bool {
        self.current_size == 0
    }
}

/// Error de persistencia
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PersistenceError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: u32, got: u32 },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    #[error("Buffer full, cannot store more transactions")]
    BufferFull,
}

/// Resultado de una operación de persistencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceResult {
    pub success: bool,
    pub key: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub error: Option<String>,
}
