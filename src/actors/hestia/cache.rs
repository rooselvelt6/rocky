// src/actors/hestia/cache.rs
// OLYMPUS v15 - Hestia Cache Manager
// Cache multi-nivel con políticas avanzadas: TTL, LRU, LFU, Write-Through, Write-Behind

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, info, warn};

use crate::infrastructure::ValkeyStore;
use crate::errors::PersistenceError;
use crate::actors::hestia::sync::SyncManager;

pub use tokio::time::Duration;

/// Política de escritura en cache
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WritePolicy {
    WriteThrough,  // Escribe en cache y persistencia al mismo tiempo
    WriteBehind,   // Escribe en cache, luego async a persistencia
    WriteAround,   // Escribe solo en persistencia, invalida cache
}

/// Política de lectura en cache
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReadPolicy {
    CacheAside,    // Lee cache primero, si no existe lee persistencia
    ReadThrough,   // Cache se encarga de leer de persistencia si no existe
    RefreshAhead,  // Precarga datos próximos a expirar
}

/// Nivel de cache (Multi-Level Caching)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheLevel {
    L1, // Hot cache - memoria local ultra-rápida (siempre en Valkey hash)
    L2, // Warm cache - Valkey (keys individuales)
    L3, // Cold storage - SurrealDB (persistencia real)
}

/// Metadata de una entrada de cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub level: CacheLevel,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub ttl_seconds: Option<u64>,
    pub size_bytes: usize,
    pub tags: HashSet<String>,
    pub write_count: u64,
    pub dirty: bool,  // Para write-behind
}

impl CacheEntry {
    pub fn new(
        key: String, 
        value: serde_json::Value, 
        level: CacheLevel,
        ttl_seconds: Option<u64>,
        tags: HashSet<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        let size_bytes = serde_json::to_string(&value).unwrap_or_default().len();
        
        Self {
            key,
            value,
            level,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            ttl_seconds,
            size_bytes,
            tags,
            write_count: 0,
            dirty: false,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let expiry = self.created_at + chrono::Duration::seconds(ttl as i64);
            chrono::Utc::now() > expiry
        } else {
            false
        }
    }
    
    pub fn touch(&mut self) {
        self.accessed_at = chrono::Utc::now();
        self.access_count += 1;
    }
    
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.write_count += 1;
    }
    
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Estadísticas del cache
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub total_evictions: u64,
    pub total_writes: u64,
    pub dirty_entries: u64,
    pub memory_used_bytes: usize,
    pub entry_count: usize,
}

impl CacheStats {
    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total == 0 { 0.0 } else { self.l1_hits as f64 / total as f64 }
    }
    
    pub fn l2_hit_rate(&self) -> f64 {
        let total = self.l2_hits + self.l2_misses;
        if total == 0 { 0.0 } else { self.l2_hits as f64 / total as f64 }
    }
    
    pub fn overall_hit_rate(&self) -> f64 {
        let hits = self.l1_hits + self.l2_hits;
        let misses = self.l1_misses + self.l2_misses;
        let total = hits + misses;
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}

/// Configuración del cache manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub l1_max_size: usize,
    pub l2_max_size: usize,
    pub default_ttl_seconds: Option<u64>,
    pub write_policy: WritePolicy,
    pub read_policy: ReadPolicy,
    pub enable_compression: bool,
    pub compression_threshold_bytes: usize,
    pub refresh_ahead_interval_secs: u64,
    pub write_behind_interval_secs: u64,
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 1000,
            l2_max_size: 10000,
            default_ttl_seconds: Some(3600), // 1 hora default
            write_policy: WritePolicy::WriteThrough,
            read_policy: ReadPolicy::CacheAside,
            enable_compression: false,
            compression_threshold_bytes: 1024,
            refresh_ahead_interval_secs: 60,
            write_behind_interval_secs: 30,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// Política de evicción
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    Random,
    TTLPriority,
}

/// Cache Manager multi-nivel
#[derive(Debug)]
pub struct CacheManager {
    valkey: Arc<ValkeyStore>,
    sync_manager: Option<Arc<SyncManager>>,
    
    // L1 Cache: Hash en memoria (ultra rápido)
    l1_cache: RwLock<HashMap<String, CacheEntry>>,
    
    // L2 Cache: Valkey (persistido rápido)
    l2_prefix: String,
    
    // Configuración
    config: RwLock<CacheConfig>,
    
    // Estadísticas
    stats: RwLock<CacheStats>,
    
    // LRU tracking para L1
    l1_lru: Mutex<VecDeque<String>>,
    
    // Background tasks control
    refresh_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    write_behind_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl CacheManager {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self::with_config(valkey, CacheConfig::default())
    }
    
    pub fn with_config(valkey: Arc<ValkeyStore>, config: CacheConfig) -> Self {
        Self {
            valkey,
            sync_manager: None,
            l1_cache: RwLock::new(HashMap::new()),
            l2_prefix: "olympus:hestia:cache:l2".to_string(),
            config: RwLock::new(config),
            stats: RwLock::new(CacheStats::default()),
            l1_lru: Mutex::new(VecDeque::new()),
            refresh_handle: Mutex::new(None),
            write_behind_handle: Mutex::new(None),
        }
    }
    
    pub fn with_sync_manager(mut self, sync_manager: Arc<SyncManager>) -> Self {
        self.sync_manager = Some(sync_manager);
        self
    }
    
    /// Inicia tareas en background (refresh-ahead, write-behind)
    pub async fn start_background_tasks(&self) {
        let config = self.config.read().await.clone();
        
        // Refresh-ahead task
        if config.read_policy == ReadPolicy::RefreshAhead {
            let this = Arc::new(self.clone());
            let handle = tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(config.refresh_ahead_interval_secs));
                loop {
                    interval.tick().await;
                    if let Err(e) = this.refresh_expiring_entries().await {
                        warn!("Refresh-ahead error: {}", e);
                    }
                }
            });
            *self.refresh_handle.lock().await = Some(handle);
        }
        
        // Write-behind task
        if config.write_policy == WritePolicy::WriteBehind {
            let this = Arc::new(self.clone());
            let handle = tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(config.write_behind_interval_secs));
                loop {
                    interval.tick().await;
                    if let Err(e) = this.flush_dirty_entries().await {
                        warn!("Write-behind error: {}", e);
                    }
                }
            });
            *self.write_behind_handle.lock().await = Some(handle);
        }
        
        info!("CacheManager background tasks started");
    }
    
    /// Obtiene un valor del cache (L1 -> L2 -> L3)
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, PersistenceError> {
        // L1 Cache (memoria local)
        {
            let l1 = self.l1_cache.read().await;
            if let Some(entry) = l1.get(key) {
                if !entry.is_expired() {
                    let mut stats = self.stats.write().await;
                    stats.l1_hits += 1;
                    drop(stats);
                    
                    // Actualizar LRU
                    let mut lru = self.l1_lru.lock().await;
                    lru.retain(|k| k != key);
                    lru.push_front(key.to_string());
                    
                    debug!("L1 cache hit for key '{}'", key);
                    return Ok(Some(entry.value.clone()));
                }
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.l1_misses += 1;
        drop(stats);
        
        // L2 Cache (Valkey)
        let l2_key = format!("{}:{}", self.l2_prefix, key);
        if let Ok(Some(json)) = self.valkey.get(&l2_key).await {
            let entry: CacheEntry = serde_json::from_str(&json)
                .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?;
            
            if !entry.is_expired() {
                // Promover a L1
                self.promote_to_l1(entry.clone()).await?;
                
                let mut stats = self.stats.write().await;
                stats.l2_hits += 1;
                drop(stats);
                
                debug!("L2 cache hit for key '{}'", key);
                return Ok(Some(entry.value));
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.l2_misses += 1;
        drop(stats);
        
        // Read-through: si hay sync_manager, intentar recuperar de L3
        if let Some(ref sync) = self.sync_manager {
            if let Ok(Some(value)) = sync.fetch_from_l3(key).await {
                // Cargar en cache
                self.set(key, &value, None, HashSet::new()).await?;
                
                let mut stats = self.stats.write().await;
                stats.l3_hits += 1;
                drop(stats);
                
                debug!("L3 cache hit for key '{}' (read-through)", key);
                return Ok(Some(value));
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.l3_misses += 1;
        drop(stats);
        
        debug!("Cache miss for key '{}'", key);
        Ok(None)
    }
    
    /// Almacena un valor en el cache
    pub async fn set(
        &self, 
        key: &str, 
        value: &serde_json::Value,
        ttl_seconds: Option<u64>,
        tags: HashSet<String>,
    ) -> Result<(), PersistenceError> {
        let config = self.config.read().await.clone();
        let ttl = ttl_seconds.or(config.default_ttl_seconds);
        
        let entry = CacheEntry::new(
            key.to_string(),
            value.clone(),
            CacheLevel::L1,
            ttl,
            tags,
        );
        
        // Escribir según política
        match config.write_policy {
            WritePolicy::WriteThrough => {
                // Escribir en L1, L2 y L3 (si hay sync)
                self.write_to_l1(entry.clone()).await?;
                self.write_to_l2(&entry).await?;
                
                if let Some(ref sync) = self.sync_manager {
                    sync.sync_to_l3(key, value).await?;
                }
            }
            WritePolicy::WriteBehind => {
                // Escribir en L1 y L2, marcar como dirty para L3
                let mut entry = entry;
                entry.mark_dirty();
                self.write_to_l1(entry.clone()).await?;
                self.write_to_l2(&entry).await?;
            }
            WritePolicy::WriteAround => {
                // Escribir solo en L3, invalidar cache
                if let Some(ref sync) = self.sync_manager {
                    sync.sync_to_l3(key, value).await?;
                }
                self.invalidate(key).await?;
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.total_writes += 1;
        drop(stats);
        
        debug!("Cache set for key '{}' (write policy: {:?})", key, config.write_policy);
        Ok(())
    }
    
    /// Invalida una entrada de cache
    pub async fn invalidate(&self, key: &str) -> Result<(), PersistenceError> {
        // Eliminar de L1
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(entry) = l1.remove(key) {
                let mut stats = self.stats.write().await;
                stats.memory_used_bytes = stats.memory_used_bytes.saturating_sub(entry.size_bytes);
                stats.entry_count = stats.entry_count.saturating_sub(1);
            }
        }
        
        // Eliminar de LRU
        {
            let mut lru = self.l1_lru.lock().await;
            lru.retain(|k| k != key);
        }
        
        // Eliminar de L2
        let l2_key = format!("{}:{}", self.l2_prefix, key);
        self.valkey.del(&l2_key).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        debug!("Cache invalidated for key '{}'", key);
        Ok(())
    }
    
    /// Invalida entradas por tag
    pub async fn invalidate_by_tag(&self, tag: &str) -> Result<u64, PersistenceError> {
        let mut count = 0u64;
        
        // Buscar en L1
        {
            let l1 = self.l1_cache.read().await;
            let keys_to_invalidate: Vec<String> = l1
                .values()
                .filter(|e| e.tags.contains(tag))
                .map(|e| e.key.clone())
                .collect();
            drop(l1);
            
            for key in keys_to_invalidate {
                self.invalidate(&key).await?;
                count += 1;
            }
        }
        
        debug!("Invalidated {} entries with tag '{}'", count, tag);
        Ok(count)
    }
    
    /// Limpia todo el cache
    pub async fn clear(&self) -> Result<(), PersistenceError> {
        // Limpiar L1
        {
            let mut l1 = self.l1_cache.write().await;
            l1.clear();
        }
        
        // Limpiar LRU
        {
            let mut lru = self.l1_lru.lock().await;
            lru.clear();
        }
        
        // Limpiar L2 (eliminar por prefijo)
        let _ = self.valkey.hgetall(&self.l2_prefix).await;
        
        // Resetear estadísticas
        {
            let mut stats = self.stats.write().await;
            *stats = CacheStats::default();
        }
        
        info!("Cache cleared");
        Ok(())
    }
    
    /// Obtiene estadísticas
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
    
    /// Obtiene el tamaño actual del cache L1
    pub async fn size(&self) -> usize {
        self.l1_cache.read().await.len()
    }
    
    /// Verifica si una clave existe en cache
    pub async fn exists(&self, key: &str) -> bool {
        // Verificar L1
        {
            let l1 = self.l1_cache.read().await;
            if let Some(entry) = l1.get(key) {
                return !entry.is_expired();
            }
        }
        
        // Verificar L2
        let l2_key = format!("{}:{}", self.l2_prefix, key);
        if let Ok(Some(json)) = self.valkey.get(&l2_key).await {
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(&json) {
                return !entry.is_expired();
            }
        }
        
        false
    }
    
    /// Obtiene múltiples valores por patrón de tag
    pub async fn get_by_tag(&self, tag: &str) -> HashMap<String, serde_json::Value> {
        let l1 = self.l1_cache.read().await;
        let mut result = HashMap::new();
        
        for (key, entry) in l1.iter() {
            if entry.tags.contains(tag) && !entry.is_expired() {
                result.insert(key.clone(), entry.value.clone());
            }
        }
        
        result
    }
    
    /// Precarga múltiples valores
    pub async fn prefetch(&self, keys: &[String]) -> Result<usize, PersistenceError> {
        let mut loaded = 0;
        
        for key in keys {
            if self.exists(key).await {
                continue; // Ya está en cache
            }
            
            // Intentar cargar de L3
            if let Some(ref sync) = self.sync_manager {
                if let Ok(Some(value)) = sync.fetch_from_l3(key).await {
                    self.set(key, &value, None, HashSet::new()).await?;
                    loaded += 1;
                }
            }
        }
        
        info!("Prefetched {} keys into cache", loaded);
        Ok(loaded)
    }
    
    // Métodos privados de ayuda
    
    async fn write_to_l1(&self, entry: CacheEntry) -> Result<(), PersistenceError> {
        let config = self.config.read().await.clone();
        
        // Verificar si necesitamos evicción
        self.maybe_evict_l1(config.l1_max_size).await?;
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.memory_used_bytes += entry.size_bytes;
        stats.entry_count += 1;
        drop(stats);
        
        // Escribir en L1
        let mut l1 = self.l1_cache.write().await;
        l1.insert(entry.key.clone(), entry.clone());
        drop(l1);
        
        // Actualizar LRU
        let mut lru = self.l1_lru.lock().await;
        lru.retain(|k| k != &entry.key);
        lru.push_front(entry.key);
        
        Ok(())
    }
    
    async fn write_to_l2(&self, entry: &CacheEntry) -> Result<(), PersistenceError> {
        let l2_key = format!("{}:{}", self.l2_prefix, &entry.key);
        let json = serde_json::to_string(entry)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        
        self.valkey.set(&l2_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn promote_to_l1(&self, entry: CacheEntry) -> Result<(), PersistenceError> {
        self.write_to_l1(entry).await
    }
    
    async fn maybe_evict_l1(&self, max_size: usize) -> Result<(), PersistenceError> {
        let current_size = self.l1_cache.read().await.len();
        
        if current_size < max_size {
            return Ok(());
        }
        
        let config = self.config.read().await.clone();
        let victim = match config.eviction_policy {
            EvictionPolicy::LRU => self.find_lru_victim().await,
            EvictionPolicy::LFU => self.find_lfu_victim().await,
            EvictionPolicy::FIFO => self.find_lru_victim().await, // LRU es similar a FIFO en VecDeque
            EvictionPolicy::Random => self.find_random_victim().await,
            EvictionPolicy::TTLPriority => self.find_ttl_victim().await,
        };
        
        if let Some(key) = victim {
            warn!("Evicting L1 cache entry '{}' (policy: {:?})", key, config.eviction_policy);
            
            // Eliminar de L1
            {
                let mut l1 = self.l1_cache.write().await;
                if let Some(entry) = l1.remove(&key) {
                    let mut stats = self.stats.write().await;
                    stats.memory_used_bytes = stats.memory_used_bytes.saturating_sub(entry.size_bytes);
                    stats.entry_count = stats.entry_count.saturating_sub(1);
                    stats.total_evictions += 1;
                }
            }
            
            // Eliminar de LRU
            let mut lru = self.l1_lru.lock().await;
            lru.retain(|k| k != &key);
        }
        
        Ok(())
    }
    
    async fn find_lru_victim(&self) -> Option<String> {
        let lru = self.l1_lru.lock().await;
        lru.back().cloned()
    }
    
    async fn find_lfu_victim(&self) -> Option<String> {
        let l1 = self.l1_cache.read().await;
        l1.values()
            .min_by_key(|e| e.access_count)
            .map(|e| e.key.clone())
    }
    
    async fn find_random_victim(&self) -> Option<String> {
        let l1 = self.l1_cache.read().await;
        if l1.is_empty() {
            return None;
        }
        use rand::seq::IteratorRandom;
        l1.keys().choose(&mut rand::thread_rng()).cloned()
    }
    
    async fn find_ttl_victim(&self) -> Option<String> {
        let l1 = self.l1_cache.read().await;
        l1.values()
            .filter(|e| e.ttl_seconds.is_some())
            .min_by_key(|e| e.created_at + chrono::Duration::seconds(e.ttl_seconds.unwrap() as i64))
            .map(|e| e.key.clone())
    }
    
    async fn refresh_expiring_entries(&self) -> Result<(), PersistenceError> {
        let l1 = self.l1_cache.read().await;
        let expiring: Vec<String> = l1
            .values()
            .filter(|e| {
                if let Some(ttl) = e.ttl_seconds {
                    let expiry = e.created_at + chrono::Duration::seconds(ttl as i64);
                    let refresh_threshold = chrono::Utc::now() + chrono::Duration::seconds(60);
                    expiry < refresh_threshold && !e.is_expired()
                } else {
                    false
                }
            })
            .map(|e| e.key.clone())
            .collect();
        drop(l1);
        
        for key in expiring {
            // Refrescar TTL re-escribiendo
            if let Ok(Some(value)) = self.get(&key).await {
                if let Err(e) = self.set(&key, &value, None, HashSet::new()).await {
                    warn!("Failed to refresh entry '{}': {}", key, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn flush_dirty_entries(&self) -> Result<(), PersistenceError> {
        let dirty_entries: Vec<CacheEntry> = {
            let l1 = self.l1_cache.read().await;
            l1.values()
                .filter(|e| e.dirty)
                .cloned()
                .collect()
        };
        
        if dirty_entries.is_empty() {
            return Ok(());
        }
        
        info!("Flushing {} dirty entries to L3", dirty_entries.len());
        
        if let Some(ref sync) = self.sync_manager {
            for entry in dirty_entries {
                if let Err(e) = sync.sync_to_l3(&entry.key, &entry.value).await {
                    warn!("Failed to sync dirty entry '{}': {}", entry.key, e);
                    continue;
                }
                
                // Marcar como limpio
                let mut l1 = self.l1_cache.write().await;
                if let Some(e) = l1.get_mut(&entry.key) {
                    e.mark_clean();
                }
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.dirty_entries = 0;
        
        Ok(())
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        // Nota: esto es una simplificación. En producción, usarías Arc para todo.
        panic!("CacheManager should not be cloned directly. Use Arc<CacheManager> instead.")
    }
}
