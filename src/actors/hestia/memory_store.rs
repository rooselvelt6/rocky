// src/actors/hestia/memory_store.rs
// OLYMPUS v13 - Hestia Memory Store
// Almacenamiento en Valkey

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use super::ValkeyStore;
use crate::errors::PersistenceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredItem {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl StoredItem {
    pub fn new(key: String, value: serde_json::Value, ttl_seconds: Option<u64>) -> Self {
        let now = chrono::Utc::now();
        Self {
            key,
            value,
            created_at: now,
            expires_at: ttl_seconds.map(|s| now + chrono::Duration::seconds(s as i64)),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStore {
    valkey: Arc<ValkeyStore>,
    prefix: String,
}

impl MemoryStore {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self {
            valkey,
            prefix: "olympus:hestia:store".to_string(),
        }
    }
    
    pub async fn set(&self, key: &str, value: &serde_json::Value, ttl_seconds: Option<u64>) {
        let item = StoredItem::new(key.to_string(), value.clone(), ttl_seconds);
        let json = serde_json::to_string(&item).unwrap_or_default();
        self.valkey.set(&format!("{}:{}", self.prefix, key), &json).await.ok();
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, PersistenceError> {
        if let Some(json) = self.valkey.get(&format!("{}:{}", self.prefix, key)).await.ok().flatten() {
            if let Ok(item) = serde_json::from_str::<StoredItem>(&json) {
                if !item.is_expired() {
                    return Ok(Some(item.value));
                }
            }
        }
        Ok(None)
    }
    
    pub async fn delete(&self, key: &str) {
        self.valkey.del(&format!("{}:{}", self.prefix, key)).await.ok();
    }
    
    pub async fn exists(&self, key: &str) -> bool {
        self.valkey.exists(&format!("{}:{}", self.prefix, key)).await.ok().unwrap_or(false)
    }
    
    pub async fn clear(&self) {
        self.valkey.flush().await.ok();
    }
}
