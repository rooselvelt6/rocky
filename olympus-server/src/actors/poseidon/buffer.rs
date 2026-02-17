// src/actors/poseidon/buffer.rs
// OLYMPUS v15 - Poseidon Emergency Buffer
// Buffer de emergencia cuando la red falla (Valkey embedded)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::ValkeyStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedData {
    pub id: String,
    pub domain: super::DivineDomain,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub attempts: u32,
}

impl BufferedData {
    pub fn new(domain: super::DivineDomain, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            domain,
            data,
            timestamp: Utc::now(),
            attempts: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmergencyBuffer {
    valkey: Arc<ValkeyStore>,
    buffer_key: String,
    max_size: usize,
}

impl EmergencyBuffer {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self {
            valkey,
            buffer_key: "olympus:poseidon:buffer".to_string(),
            max_size: 10000,
        }
    }
    
    pub async fn buffer(&self, domain: super::DivineDomain, data: serde_json::Value) -> String {
        let buffered = BufferedData::new(domain, data);
        let json = serde_json::to_string(&buffered).unwrap_or_default();
        let _ = self.valkey.lpush(&self.buffer_key, &json).await;
        buffered.id
    }
    
    pub async fn pop(&self) -> Option<BufferedData> {
        if let Some(json) = self.valkey.rpop(&self.buffer_key).await.ok().flatten() {
            return serde_json::from_str(&json).ok();
        }
        None
    }
    
    pub async fn len(&self) -> usize {
        self.valkey.llen(&self.buffer_key).await.ok().unwrap_or(0) as usize
    }
    
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    pub async fn clear(&self) {
        let _ = self.valkey.flush().await;
    }
    
    pub async fn increment_attempts(&self, id: &str) {
        // En una implementación real, se actualizaría el item en Valkey
        let _ = id;
    }
}
