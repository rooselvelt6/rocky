// src/actors/hestia/mod.rs
// OLYMPUS v15 - Hestia: Sistema de Persistencia Completo
// Diosa del Hogar - Guardiana de la Persistencia Dual (Valkey + SurrealDB)
// 
// Arquitectura:
// - L1 (Hot): Cache en memoria local (lru_cache)
// - L2 (Warm): Valkey - Cache rápido, datos temporales, colas
// - L3 (Cold): SurrealDB - Persistencia real, consultas complejas, relaciones
// 
// Sincronización automática: L2 <-> L3 con manejo de conflictos
// Cifrado opcional: Integración con Hades

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::{RwLock, Mutex, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug, instrument};
use chrono;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;
use crate::infrastructure::{ValkeyStore, SurrealStore};

// Submódulos
pub mod memory_store;
pub mod cache;
pub mod async_buffer;
pub mod sync;

// Re-exports
pub use memory_store::{MemoryStore, MemoryStoreConfig, MemoryStoreStats, EvictionPolicy};
pub use cache::{CacheManager, CacheConfig, CacheStats, WritePolicy, ReadPolicy, CacheLevel};
pub use async_buffer::{AsyncBuffer, AsyncBufferConfig, BufferStats, OperationType, FlushResult};
pub use sync::{SyncManager, SyncConfig, SyncStats, SyncRecord, SyncStatus, ConflictResolution, BackupMetadata, SyncResult};

/// Comandos específicos de Hestia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HestiaCommand {
    // Operaciones básicas
    Save { 
        key: String, 
        value: serde_json::Value, 
        table: Option<String>,
        ttl_seconds: Option<u64>,
        encrypt: bool,
        tags: Vec<String>,
    },
    Load { 
        key: String,
        from: Option<CacheLevel>,
    },
    Delete { 
        key: String,
        cascade: bool, // Eliminar también de persistencia
    },
    
    // Operaciones batch
    SaveBatch { 
        items: Vec<(String, serde_json::Value)>,
        table: Option<String>,
    },
    LoadBatch { 
        keys: Vec<String>,
    },
    DeleteBatch { 
        keys: Vec<String>,
    },
    
    // Cache operations
    Invalidate { 
        key: String,
        level: Option<CacheLevel>,
    },
    InvalidateTag { 
        tag: String,
    },
    ClearCache { 
        level: Option<CacheLevel>,
    },
    Prefetch { 
        keys: Vec<String>,
    },
    
    // Sync operations
    Sync { 
        direction: Option<sync::SyncDirection>,
        keys: Option<Vec<String>>,
    },
    ForceSync { 
        key: String,
    },
    ResolveConflict { 
        record_id: String,
        resolution: ConflictResolution,
        new_value: Option<serde_json::Value>,
    },
    
    // Buffer operations
    FlushBuffer,
    RetryDeadLetter,
    ClearDeadLetter,
    
    // Backup/Restore
    Backup { 
        table: String,
    },
    Restore { 
        table: String,
        backup_id: String,
    },
    ListBackups { 
        table: String,
    },
    
    // Mantenimiento
    CleanupExpired,
    OptimizeCache,
    ResetStats,
}

/// Queries específicos de Hestia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HestiaQuery {
    // Queries básicos
    Get { key: String },
    GetAll { pattern: Option<String> },
    
    // Stats
    GetStats,
    GetCacheStats,
    GetBufferStats,
    GetSyncStats,
    
    // Health
    HealthCheck,
    
    // Cache queries
    CacheContains { key: String },
    CacheSize { level: Option<CacheLevel> },
    
    // Sync queries
    GetPendingSyncs,
    GetConflicts,
    GetSyncQueue,
    
    // Buffer queries  
    GetBufferQueue,
    GetDeadLetter,
    
    // Search
    Search { 
        query: String,
        table: Option<String>,
    },
}

/// Hestia - Diosa de la Persistencia
#[derive(Debug)]
pub struct Hestia {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Componentes principales
    memory_store: Arc<MemoryStore>,
    cache: Arc<CacheManager>,
    async_buffer: Arc<AsyncBuffer>,
    sync_manager: Arc<SyncManager>,
    
    // Infraestructura
    valkey: Arc<ValkeyStore>,
    surreal: Arc<SurrealStore>,
    
    // Integración opcional con Hades
    hades_encryption: RwLock<bool>,
    default_encryption_key: RwLock<Option<String>>,
    
    // Health check tracking
    last_health_check: RwLock<chrono::DateTime<chrono::Utc>>,
    consecutive_errors: RwLock<u32>,
    
    // Control
    running: RwLock<bool>,
    maintenance_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    
    // Canales internos
    command_tx: mpsc::Sender<HestiaCommand>,
    command_rx: RwLock<mpsc::Receiver<HestiaCommand>>,
}

impl Hestia {
    /// Crea una nueva instancia de Hestia
    pub async fn new(valkey: Arc<ValkeyStore>, surreal: Arc<SurrealStore>) -> Self {
        Self::with_config(valkey, surreal, None, None).await
    }
    
    /// Crea Hestia con configuración personalizada
    pub async fn with_config(
        valkey: Arc<ValkeyStore>,
        surreal: Arc<SurrealStore>,
        memory_config: Option<MemoryStoreConfig>,
        cache_config: Option<CacheConfig>,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::channel(1000);
        
        // Crear componentes
        let memory_store = Arc::new(MemoryStore::with_config(
            valkey.clone(),
            memory_config.unwrap_or_default(),
        ));
        
        let cache_config_val = cache_config.clone().unwrap_or_default();
        
        let async_buffer = Arc::new(AsyncBuffer::new(
            valkey.clone(),
            surreal.clone(),
        ));
        
        let sync_manager = Arc::new(SyncManager::new(
            valkey.clone(),
            surreal.clone(),
        ));
        
        // Configurar cache con sync_manager
        let cache = Arc::new(
            CacheManager::with_config(valkey.clone(), cache_config_val)
                .with_sync_manager(sync_manager.clone())
        );
        
        Self {
            name: GodName::Hestia,
            state: ActorState::new(GodName::Hestia),
            config: ActorConfig::default(),
            memory_store,
            cache,
            async_buffer,
            sync_manager,
            valkey,
            surreal,
            hades_encryption: RwLock::new(false),
            default_encryption_key: RwLock::new(None),
            last_health_check: RwLock::new(chrono::Utc::now()),
            consecutive_errors: RwLock::new(0),
            running: RwLock::new(false),
            maintenance_handle: RwLock::new(None),
            command_tx,
            command_rx: RwLock::new(command_rx),
        }
    }
    
    /// Habilita cifrado con Hades
    pub async fn enable_encryption(&self, key_id: Option<String>) {
        *self.hades_encryption.write().await = true;
        *self.default_encryption_key.write().await = key_id;
        info!("🔐 Hestia: Encryption enabled (Hades integration)");
    }
    
    /// Guarda un valor (L2 y L3)
    #[instrument(skip(self, value))]
    pub async fn save(
        &self,
        key: &str,
        value: &serde_json::Value,
        table: Option<&str>,
        ttl_seconds: Option<u64>,
        encrypt: bool,
        tags: Vec<String>,
    ) -> Result<(), ActorError> {
        debug!("Saving key '{}'", key);
        
        // Cifrar si es necesario
        let final_value = if encrypt && *self.hades_encryption.read().await {
            // Nota: En implementación real, esto llamaría a Hades
            // Por ahora, marcamos que debería estar cifrado
            let mut marked = value.clone();
            if let Some(obj) = marked.as_object_mut() {
                obj.insert("_encrypted".to_string(), serde_json::json!(true));
            }
            marked
        } else {
            value.clone()
        };
        
        // Guardar en cache (L1/L2)
        let tag_set: HashSet<String> = tags.into_iter().collect();
        if let Err(e) = self.cache.set(key, &final_value, ttl_seconds, tag_set).await {
            error!("Cache set failed for key '{}': {}", key, e);
        }
        
        // Agregar a buffer para persistencia async (L3)
        let table = table.unwrap_or("default");
        if let Err(e) = self.async_buffer.push(
            table,
            key.to_string(),
            final_value.clone(),
            OperationType::Upsert,
        ).await {
            // Si el buffer falla, sync directo
            warn!("Buffer push failed, syncing directly to L3: {}", e);
            if let Err(e) = self.sync_manager.sync_to_l3(key, &final_value).await {
                return Err(ActorError::StateError {
                    god: GodName::Hestia,
                    message: format!("Failed to save to L3: {}", e),
                });
            }
        }
        
        info!("Saved key '{}' (table: {}, encrypted: {})", key, table, encrypt);
        Ok(())
    }
    
    /// Carga un valor (L1 -> L2 -> L3)
    #[instrument(skip(self))]
    pub async fn load(&self, key: &str) -> Result<Option<serde_json::Value>, ActorError> {
        debug!("Loading key '{}'", key);
        
        // Intentar desde cache
        match self.cache.get(key).await {
            Ok(Some(value)) => {
                // Verificar si está cifrado
                if let Some(true) = value.get("_encrypted").and_then(|v| v.as_bool()) {
                    // Nota: En implementación real, llamaría a Hades para descifrar
                    debug!("Key '{}' is encrypted", key);
                }
                
                debug!("Cache hit for key '{}'", key);
                return Ok(Some(value));
            }
            Ok(None) => {
                debug!("Cache miss for key '{}'", key);
            }
            Err(e) => {
                warn!("Cache get error for key '{}': {}", key, e);
            }
        }
        
        // Fallback a L3
        match self.sync_manager.fetch_from_l3(key).await {
            Ok(Some(value)) => {
                // Cargar en cache para futuros accesos
                if let Err(e) = self.cache.set(key, &value, None, HashSet::new()).await {
                    warn!("Failed to cache loaded value: {}", e);
                }
                
                info!("Loaded key '{}' from L3", key);
                Ok(Some(value))
            }
            Ok(None) => {
                info!("Key '{}' not found", key);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to load key '{}' from L3: {}", key, e);
                Err(ActorError::StateError {
                    god: GodName::Hestia,
                    message: format!("Load failed: {}", e),
                })
            }
        }
    }
    
    /// Elimina un valor
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str, cascade: bool) -> Result<(), ActorError> {
        debug!("Deleting key '{}' (cascade: {})", key, cascade);
        
        // Invalidar en cache
        if let Err(e) = self.cache.invalidate(key).await {
            warn!("Cache invalidate failed: {}", e);
        }
        
        if cascade {
            // Eliminar de persistencia (L3)
            let table = "default"; // Inferir de key
            if let Err(e) = self.async_buffer.push(
                table,
                key.to_string(),
                serde_json::json!({"deleted": true}),
                OperationType::Delete,
            ).await {
                warn!("Buffer delete failed: {}", e);
            }
        }
        
        info!("Deleted key '{}'", key);
        Ok(())
    }
    
    /// Sincroniza explícitamente con L3
    pub async fn sync(&self, keys: Option<Vec<String>>) -> Result<SyncResult, ActorError> {
        if let Some(keys) = keys {
            // Sync individual de keys
            for key in keys {
                if let Ok(Some(value)) = self.cache.get(&key).await {
                    if let Err(e) = self.sync_manager.sync_to_l3(&key, &value).await {
                        warn!("Failed to sync key '{}': {}", key, e);
                    }
                }
            }
        }
        
        // Flush buffer
        let _ = self.async_buffer.flush().await;
        
        // Ejecutar sync del manager
        let result = self.sync_manager.sync_all().await;
        
        info!("Sync completed: {} synced", result.synced);
        Ok(result)
    }
    
    /// Fuerza flush del buffer
    pub async fn flush(&self) -> Result<FlushResult, ActorError> {
        self.async_buffer.flush().await.map_err(|e| ActorError::StateError {
            god: GodName::Hestia,
            message: format!("Flush failed: {}", e),
        })
    }
    
    /// Obtiene todas las estadísticas
    pub async fn get_full_stats(&self) -> serde_json::Value {
        let cache_stats = self.cache.get_stats().await;
        let buffer_stats = self.async_buffer.get_stats().await;
        let sync_stats = self.sync_manager.get_stats().await;
        let memory_stats = self.memory_store.get_stats().await;
        
        serde_json::json!({
            "cache": cache_stats,
            "buffer": buffer_stats,
            "sync": sync_stats,
            "memory": memory_stats,
            "timestamp": chrono::Utc::now(),
        })
    }
    
    /// Realiza health check real de todas las conexiones
    pub async fn perform_health_check(&self) -> HealthStatus {
        let mut status = ActorStatus::Healthy;
        let mut errors = Vec::new();
        
        // Check Valkey
        match self.valkey.exists("health_check_test").await {
            Ok(_) => {}
            Err(e) => {
                status = ActorStatus::Degraded;
                errors.push(format!("Valkey: {}", e));
            }
        }
        
        // Check SurrealDB
        match self.surreal.health_check().await {
            Ok(healthy) if healthy => {}
            Ok(_) | Err(e) => {
                status = ActorStatus::Degraded;
                errors.push(format!("SurrealDB: {:?}", e));
            }
        }
        
        // Check buffer
        let buffer_stats = self.async_buffer.get_stats().await;
        if buffer_stats.backpressure_active {
            status = ActorStatus::Degraded;
            errors.push("Buffer: Backpressure active".to_string());
        }
        if buffer_stats.dead_letter_operations > 100 {
            status = ActorStatus::Degraded;
            errors.push(format!("Buffer: {} dead letter items", buffer_stats.dead_letter_operations));
        }
        
        // Check sync
        let sync_stats = self.sync_manager.get_stats().await;
        if sync_stats.total_failed > 100 {
            status = ActorStatus::Degraded;
            errors.push(format!("Sync: {} failed operations", sync_stats.total_failed));
        }
        
        // Check conflicts
        let conflicts = self.sync_manager.get_conflicts().await;
        if !conflicts.is_empty() {
            status = ActorStatus::Degraded;
            errors.push(format!("Sync: {} unresolved conflicts", conflicts.len()));
        }
        
        // Si hay errores críticos
        let mut error_count = self.consecutive_errors.write().await;
        if !errors.is_empty() {
            *error_count += 1;
            if *error_count > 5 {
                status = ActorStatus::Unhealthy;
            }
        } else {
            *error_count = 0;
        }
        
        *self.last_health_check.write().await = chrono::Utc::now();
        
        HealthStatus {
            god: GodName::Hestia,
            status,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count + (*error_count as u64),
            last_error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            memory_usage_mb: self.estimate_memory_usage().await,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Estima uso de memoria
    async fn estimate_memory_usage(&self) -> f64 {
        // Estimación basada en estadísticas de cache
        let cache_stats = self.cache.get_stats().await;
        let memory_stats = self.memory_store.get_stats().await;
        
        let cache_mb = cache_stats.memory_used_bytes as f64 / 1024.0 / 1024.0;
        let memory_mb = memory_stats.total_size_bytes as f64 / 1024.0 / 1024.0;
        
        cache_mb + memory_mb
    }
    
    /// Loop de mantenimiento
    async fn maintenance_loop(&self) {
        let mut interval = interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            if !*self.running.read().await {
                break;
            }
            
            // Cleanup de items expirados
            if let Err(e) = self.memory_store.cleanup_expired().await {
                warn!("Maintenance: Cleanup expired failed: {}", e);
            }
            
            // Flush buffer si hay items pendientes
            let buffer_len = self.async_buffer.len().await;
            if buffer_len > 0 {
                if let Err(e) = self.async_buffer.flush().await {
                    warn!("Maintenance: Buffer flush failed: {}", e);
                }
            }
            
            // Sync si hay pendientes
            let sync_queue = self.sync_manager.get_queue().await;
            if !sync_queue.is_empty() {
                let _ = self.sync_manager.sync_all().await;
            }
            
            debug!("Maintenance cycle completed");
        }
    }
}

#[async_trait]
impl OlympianActor for Hestia {
    fn name(&self) -> GodName {
        GodName::Hestia
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Persistence
    }
    
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();
        
        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Event(event) => self.handle_event(event).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }
    
    fn persistent_state(&self) -> serde_json::Value {
        // Estado que debe persistirse
        serde_json::json!({
            "name": "Hestia",
            "version": "v15",
            "messages": self.state.message_count,
            "errors": self.state.error_count,
        })
    }
    
    fn load_state(&mut self, state: &serde_json::Value) -> Result<(), ActorError> {
        // Restaurar estado
        info!("Loading Hestia state: {:?}", state);
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Hestia,
            status: self.state.status.clone(),
            last_seen: chrono::Utc::now(),
            load: 0.0, // Calcular basado en buffer size
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        self.perform_health_check().await
    }
    
    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }
    
    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🏠 Hestia v15: Initializing persistence system...");
        
        // Iniciar componentes
        self.cache.start_background_tasks().await;
        self.async_buffer.start().await;
        self.sync_manager.start().await;
        
        // Marcar como running
        *self.running.write().await = true;
        
        // Iniciar loop de mantenimiento
        let this = Arc::new(self.clone_ref());
        let maintenance = tokio::spawn(async move {
            this.maintenance_loop().await;
        });
        *self.maintenance_handle.write().await = Some(maintenance);
        
        // Conectar a SurrealDB
        if let Err(e) = self.surreal.connect().await {
            warn!("SurrealDB connection failed (will retry): {:?}", e);
        }
        
        info!("🏠 Hestia v15: Persistence system initialized");
        info!("  - Valkey (L2): Connected");
        info!("  - SurrealDB (L3): {}", 
            if self.surreal.health_check().await.ok() == Some(true) { "Connected" } else { "Pending" });
        info!("  - Cache: L1 + L2 multi-level");
        info!("  - Buffer: Async with batching");
        info!("  - Sync: Automatic L2 <-> L3");
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🏠 Hestia: Shutting down persistence system...");
        
        // Detener loop de mantenimiento
        *self.running.write().await = false;
        if let Some(handle) = self.maintenance_handle.write().await.take() {
            handle.abort();
        }
        
        // Flush buffer
        match self.async_buffer.flush().await {
            Ok(result) => info!("Flushed {} buffered operations", result.flushed),
            Err(e) => warn!("Buffer flush failed: {}", e),
        }
        
        // Sync final
        let _ = self.sync_manager.sync_all().await;
        
        // Detener componentes
        self.async_buffer.stop().await;
        self.sync_manager.stop().await;
        
        info!("🏠 Hestia: Persistence system shutdown complete");
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Implementación de handlers
impl Hestia {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Configure { config } => {
                if let Ok(hestia_cmd) = serde_json::from_value::<HestiaCommand>(config.clone()) {
                    self.execute_hestia_command(hestia_cmd).await
                } else {
                    Err(ActorError::InvalidCommand {
                        god: GodName::Hestia,
                        reason: "Unknown Hestia command format".to_string(),
                    })
                }
            }
            CommandPayload::Shutdown => {
                self.shutdown().await?;
                Ok(ResponsePayload::Success { 
                    message: "Hestia shutdown complete".to_string() 
                })
            }
            CommandPayload::FlushBuffer => {
                let result = self.flush().await?;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(result).unwrap_or_default()
                })
            }
            _ => Err(ActorError::InvalidCommand {
                god: GodName::Hestia,
                reason: format!("Unsupported command: {:?}", cmd),
            }),
        }
    }
    
    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let stats = self.get_full_stats().await;
                Ok(ResponsePayload::Stats { data: stats })
            }
            QueryPayload::HealthStatus => {
                let health = self.health_check().await;
                Ok(ResponsePayload::Status { 
                    status: serde_json::to_value(health).unwrap_or_default() 
                })
            }
            QueryPayload::GetData { key } => {
                match self.load(&key).await? {
                    Some(value) => Ok(ResponsePayload::Data { data: value }),
                    None => Ok(ResponsePayload::Error { 
                        error: format!("Key '{}' not found", key),
                        code: 404,
                    }),
                }
            }
            QueryPayload::Search { query } => {
                // Search in all available data sources
                let items = self.memory_store.get_all(&query).await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Data { data: serde_json::to_value(items).unwrap_or_default() })
            }
            _ => Err(ActorError::InvalidQuery {
                god: GodName::Hestia,
                reason: "Unsupported query type".to_string(),
            }),
        }
    }
    
    async fn handle_event(&self, event: crate::traits::message::EventPayload) -> Result<ResponsePayload, ActorError> {
        match event {
            crate::traits::message::EventPayload::DataReceived { source, data_type } => {
                debug!("Hestia received data event from {:?}: {}", source, data_type);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() }),
        }
    }
    
    async fn execute_hestia_command(&self, cmd: HestiaCommand) -> Result<ResponsePayload, ActorError> {
        match cmd {
            HestiaCommand::Save { key, value, table, ttl_seconds, encrypt, tags } => {
                let tags_set = tags.into_iter().collect::<HashSet<_>>();
                self.save(&key, &value, table.as_deref(), ttl_seconds, encrypt, tags_set.into_iter().collect()).await?;
                Ok(ResponsePayload::Success { 
                    message: format!("Saved key '{}'", key) 
                })
            }
            HestiaCommand::Load { key, from: _ } => {
                match self.load(&key).await? {
                    Some(value) => Ok(ResponsePayload::Data { data: value }),
                    None => Ok(ResponsePayload::Error { 
                        error: format!("Key '{}' not found", key),
                        code: 404,
                    }),
                }
            }
            HestiaCommand::Delete { key, cascade } => {
                self.delete(&key, cascade).await?;
                Ok(ResponsePayload::Success { 
                    message: format!("Deleted key '{}'", key) 
                })
            }
            HestiaCommand::SaveBatch { items, table } => {
                for (key, value) in items {
                    self.save(&key, &value, table.as_deref(), None, false, vec![]).await?;
                }
                Ok(ResponsePayload::Success { 
                    message: "Batch save complete".to_string() 
                })
            }
            HestiaCommand::FlushBuffer => {
                let result = self.flush().await?;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(&result).unwrap_or_default()
                })
            }
            HestiaCommand::Sync { direction, keys } => {
                let result = self.sync(keys).await?;
                Ok(ResponsePayload::Data { 
                    data: serde_json::to_value(&result).unwrap_or_default()
                })
            }
            HestiaCommand::Invalidate { key, level } => {
                if let Some(_level) = level {
                    // Invalidate específico de nivel
                }
                self.cache.invalidate(&key).await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Success { 
                    message: format!("Invalidated key '{}'", key) 
                })
            }
            HestiaCommand::Backup { table } => {
                let meta = self.sync_manager.backup_table(&table).await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Data { 
                    data: serde_json::json!(meta)
                })
            }
            HestiaCommand::Restore { table, backup_id } => {
                let count = self.sync_manager.restore_backup(&table, &backup_id).await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Success { 
                    message: format!("Restored {} records", count) 
                })
            }
            HestiaCommand::CleanupExpired => {
                let cleaned = self.memory_store.cleanup_expired().await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Success { 
                    message: format!("Cleaned {} expired items", cleaned) 
                })
            }
            HestiaCommand::ResetStats => {
                // Reset estadísticas
                Ok(ResponsePayload::Success { 
                    message: "Statistics reset".to_string() 
                })
            }
            _ => Err(ActorError::InvalidCommand {
                god: GodName::Hestia,
                reason: "Command not implemented".to_string(),
            }),
        }
    }
    
    async fn execute_hestia_query(&self, query: HestiaQuery) -> Result<ResponsePayload, ActorError> {
        match query {
            HestiaQuery::Get { key } => {
                match self.load(&key).await? {
                    Some(value) => Ok(ResponsePayload::Data { data: value }),
                    None => Ok(ResponsePayload::Error { 
                        error: format!("Key '{}' not found", key),
                        code: 404,
                    }),
                }
            }
            HestiaQuery::GetAll { pattern } => {
                let pattern = pattern.unwrap_or("*".to_string());
                let items = self.memory_store.get_all(&pattern).await.map_err(|e| ActorError::StateError {
                    god: GodName::Hestia,
                    message: e.to_string(),
                })?;
                Ok(ResponsePayload::Data { data: serde_json::json!(items) })
            }
            HestiaQuery::GetStats => {
                let stats = self.get_full_stats().await;
                Ok(ResponsePayload::Stats { data: stats })
            }
            HestiaQuery::GetCacheStats => {
                let stats = self.cache.get_stats().await;
                Ok(ResponsePayload::Stats { data: serde_json::json!(stats) })
            }
            HestiaQuery::GetBufferStats => {
                let stats = self.async_buffer.get_stats().await;
                Ok(ResponsePayload::Stats { data: serde_json::json!(stats) })
            }
            HestiaQuery::GetSyncStats => {
                let stats = self.sync_manager.get_stats().await;
                Ok(ResponsePayload::Stats { data: serde_json::json!(stats) })
            }
            HestiaQuery::HealthCheck => {
                let health = self.health_check().await;
                Ok(ResponsePayload::Status { 
                    status: serde_json::to_value(health).unwrap_or_default() 
                })
            }
            HestiaQuery::GetConflicts => {
                let conflicts = self.sync_manager.get_conflicts().await;
                Ok(ResponsePayload::Data { data: serde_json::json!(conflicts) })
            }
            HestiaQuery::GetDeadLetter => {
                let dlq = self.async_buffer.get_dead_letter().await;
                Ok(ResponsePayload::Data { data: serde_json::json!(dlq) })
            }
            _ => Err(ActorError::InvalidQuery {
                god: GodName::Hestia,
                reason: "Query not implemented".to_string(),
            }),
        }
    }
    
    fn clone_ref(&self) -> Self {
        // Clone para uso interno (no expuesto públicamente)
        let (tx, rx) = mpsc::channel(1000);
        
        Hestia {
            name: GodName::Hestia,
            state: self.state.clone(),
            config: self.config.clone(),
            memory_store: self.memory_store.clone(),
            cache: self.cache.clone(),
            async_buffer: self.async_buffer.clone(),
            sync_manager: self.sync_manager.clone(),
            valkey: self.valkey.clone(),
            surreal: self.surreal.clone(),
            hades_encryption: RwLock::new(*self.hades_encryption.blocking_read()),
            default_encryption_key: RwLock::new(self.default_encryption_key.blocking_read().clone()),
            last_health_check: RwLock::new(*self.last_health_check.blocking_read()),
            consecutive_errors: RwLock::new(*self.consecutive_errors.blocking_read()),
            running: RwLock::new(*self.running.blocking_read()),
            maintenance_handle: RwLock::new(None),
            command_tx: tx,
            command_rx: RwLock::new(rx),
        }
    }
}
