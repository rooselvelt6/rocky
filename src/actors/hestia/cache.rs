// src/actors/hestia/cache.rs
// OLYMPUS v13 - Hestia Cache Manager
// Cache de datos frecuentes

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use super::ValkeyStore;
use crate::errors::PersistenceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub ttl_seconds: u64,
}

impl CacheEntry {
    pub fn new(key: String, value: serde_json::Value, ttl_seconds: u64) -> Self {
        let now = chrono::Utc::now();
        Self {
            key,
            value,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            ttl_seconds,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.created_at + chrono::Duration::seconds(self.ttl_seconds as i64)
    }
}

#[derive(Debug, Clone)]
pub struct CacheManager {
    valkey: Arc<ValkeyStore>,
    prefix: String,
}

impl CacheManager {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self {
            valkey,
            prefix: "olympus:hestia:cache".to_string(),
        }
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, PersistenceError> {
        if let Some(json) = self.valkey.hget(&self.prefix, key).await.ok().flatten() {
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(&json) {
                if !entry.is_expired() {
                    return Ok(Some(entry.value));
                }
            }
        }
        Ok(None)
    }
    
    pub async fn set(&self, key: &str, value: &serde_json::Value, ttl_seconds: u64) {
        let entry = CacheEntry::new(key.to_string(), value.clone(), ttl_seconds);
        let json = serde_json::to_string(&entry).unwrap_or_default();
        self.valkey.hset(&self.prefix, key, &json).await.ok();
    }
    
    pub async fn invalidate(&self, key: &str) {
        self.valkey.hdel(&self.prefix, key).await.ok();
    }
    
    pub async fn size(&self) -> usize {
        let hash = self.valkey.hgetall(&self.prefix).await.ok().unwrap_or_default();
        hash.len()
    }
    
    pub async fn clear(&self) {
        self.valkey.del(&self.prefix).await.ok();
    }
}
