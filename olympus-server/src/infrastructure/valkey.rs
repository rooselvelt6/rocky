// src/infrastructure/valkey.rs
// OLYMPUS v13 - Valkey Embedded Client
// Memoria de corto plazo para buffer y cache

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValkeyError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValkeyConfig {
    pub port: u16,
    pub max_connections: u32,
    pub max_memory_mb: usize,
    pub persistence_enabled: bool,
}

impl Default for ValkeyConfig {
    fn default() -> Self {
        Self {
            port: 6379,
            max_connections: 10,
            max_memory_mb: 256,
            persistence_enabled: false,
        }
    }
}

#[derive(Debug)]
pub struct ValkeyStore {
    config: ValkeyConfig,
    // Note: Valkey embedded mode requires the valkey-server process
    // For embedded usage, we use DashMap as in-memory store
    memory: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    queues: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<String>>>>,
    hashes: Arc<tokio::sync::RwLock<std::collections::HashMap<String, std::collections::HashMap<String, String>>>>,
}

impl ValkeyStore {
    pub fn new(config: ValkeyConfig) -> Self {
        Self {
            config,
            memory: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            queues: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            hashes: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn default() -> Self {
        Self::new(ValkeyConfig::default())
    }

    // String operations
    pub async fn set(&self, key: &str, value: &str) -> Result<(), ValkeyError> {
        self.memory.write().await.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, ValkeyError> {
        Ok(self.memory.read().await.get(key).cloned())
    }

    pub async fn del(&self, key: &str) -> Result<(), ValkeyError> {
        self.memory.write().await.remove(key);
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, ValkeyError> {
        Ok(self.memory.read().await.contains_key(key))
    }

    pub async fn set_ex(&self, key: &str, value: &str, _seconds: u64) -> Result<(), ValkeyError> {
        self.memory.write().await.insert(key.to_string(), value.to_string());
        Ok(())
    }

    // Queue operations (LPush/RPop)
    pub async fn lpush(&self, queue: &str, value: &str) -> Result<(), ValkeyError> {
        let mut queues = self.queues.write().await;
        let q = queues.entry(queue.to_string()).or_insert_with(Vec::new);
        q.insert(0, value.to_string());
        Ok(())
    }

    pub async fn rpop(&self, queue: &str) -> Result<Option<String>, ValkeyError> {
        let mut queues = self.queues.write().await;
        if let Some(q) = queues.get_mut(queue) {
            Ok(q.pop())
        } else {
            Ok(None)
        }
    }

    pub async fn llen(&self, queue: &str) -> Result<u64, ValkeyError> {
        let queues = self.queues.read().await;
        Ok(queues.get(queue).map(|q| q.len() as u64).unwrap_or(0))
    }

    // Hash operations
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<(), ValkeyError> {
        let mut hashes = self.hashes.write().await;
        let h = hashes.entry(key.to_string()).or_insert_with(std::collections::HashMap::new);
        h.insert(field.to_string(), value.to_string());
        Ok(())
    }

    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>, ValkeyError> {
        let hashes = self.hashes.read().await;
        Ok(hashes.get(key).and_then(|h| h.get(field).cloned()))
    }

    pub async fn hgetall(&self, key: &str) -> Result<std::collections::HashMap<String, String>, ValkeyError> {
        let hashes = self.hashes.read().await;
        Ok(hashes.get(key).cloned().unwrap_or_default())
    }

    pub async fn hdel(&self, key: &str, field: &str) -> Result<(), ValkeyError> {
        let mut hashes = self.hashes.write().await;
        if let Some(h) = hashes.get_mut(key) {
            h.remove(field);
        }
        Ok(())
    }

    // Utility
    pub async fn flush(&self) -> Result<(), ValkeyError> {
        self.memory.write().await.clear();
        self.queues.write().await.clear();
        self.hashes.write().await.clear();
        Ok(())
    }

    pub async fn len(&self) -> usize {
        self.memory.read().await.len()
    }
}

pub type SharedValkeyStore = Arc<ValkeyStore>;
