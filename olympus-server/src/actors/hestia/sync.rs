// src/actors/hestia/sync.rs
// OLYMPUS v15 - Hestia Sync Manager
// Sincronización bidireccional entre Valkey (L2) y SurrealDB (L3)
// Con manejo de conflictos, operaciones atómicas y backup/restore

use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, error, instrument};

use crate::infrastructure::{ValkeyStore, SurrealStore};
use crate::errors::PersistenceError;

/// Dirección de sincronización
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SyncDirection {
    L2ToL3,  // Valkey -> SurrealDB
    L3ToL2,  // SurrealDB -> Valkey
    Bidirectional,
}

/// Estado de un registro de sincronización
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Failed,
    Conflict,
    Resolved,
}

/// Registro de sincronización
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub id: String,
    pub key: String,
    pub table: String,
    pub value: serde_json::Value,
    pub l2_version: u64,
    pub l3_version: u64,
    pub l2_timestamp: chrono::DateTime<chrono::Utc>,
    pub l3_timestamp: chrono::DateTime<chrono::Utc>,
    pub status: SyncStatus,
    pub conflict_resolution: Option<ConflictResolution>,
    pub retry_count: u32,
    pub last_sync_attempt: Option<chrono::DateTime<chrono::Utc>>,
    pub checksum: String,
}

impl SyncRecord {
    pub fn new(key: String, table: String, value: serde_json::Value) -> Self {
        let checksum = Self::calculate_checksum(&value);
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            table,
            value,
            l2_version: 1,
            l3_version: 0,
            l2_timestamp: chrono::Utc::now(),
            l3_timestamp: chrono::DateTime::UNIX_EPOCH,
            status: SyncStatus::Pending,
            conflict_resolution: None,
            retry_count: 0,
            last_sync_attempt: None,
            checksum,
        }
    }
    
    fn calculate_checksum(value: &serde_json::Value) -> String {
        use blake3::hash;
        let json = serde_json::to_string(value).unwrap_or_default();
        hash(json.as_bytes()).to_hex().to_string()
    }
    
    pub fn update_l2(&mut self, value: serde_json::Value) {
        self.value = value;
        self.l2_version += 1;
        self.l2_timestamp = chrono::Utc::now();
        self.checksum = Self::calculate_checksum(&self.value);
        self.status = SyncStatus::Pending;
        self.l3_version = 0; // Reset L3 version
    }
    
    pub fn mark_synced(&mut self) {
        self.l3_version = self.l2_version;
        self.l3_timestamp = chrono::Utc::now();
        self.status = SyncStatus::Synced;
        self.retry_count = 0;
        self.last_sync_attempt = Some(chrono::Utc::now());
    }
    
    pub fn mark_conflict(&mut self, resolution: ConflictResolution) {
        self.status = SyncStatus::Conflict;
        self.conflict_resolution = Some(resolution);
    }
    
    pub fn is_in_conflict(&self) -> bool {
        self.status == SyncStatus::Conflict
    }
    
    pub fn needs_sync(&self) -> bool {
        self.l2_version > self.l3_version || self.status == SyncStatus::Pending
    }
}

/// Resolución de conflictos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    L2Wins,           // Valkey gana
    L3Wins,           // SurrealDB gana
    LastWriteWins,    // Última escritura gana
    Manual,           // Requiere intervención manual
    Merge { strategy: MergeStrategy }, // Merge automático
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    JsonMerge,        // Merge de JSON profundo
    ArrayConcat,      // Concatenar arrays
    NumericSum,       // Sumar valores numéricos
    KeepBoth,         // Mantener ambos como array
}

/// Configuración del sync manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub sync_interval_secs: u64,
    pub max_concurrent_syncs: usize,
    pub batch_size: usize,
    pub conflict_resolution: ConflictResolution,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub enable_checksums: bool,
    pub enable_compression: bool,
    pub sync_direction: SyncDirection,
    pub tables_to_sync: HashSet<String>, // Empty = all tables
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_secs: 30,
            max_concurrent_syncs: 10,
            batch_size: 100,
            conflict_resolution: ConflictResolution::LastWriteWins,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            enable_checksums: true,
            enable_compression: false,
            sync_direction: SyncDirection::Bidirectional,
            tables_to_sync: HashSet::new(),
        }
    }
}

/// Estadísticas de sincronización
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStats {
    pub total_synced: u64,
    pub total_failed: u64,
    pub total_conflicts: u64,
    pub pending_count: usize,
    pub syncing_count: usize,
    pub last_sync_duration_ms: u64,
    pub average_sync_duration_ms: f64,
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub table: String,
    pub record_count: u64,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Sync Manager - Corazón de la sincronización Valkey <-> SurrealDB
#[derive(Debug)]
pub struct SyncManager {
    valkey: Arc<ValkeyStore>,
    surreal: Arc<SurrealStore>,
    
    // Configuración
    config: RwLock<SyncConfig>,
    
    // Estado de sync
    sync_queue: RwLock<VecDeque<SyncRecord>>,
    active_syncs: RwLock<HashMap<String, SyncRecord>>,
    conflict_queue: RwLock<VecDeque<SyncRecord>>,
    
    // Estadísticas
    stats: RwLock<SyncStats>,
    latency_history: RwLock<VecDeque<u64>>,
    
    // Control
    running: RwLock<bool>,
    sync_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    
    // Canales
    sync_tx: mpsc::Sender<SyncRecord>,
    sync_rx: RwLock<mpsc::Receiver<SyncRecord>>,
    
    // Prefijos de claves
    sync_queue_key: String,
    conflict_key: String,
}

impl SyncManager {
    pub fn new(valkey: Arc<ValkeyStore>, surreal: Arc<SurrealStore>) -> Self {
        Self::with_config(valkey, surreal, SyncConfig::default())
    }
    
    pub fn with_config(
        valkey: Arc<ValkeyStore>,
        surreal: Arc<SurrealStore>,
        config: SyncConfig,
    ) -> Self {
        let (sync_tx, sync_rx) = mpsc::channel(1000);
        
        Self {
            valkey,
            surreal,
            config: RwLock::new(config),
            sync_queue: RwLock::new(VecDeque::new()),
            active_syncs: RwLock::new(HashMap::new()),
            conflict_queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(SyncStats::default()),
            latency_history: RwLock::new(VecDeque::with_capacity(100)),
            running: RwLock::new(false),
            sync_handle: RwLock::new(None),
            sync_tx,
            sync_rx: RwLock::new(sync_rx),
            sync_queue_key: "olympus:hestia:sync:queue".to_string(),
            conflict_key: "olympus:hestia:sync:conflicts".to_string(),
        }
    }
    
    /// Inicia el sync manager
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);
        
        let config = self.config.read().await.clone();
        
        // Spawn sync worker
        let this = Arc::new(self.clone_ref());
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.sync_interval_secs));
            
            loop {
                interval.tick().await;
                
                let is_running = *this.running.read().await;
                if !is_running {
                    break;
                }
                
                // Procesar cola de sync
                if let Err(e) = this.process_sync_queue().await {
                    error!("Sync queue processing error: {}", e);
                }
            }
        });
        
        *self.sync_handle.write().await = Some(handle);
        
        info!("SyncManager started (interval: {}s)", config.sync_interval_secs);
    }
    
    /// Detiene el sync manager
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);
        
        if let Some(handle) = self.sync_handle.write().await.take() {
            handle.abort();
        }
        
        info!("SyncManager stopped");
    }
    
    /// Sincroniza una clave individual de Valkey a SurrealDB
    pub async fn sync_to_l3(&self, key: &str, value: &serde_json::Value) -> Result<(), PersistenceError> {
        // Crear o actualizar registro
        let table = self.infer_table(key);
        let mut record = SyncRecord::new(key.to_string(), table, value.clone());
        record.status = SyncStatus::Pending;
        
        // Agregar a cola
        self.queue_sync(record).await?;
        
        Ok(())
    }
    
    /// Recupera una clave de L3 (SurrealDB) a L2 (Valkey)
    pub async fn fetch_from_l3(&self, key: &str) -> Result<Option<serde_json::Value>, PersistenceError> {
        let table = self.infer_table(key);
        
        // Consultar SurrealDB
        let result: Result<Option<serde_json::Value>, _> = self.surreal.select(&table, key).await;
        
        match result {
            Ok(Some(value)) => {
                debug!("Fetched key '{}' from L3", key);
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Failed to fetch key '{}' from L3: {}", key, e);
                Err(PersistenceError::SurrealError(e.to_string()))
            }
        }
    }
    
    /// Sincroniza todas las claves pendientes
    pub async fn sync_all(&self) -> SyncResult {
        let start = Instant::now();
        let mut synced = 0u64;
        let mut failed = 0u64;
        let mut conflicts = 0u64;
        
        // Obtener pendientes
        let pending: Vec<SyncRecord> = {
            let queue = self.sync_queue.read().await;
            queue.iter().filter(|r| r.needs_sync()).cloned().collect()
        };
        
        // Procesar en batches
        let config = self.config.read().await.clone();
        for chunk in pending.chunks(config.batch_size) {
            let batch: Vec<SyncRecord> = chunk.to_vec();
            
            for record in batch {
                match self.sync_single_record(record.clone()).await {
                    Ok(_) => {
                        synced += 1;
                    }
                    Err(e) => {
                        if record.is_in_conflict() {
                            conflicts += 1;
                        } else {
                            failed += 1;
                        }
                        warn!("Sync failed for '{}': {}", record.key, e);
                    }
                }
            }
        }
        
        let duration = start.elapsed().as_millis() as u64;
        
        // Actualizar estadísticas
        {
            let mut stats = self.stats.write().await;
            stats.total_synced += synced;
            stats.total_failed += failed;
            stats.total_conflicts += conflicts;
            stats.last_sync_duration_ms = duration;
            stats.last_sync_time = Some(chrono::Utc::now());
            
            let mut history = self.latency_history.write().await;
            history.push_back(duration);
            if history.len() > 100 {
                history.pop_front();
            }
            
            if !history.is_empty() {
                let avg = history.iter().sum::<u64>() as f64 / history.len() as f64;
                stats.average_sync_duration_ms = avg;
            }
        }
        
        info!("Sync completed: {} synced, {} failed, {} conflicts in {}ms", 
            synced, failed, conflicts, duration);
        
        SyncResult {
            synced: synced as usize,
            failed: failed as usize,
            conflicts: conflicts as usize,
            duration_ms: duration,
        }
    }
    
    /// Resuelve un conflicto manualmente
    pub async fn resolve_conflict(
        &self, 
        record_id: &str, 
        resolution: ConflictResolution,
        new_value: Option<serde_json::Value>,
    ) -> Result<(), PersistenceError> {
        let mut conflict_queue = self.conflict_queue.write().await;
        
        if let Some(pos) = conflict_queue.iter().position(|r| r.id == record_id) {
            let mut record = conflict_queue.remove(pos).unwrap();
            
            record.conflict_resolution = Some(resolution.clone());
            record.status = SyncStatus::Resolved;
            
            // Aplicar resolución
            match resolution {
                ConflictResolution::L2Wins => {
                    // Reintentar sync de L2
                    self.queue_sync(record).await?;
                }
                ConflictResolution::L3Wins => {
                    // Actualizar L2 con valor de L3
                    if let Ok(Some(l3_value)) = self.fetch_from_l3(&record.key).await {
                        record.value = l3_value;
                        record.mark_synced();
                    }
                }
                ConflictResolution::Manual => {
                    if let Some(value) = new_value {
                        record.value = value;
                        record.l2_version += 1;
                        record.status = SyncStatus::Pending;
                        self.queue_sync(record).await?;
                    }
                }
                _ => {}
            }
            
            info!("Conflict resolved for record {}", record_id);
        }
        
        Ok(())
    }
    
    /// Crea un backup de una tabla
    pub async fn backup_table(&self, table: &str) -> Result<BackupMetadata, PersistenceError> {
        let start = Instant::now();
        
        // Obtener todos los registros de la tabla
        let query = format!("SELECT * FROM {}", table);
        let records: Vec<serde_json::Value> = self.surreal.query(&query).await
            .map_err(|e| PersistenceError::SurrealError(e.to_string()))?;
        
        // Calcular checksum y metadata
        let count = records.len() as u64;
        let data = serde_json::to_string(&records)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        let size_bytes = data.len() as u64;
        let checksum = blake3::hash(data.as_bytes()).to_hex().to_string();
        
        let metadata = BackupMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            table: table.to_string(),
            record_count: count,
            size_bytes,
            checksum: checksum.clone(),
        };
        
        // Guardar en Valkey
        let json = serde_json::to_string(&records)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        let backup_key = format!("olympus:hestia:backup:{}:{}", table, metadata.id);
        self.valkey.set(&backup_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Guardar metadata
        let meta_key = format!("olympus:hestia:backup:{}:{}:meta", table, metadata.id);
        let meta_json = serde_json::to_string(&metadata)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        self.valkey.set(&meta_key, &meta_json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        let duration = start.elapsed().as_millis();
        info!("Backup created for table {}: {} records in {}ms", table, count, duration);
        
        Ok(metadata)
    }
    
    /// Restaura un backup
    pub async fn restore_backup(
        &self, 
        table: &str, 
        backup_id: &str
    ) -> Result<u64, PersistenceError> {
        let start = Instant::now();
        
        // Leer backup
        let backup_key = format!("olympus:hestia:backup:{}:{}", table, backup_id);
        let json = self.valkey.get(&backup_key).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?
            .ok_or_else(|| PersistenceError::KeyNotFound(backup_key.clone()))?;
        
        let records: Vec<serde_json::Value> = serde_json::from_str(&json)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?;
        
        // Restaurar a SurrealDB
        let mut restored = 0u64;
        for record in records {
            match self.surreal.create(table, &record).await {
                Ok(_) => restored += 1,
                Err(e) => {
                    warn!("Failed to restore record: {}", e);
                }
            }
        }
        
        let duration = start.elapsed().as_millis();
        info!("Backup restored for table {}: {} records in {}ms", 
            table, restored, duration);
        
        Ok(restored)
    }
    
    /// Lista backups disponibles
    pub async fn list_backups(&self, _table: &str) -> Vec<BackupMetadata> {
        // En una implementación real, escanearías las claves
        // Por ahora retornamos vacío
        Vec::new()
    }
    
    /// Obtiene estadísticas
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }
    
    /// Obtiene conflictos pendientes
    pub async fn get_conflicts(&self) -> Vec<SyncRecord> {
        self.conflict_queue.read().await.iter().cloned().collect()
    }
    
    /// Obtiene registros en cola de sync
    pub async fn get_queue(&self) -> Vec<SyncRecord> {
        self.sync_queue.read().await.iter().cloned().collect()
    }
    
    // Métodos privados
    
    fn clone_ref(&self) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        SyncManager {
            valkey: self.valkey.clone(),
            surreal: self.surreal.clone(),
            config: RwLock::new(self.config.try_read().unwrap().clone()),
            sync_queue: RwLock::new(VecDeque::new()),
            active_syncs: RwLock::new(HashMap::new()),
            conflict_queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(self.stats.try_read().unwrap().clone()),
            latency_history: RwLock::new(VecDeque::with_capacity(100)),
            running: RwLock::new(*self.running.try_read().unwrap()),
            sync_handle: RwLock::new(None),
            sync_tx: tx,
            sync_rx: RwLock::new(rx),
            sync_queue_key: self.sync_queue_key.clone(),
            conflict_key: self.conflict_key.clone(),
        }
    }
    
    async fn queue_sync(&self, record: SyncRecord) -> Result<(), PersistenceError> {
        // Guardar en Valkey para durabilidad
        let json = serde_json::to_string(&record)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        self.valkey.lpush(&self.sync_queue_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Agregar a cola en memoria
        let mut queue = self.sync_queue.write().await;
        queue.push_back(record);
        
        Ok(())
    }
    
    async fn process_sync_queue(&self) -> Result<(), PersistenceError> {
        let config = self.config.read().await.clone();
        
        // Recoger pendientes
        let to_sync: Vec<SyncRecord> = {
            let queue = self.sync_queue.read().await;
            queue.iter()
                .filter(|r| r.needs_sync() && r.status != SyncStatus::Syncing)
                .take(config.batch_size)
                .cloned()
                .collect()
        };
        
        if to_sync.is_empty() {
            return Ok(());
        }
        
        // Marcar como syncing
        {
            let mut queue = self.sync_queue.write().await;
            for record in &to_sync {
                if let Some(r) = queue.iter_mut().find(|r| r.id == record.id) {
                    r.status = SyncStatus::Syncing;
                }
            }
        }
        
        // Procesar en paralelo limitado
        let mut handles = Vec::new();
        for record in to_sync {
            let this = Arc::new(self.clone_ref());
            let handle = tokio::spawn(async move {
                this.sync_single_record(record).await
            });
            handles.push(handle);
        }
        
        // Esperar resultados
        for handle in handles {
            let _ = handle.await;
        }
        
        Ok(())
    }
    
    #[instrument(skip(self, record))]
    async fn sync_single_record(&self, mut record: SyncRecord) -> Result<(), PersistenceError> {
        record.last_sync_attempt = Some(chrono::Utc::now());
        
        // Verificar si hay conflicto (L3 tiene versión diferente)
        let l3_data: Option<serde_json::Value> = self.surreal
            .select(&record.table, &record.key)
            .await
            .map_err(|e| PersistenceError::SurrealError(e.to_string()))?;
        
        if let Some(l3_value) = l3_data {
            // Verificar si hay conflicto
            let l3_checksum = SyncRecord::calculate_checksum(&l3_value);
            if l3_checksum != record.checksum {
                // Conflicto detectado
                let resolution = self.resolve_conflict_auto(&record, &l3_value).await?;
                
                if let ConflictResolution::Manual = resolution {
                    let key = record.key.clone();
                    record.mark_conflict(resolution);
                    self.move_to_conflict_queue(record).await?;
                    return Err(PersistenceError::TransactionFailed(
                        format!("Conflict detected for key '{}'", key)
                    ));
                }
            }
        }
        
        // Escribir a L3 (SurrealDB)
        let result = match self.surreal.update(&record.table, &record.key, &record.value).await {
            Ok(_) => {
                record.mark_synced();
                Ok(())
            }
            Err(e) => {
                record.retry_count += 1;
                
                if record.retry_count >= self.config.read().await.retry_attempts {
                    Err(PersistenceError::TransactionFailed(e.to_string()))
                } else {
                    // Reintentar
                    sleep(Duration::from_millis(self.config.read().await.retry_delay_ms)).await;
                    Err(PersistenceError::TransactionFailed(e.to_string()))
                }
            }
        };
        
        // Actualizar cola
        {
            let mut queue = self.sync_queue.write().await;
            if let Some(r) = queue.iter_mut().find(|r| r.id == record.id) {
                *r = record.clone();
            }
        }
        
        result
    }
    
    async fn resolve_conflict_auto(
        &self, 
        record: &SyncRecord, 
        _l3_value: &serde_json::Value,
    ) -> Result<ConflictResolution, PersistenceError> {
        let config = self.config.read().await.clone();
        
        match config.conflict_resolution {
            ConflictResolution::L2Wins => Ok(ConflictResolution::L2Wins),
            ConflictResolution::L3Wins => Ok(ConflictResolution::L3Wins),
            ConflictResolution::LastWriteWins => {
                // Comparar timestamps
                if record.l2_timestamp > record.l3_timestamp {
                    Ok(ConflictResolution::L2Wins)
                } else {
                    Ok(ConflictResolution::L3Wins)
                }
            }
            ConflictResolution::Manual => Ok(ConflictResolution::Manual),
            ConflictResolution::Merge { strategy } => {
                // Merge automático (simplificado)
                Ok(ConflictResolution::Merge { strategy })
            }
        }
    }
    
    async fn move_to_conflict_queue(&self, record: SyncRecord) -> Result<(), PersistenceError> {
        // Guardar en Valkey
        let json = serde_json::to_string(&record)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        self.valkey.lpush(&self.conflict_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Agregar a cola
        let mut conflicts = self.conflict_queue.write().await;
        conflicts.push_back(record);
        
        Ok(())
    }
    
    fn infer_table(&self, key: &str) -> String {
        // Inferir tabla del prefijo de la clave
        if key.contains(':') {
            key.split(':').next().unwrap_or("default").to_string()
        } else {
            "default".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced: usize,
    pub failed: usize,
    pub conflicts: usize,
    pub duration_ms: u64,
}
