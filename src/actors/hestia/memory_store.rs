// src/actors/hestia/memory_store.rs
// OLYMPUS v15 - Hestia Memory Store
// Almacenamiento en Valkey con LRU eviction y estadísticas avanzadas

use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::infrastructure::ValkeyStore;
use crate::errors::PersistenceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredItem {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub size_bytes: usize,
}

impl StoredItem {
    pub fn new(key: String, value: serde_json::Value, ttl_seconds: Option<u64>) -> Self {
        let now = chrono::Utc::now();
        let size_bytes = serde_json::to_string(&value).unwrap_or_default().len();
        
        Self {
            key,
            value,
            created_at: now,
            expires_at: ttl_seconds.map(|s| now + chrono::Duration::seconds(s as i64)),
            last_accessed: now,
            access_count: 0,
            size_bytes,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }
    
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
        self.access_count += 1;
    }
}

/// Estadísticas del MemoryStore
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStoreStats {
    pub total_items: usize,
    pub total_size_bytes: usize,
    pub evicted_count: u64,
    pub expired_count: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub last_eviction_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl MemoryStoreStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStoreConfig {
    pub max_items: usize,
    pub max_size_bytes: usize,
    pub default_ttl_seconds: Option<u64>,
    pub eviction_policy: EvictionPolicy,
}

impl Default for MemoryStoreConfig {
    fn default() -> Self {
        Self {
            max_items: 10000,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl_seconds: None,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,           // Least Recently Used
    LFU,           // Least Frequently Used
    FIFO,          // First In First Out
    TTL,           // Time To Live priority
    Random,        // Random eviction
}

/// LRU Cache con soporte para TTL y estadísticas
#[derive(Debug)]
pub struct MemoryStore {
    valkey: Arc<ValkeyStore>,
    prefix: String,
    config: MemoryStoreConfig,
    
    // LRU tracking
    lru_order: RwLock<VecDeque<String>>,
    
    // Estadísticas
    stats: RwLock<MemoryStoreStats>,
}

impl MemoryStore {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self::with_config(valkey, MemoryStoreConfig::default())
    }
    
    pub fn with_config(valkey: Arc<ValkeyStore>, config: MemoryStoreConfig) -> Self {
        Self {
            valkey,
            prefix: "olympus:hestia:store".to_string(),
            config,
            lru_order: RwLock::new(VecDeque::new()),
            stats: RwLock::new(MemoryStoreStats::default()),
        }
    }
    
    fn full_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }
    
    /// Almacena un valor con TTL opcional
    pub async fn set(
        &self, 
        key: &str, 
        value: &serde_json::Value, 
        ttl_seconds: Option<u64>
    ) -> Result<(), PersistenceError> {
        let ttl = ttl_seconds.or(self.config.default_ttl_seconds);
        let item = StoredItem::new(key.to_string(), value.clone(), ttl);
        let json = serde_json::to_string(&item)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        
        // Verificar si necesitamos eviction
        self.maybe_evict().await?;
        
        // Almacenar en Valkey
        self.valkey.set(&self.full_key(key), &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Actualizar LRU order
        let mut lru = self.lru_order.write().await;
        lru.retain(|k| k != key);
        lru.push_front(key.to_string());
        drop(lru);
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.total_items += 1;
        stats.total_size_bytes += item.size_bytes;
        drop(stats);
        
        debug!("Stored item '{}' ({} bytes)", key, item.size_bytes);
        Ok(())
    }
    
    /// Recupera un valor
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, PersistenceError> {
        let result = self.valkey.get(&self.full_key(key)).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        match result {
            Some(json) => {
                let mut item: StoredItem = serde_json::from_str(&json)
                    .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?;
                
                if item.is_expired() {
                    // Eliminar item expirado
                    self.delete(key).await?;
                    
                    let mut stats = self.stats.write().await;
                    stats.expired_count += 1;
                    stats.miss_count += 1;
                    drop(stats);
                    
                    return Ok(None);
                }
                
                // Actualizar access stats
                item.touch();
                let updated_json = serde_json::to_string(&item)
                    .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
                self.valkey.set(&self.full_key(key), &updated_json).await
                    .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
                
                // Mover al frente de LRU
                let mut lru = self.lru_order.write().await;
                lru.retain(|k| k != key);
                lru.push_front(key.to_string());
                drop(lru);
                
                // Actualizar estadísticas
                let mut stats = self.stats.write().await;
                stats.hit_count += 1;
                drop(stats);
                
                debug!("Cache hit for key '{}' (access count: {})", key, item.access_count);
                Ok(Some(item.value))
            }
            None => {
                let mut stats = self.stats.write().await;
                stats.miss_count += 1;
                drop(stats);
                
                debug!("Cache miss for key '{}'", key);
                Ok(None)
            }
        }
    }
    
    /// Elimina un valor
    pub async fn delete(&self, key: &str) -> Result<(), PersistenceError> {
        // Obtener el item primero para actualizar estadísticas
        if let Ok(Some(json)) = self.valkey.get(&self.full_key(key)).await {
            if let Ok(item) = serde_json::from_str::<StoredItem>(&json) {
                let mut stats = self.stats.write().await;
                stats.total_items = stats.total_items.saturating_sub(1);
                stats.total_size_bytes = stats.total_size_bytes.saturating_sub(item.size_bytes);
                drop(stats);
            }
        }
        
        // Eliminar de Valkey
        self.valkey.del(&self.full_key(key)).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Eliminar de LRU order
        let mut lru = self.lru_order.write().await;
        lru.retain(|k| k != key);
        drop(lru);
        
        debug!("Deleted item '{}'", key);
        Ok(())
    }
    
    /// Verifica si una clave existe
    pub async fn exists(&self, key: &str) -> Result<bool, PersistenceError> {
        match self.get(key).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    
    /// Limpia todos los datos
    pub async fn clear(&self) -> Result<(), PersistenceError> {
        // Obtener todas las claves con el prefijo y eliminarlas
        let pattern = format!("{}:*", self.prefix);
        self.valkey.flush().await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Limpiar LRU order
        let mut lru = self.lru_order.write().await;
        lru.clear();
        drop(lru);
        
        // Resetear estadísticas
        let mut stats = self.stats.write().await;
        *stats = MemoryStoreStats::default();
        drop(stats);
        
        info!("MemoryStore cleared");
        Ok(())
    }
    
    /// Obtiene múltiples valores por patrón (simulado con prefijo)
    pub async fn get_all(&self, pattern: &str) -> Result<HashMap<String, serde_json::Value>, PersistenceError> {
        // En una implementación real, usaría SCAN de Redis/Valkey
        // Por ahora, simulamos con un hash getall
        let all_data = self.valkey.hgetall(&self.prefix).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        let mut result = HashMap::new();
        for (key, json) in all_data {
            if key.contains(pattern) || pattern == "*" {
                if let Ok(item) = serde_json::from_str::<StoredItem>(&json) {
                    if !item.is_expired() {
                        result.insert(key, item.value);
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Obtiene las claves más recientemente usadas
    pub async fn get_lru_keys(&self, limit: usize) -> Vec<String> {
        let lru = self.lru_order.read().await;
        lru.iter().take(limit).cloned().collect()
    }
    
    /// Obtiene estadísticas del store
    pub async fn get_stats(&self) -> MemoryStoreStats {
        self.stats.read().await.clone()
    }
    
    /// Actualiza configuración
    pub async fn update_config(&mut self, config: MemoryStoreConfig) {
        self.config = config;
        info!("MemoryStore config updated: max_items={}, max_size={}MB", 
            self.config.max_items, 
            self.config.max_size_bytes / 1024 / 1024);
    }
    
    /// Verifica si se necesita eviction y lo ejecuta
    async fn maybe_evict(&self) -> Result<(), PersistenceError> {
        let stats = self.stats.read().await.clone();
        
        let needs_eviction = stats.total_items >= self.config.max_items ||
                            stats.total_size_bytes >= self.config.max_size_bytes;
        
        if !needs_eviction {
            return Ok(());
        }
        
        let victim = match self.config.eviction_policy {
            EvictionPolicy::LRU => self.find_lru_victim().await,
            EvictionPolicy::FIFO => self.find_fifo_victim().await,
            EvictionPolicy::Random => self.find_random_victim().await,
            _ => self.find_lru_victim().await, // LRU como default
        };
        
        if let Some(key) = victim {
            warn!("Evicting key '{}' (policy: {:?})", key, self.config.eviction_policy);
            self.delete(&key).await?;
            
            let mut stats = self.stats.write().await;
            stats.evicted_count += 1;
            stats.last_eviction_time = Some(chrono::Utc::now());
        }
        
        Ok(())
    }
    
    async fn find_lru_victim(&self) -> Option<String> {
        let lru = self.lru_order.read().await;
        lru.back().cloned()
    }
    
    async fn find_fifo_victim(&self) -> Option<String> {
        let lru = self.lru_order.read().await;
        lru.back().cloned()
    }
    
    async fn find_random_victim(&self) -> Option<String> {
        let lru = self.lru_order.read().await;
        if lru.is_empty() {
            return None;
        }
        use rand::seq::SliceRandom;
        lru.iter().collect::<Vec<_>>().choose(&mut rand::thread_rng()).map(|s| s.to_string())
    }
    
    /// Limpia items expirados
    pub async fn cleanup_expired(&self) -> Result<u64, PersistenceError> {
        let lru = self.lru_order.read().await;
        let keys: Vec<String> = lru.iter().cloned().collect();
        drop(lru);
        
        let mut cleaned = 0u64;
        for key in keys {
            if let Ok(Some(json)) = self.valkey.get(&self.full_key(&key)).await {
                if let Ok(item) = serde_json::from_str::<StoredItem>(&json) {
                    if item.is_expired() {
                        self.delete(&key).await?;
                        cleaned += 1;
                    }
                }
            }
        }
        
        if cleaned > 0 {
            info!("Cleaned up {} expired items", cleaned);
            let mut stats = self.stats.write().await;
            stats.expired_count += cleaned;
        }
        
        Ok(cleaned)
    }
    
    /// Prefetch de múltiples claves
    pub async fn prefetch(&self, keys: &[String]) -> Result<HashMap<String, serde_json::Value>, PersistenceError> {
        let mut result = HashMap::new();
        for key in keys {
            if let Ok(Some(value)) = self.get(key).await {
                result.insert(key.clone(), value);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_store_basic() {
        let valkey = Arc::new(ValkeyStore::default());
        let store = MemoryStore::new(valkey);
        
        // Test set y get
        let value = serde_json::json!({"test": "value"});
        store.set("key1", &value, None).await.unwrap();
        
        let retrieved = store.get("key1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
        
        // Test delete
        store.delete("key1").await.unwrap();
        let deleted = store.get("key1").await.unwrap();
        assert!(deleted.is_none());
    }
    
    #[tokio::test]
    async fn test_memory_store_expiration() {
        let valkey = Arc::new(ValkeyStore::default());
        let store = MemoryStore::new(valkey);
        
        let value = serde_json::json!({"test": "value"});
        store.set("key1", &value, Some(1)).await.unwrap();
        
        // Inmediatamente debería existir
        let exists = store.exists("key1").await.unwrap();
        assert!(exists);
        
        // Esperar 2 segundos
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Ahora debería estar expirado
        let expired = store.get("key1").await.unwrap();
        assert!(expired.is_none());
    }
}
