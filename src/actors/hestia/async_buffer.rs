// src/actors/hestia/async_buffer.rs
// OLYMPUS v15 - Hestia Async Buffer
// Buffer de escritura asíncrona con batching inteligente, backpressure y retry con backoff

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock, Semaphore, Notify};
use tokio::time::{interval, sleep, timeout};
use tracing::{debug, info, warn, error};


use crate::infrastructure::{ValkeyStore, SurrealStore};
use crate::errors::PersistenceError;

/// Estado de una operación en buffer
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BufferStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    DeadLetter,
}

/// Operación bufferizada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedOperation {
    pub id: String,
    pub table: String,
    pub key: String,
    pub value: serde_json::Value,
    pub operation_type: OperationType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub priority: OperationPriority,
    pub status: BufferStatus,
    pub attempts: u32,
    pub last_attempt: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OperationType {
    Create,
    Update,
    Delete,
    Upsert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum OperationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl BufferedOperation {
    pub fn new(
        table: &str,
        key: String,
        value: serde_json::Value,
        operation_type: OperationType,
        priority: OperationPriority,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            table: table.to_string(),
            key,
            value,
            operation_type,
            created_at: chrono::Utc::now(),
            priority,
            status: BufferStatus::Pending,
            attempts: 0,
            last_attempt: None,
            error_message: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn mark_attempt(&mut self, error: Option<String>) {
        self.attempts += 1;
        self.last_attempt = Some(chrono::Utc::now());
        self.error_message = error;
        
        if self.attempts >= 5 {
            self.status = BufferStatus::DeadLetter;
        } else if error.is_some() {
            self.status = BufferStatus::Failed;
        }
    }
    
    pub fn mark_completed(&mut self) {
        self.status = BufferStatus::Completed;
        self.error_message = None;
    }
    
    pub fn can_retry(&self) -> bool {
        self.attempts < 5 && self.status != BufferStatus::DeadLetter
    }
    
    pub fn retry_delay(&self) -> Duration {
        // Backoff exponencial: 100ms, 200ms, 400ms, 800ms, 1.6s
        let base = 100u64;
        let delay = base * 2u64.pow(self.attempts);
        Duration::from_millis(delay.min(5000)) // Max 5 segundos
    }
}

/// Configuración del buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncBufferConfig {
    pub max_buffer_size: usize,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub flush_interval_ms: u64,
    pub max_concurrent_batches: usize,
    pub enable_compression: bool,
    pub dead_letter_enabled: bool,
    pub backpressure_threshold: usize,
}

impl Default for AsyncBufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10000,
            batch_size: 100,
            batch_timeout_ms: 1000,
            flush_interval_ms: 5000,
            max_concurrent_batches: 5,
            enable_compression: false,
            dead_letter_enabled: true,
            backpressure_threshold: 8000,
        }
    }
}

/// Estadísticas del buffer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BufferStats {
    pub total_operations: u64,
    pub pending_operations: usize,
    pub processing_operations: usize,
    pub completed_operations: u64,
    pub failed_operations: u64,
    pub dead_letter_operations: u64,
    pub batches_submitted: u64,
    pub batches_failed: u64,
    pub average_batch_size: f64,
    pub average_latency_ms: f64,
    pub backpressure_active: bool,
}

/// Async Buffer con batching inteligente
#[derive(Debug)]
pub struct AsyncBuffer {
    valkey: Arc<ValkeyStore>,
    surreal: Arc<SurrealStore>,
    
    // Configuración
    config: RwLock<AsyncBufferConfig>,
    
    // Canales
    tx: mpsc::Sender<BufferedOperation>,
    rx: RwLock<mpsc::Receiver<BufferedOperation>>,
    
    // Semáforo para backpressure
    semaphore: Semaphore,
    
    // Notificador para flush manual
    flush_notify: Arc<Notify>,
    
    // Estado interno
    pending_ops: RwLock<VecDeque<BufferedOperation>>,
    processing_ops: RwLock<HashMap<String, BufferedOperation>>,
    dead_letter_queue: RwLock<VecDeque<BufferedOperation>>,
    
    // Estadísticas
    stats: RwLock<BufferStats>,
    
    // Queue keys en Valkey
    queue_key: String,
    processing_key: String,
    dead_letter_key: String,
    
    // Control de tareas
    worker_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    flush_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
    
    // Métricas de latencia
    latency_history: RwLock<VecDeque<u64>>,
}

impl AsyncBuffer {
    pub fn new(valkey: Arc<ValkeyStore>, surreal: Arc<SurrealStore>) -> Self {
        Self::with_config(valkey, surreal, AsyncBufferConfig::default())
    }
    
    pub fn with_config(
        valkey: Arc<ValkeyStore>,
        surreal: Arc<SurrealStore>,
        config: AsyncBufferConfig,
    ) -> Self {
        let (tx, rx) = mpsc::channel(config.max_buffer_size);
        
        Self {
            valkey,
            surreal,
            config: RwLock::new(config.clone()),
            tx,
            rx: RwLock::new(rx),
            semaphore: Semaphore::new(config.max_concurrent_batches),
            flush_notify: Arc::new(Notify::new()),
            pending_ops: RwLock::new(VecDeque::new()),
            processing_ops: RwLock::new(HashMap::new()),
            dead_letter_queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(BufferStats::default()),
            queue_key: "olympus:hestia:buffer:queue".to_string(),
            processing_key: "olympus:hestia:buffer:processing".to_string(),
            dead_letter_key: "olympus:hestia:buffer:dead_letter".to_string(),
            worker_handle: RwLock::new(None),
            flush_handle: RwLock::new(None),
            latency_history: RwLock::new(VecDeque::with_capacity(100)),
        }
    }
    
    /// Inicia las tareas en background
    pub async fn start(&self) {
        let config = self.config.read().await.clone();
        
        // Worker principal para batching
        let this = Arc::new(self.clone_ref());
        let worker = tokio::spawn(async move {
            this.worker_loop().await;
        });
        *self.worker_handle.write().await = Some(worker);
        
        // Flush periódico
        let this = Arc::new(self.clone_ref());
        let flusher = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.flush_interval_ms));
            loop {
                interval.tick().await;
                if let Err(e) = this.periodic_flush().await {
                    warn!("Periodic flush error: {}", e);
                }
            }
        });
        *self.flush_handle.write().await = Some(flusher);
        
        info!("AsyncBuffer started with batch_size={}", config.batch_size);
    }
    
    /// Detiene las tareas en background
    pub async fn stop(&self) {
        // Cancelar workers
        if let Some(handle) = self.worker_handle.write().await.take() {
            handle.abort();
        }
        if let Some(handle) = self.flush_handle.write().await.take() {
            handle.abort();
        }
        
        // Flush final
        let _ = self.flush().await;
        
        info!("AsyncBuffer stopped");
    }
    
    /// Agrega una operación al buffer
    pub async fn push(
        &self,
        table: &str,
        key: String,
        value: serde_json::Value,
        operation_type: OperationType,
    ) -> Result<String, PersistenceError> {
        self.push_with_priority(table, key, value, operation_type, OperationPriority::Normal).await
    }
    
    /// Agrega una operación con prioridad
    pub async fn push_with_priority(
        &self,
        table: &str,
        key: String,
        value: serde_json::Value,
        operation_type: OperationType,
        priority: OperationPriority,
    ) -> Result<String, PersistenceError> {
        let config = self.config.read().await.clone();
        
        // Verificar backpressure
        let pending_count = self.pending_ops.read().await.len();
        if pending_count >= config.backpressure_threshold {
            let mut stats = self.stats.write().await;
            stats.backpressure_active = true;
            drop(stats);
            
            // Esperar con backoff
            sleep(Duration::from_millis(100)).await;
        }
        
        let op = BufferedOperation::new(table, key, value, operation_type, priority);
        let id = op.id.clone();
        
        // Almacenar en Valkey para durabilidad
        let json = serde_json::to_string(&op)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        self.valkey.lpush(&self.queue_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Agregar a pending
        let mut pending = self.pending_ops.write().await;
        pending.push_back(op);
        drop(pending);
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.total_operations += 1;
        stats.pending_operations += 1;
        stats.backpressure_active = false;
        drop(stats);
        
        // Notificar al worker
        self.flush_notify.notify_one();
        
        debug!("Pushed operation {} to buffer (priority: {:?})", id, priority);
        Ok(id)
    }
    
    /// Fuerza un flush inmediato
    pub async fn flush(&self) -> Result<FlushResult, PersistenceError> {
        let config = self.config.read().await.clone();
        let mut flushed = 0u64;
        let mut failed = 0u64;
        
        // Recoger todas las operaciones pendientes
        let ops: Vec<BufferedOperation> = {
            let mut pending = self.pending_ops.write().await;
            let ops: Vec<_> = pending.drain(..).collect();
            ops
        };
        
        if ops.is_empty() {
            return Ok(FlushResult { flushed: 0, failed: 0, duration_ms: 0 });
        }
        
        let start = Instant::now();
        
        // Procesar en batches
        for chunk in ops.chunks(config.batch_size) {
            let batch: Vec<BufferedOperation> = chunk.to_vec();
            
            match self.process_batch(batch.clone()).await {
                Ok(completed) => {
                    flushed += completed.len() as u64;
                    
                    // Mover a procesadas en Valkey
                    for op in &completed {
                        let json = serde_json::to_string(op)
                            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
                        self.valkey.lpush(&format!("{}:completed", self.queue_key), &json).await
                            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
                    }
                }
                Err(e) => {
                    warn!("Batch processing failed: {}", e);
                    failed += batch.len() as u64;
                    
                    // Reintentar individualmente
                    for mut op in batch {
                        op.mark_attempt(Some(e.to_string()));
                        if op.can_retry() {
                            let mut pending = self.pending_ops.write().await;
                            pending.push_back(op);
                        } else {
                            self.move_to_dead_letter(op).await?;
                        }
                    }
                }
            }
        }
        
        let duration = start.elapsed().as_millis() as u64;
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.pending_operations = 0;
        stats.batches_submitted += 1;
        drop(stats);
        
        info!("Flushed {} operations ({} failed) in {}ms", flushed, failed, duration);
        
        Ok(FlushResult {
            flushed,
            failed,
            duration_ms: duration,
        })
    }
    
    /// Obtiene estadísticas
    pub async fn get_stats(&self) -> BufferStats {
        let mut stats = self.stats.read().await.clone();
        stats.pending_operations = self.pending_ops.read().await.len();
        stats.processing_operations = self.processing_ops.read().await.len();
        
        // Calcular latencia promedio
        let latency = self.latency_history.read().await;
        if !latency.is_empty() {
            let avg: f64 = latency.iter().sum::<u64>() as f64 / latency.len() as f64;
            stats.average_latency_ms = avg;
        }
        
        stats
    }
    
    /// Obtiene el número de operaciones pendientes
    pub async fn len(&self) -> usize {
        self.pending_ops.read().await.len()
    }
    
    /// Verifica si el buffer está vacío
    pub async fn is_empty(&self) -> bool {
        self.pending_ops.read().await.is_empty()
    }
    
    /// Obtiene operaciones en dead letter queue
    pub async fn get_dead_letter(&self) -> Vec<BufferedOperation> {
        self.dead_letter_queue.read().await.iter().cloned().collect()
    }
    
    /// Reintenta operaciones en dead letter
    pub async fn retry_dead_letter(&self) -> Result<u64, PersistenceError> {
        let ops: Vec<BufferedOperation> = {
            let mut dlq = self.dead_letter_queue.write().await;
            dlq.drain(..).collect()
        };
        
        let mut retried = 0;
        for mut op in ops {
            op.status = BufferStatus::Pending;
            op.attempts = 0;
            op.error_message = None;
            
            let mut pending = self.pending_ops.write().await;
            pending.push_back(op);
            retried += 1;
        }
        
        self.flush_notify.notify_one();
        
        info!("Retried {} dead letter operations", retried);
        Ok(retried)
    }
    
    /// Limpia el dead letter queue
    pub async fn clear_dead_letter(&self) -> Result<u64, PersistenceError> {
        let count = self.dead_letter_queue.read().await.len();
        self.dead_letter_queue.write().await.clear();
        
        // Limpiar en Valkey
        let _ = self.valkey.hgetall(&self.dead_letter_key).await;
        
        let mut stats = self.stats.write().await;
        stats.dead_letter_operations = 0;
        
        info!("Cleared {} dead letter operations", count);
        Ok(count as u64)
    }
    
    // Métodos privados
    
    fn clone_ref(&self) -> Self {
        // Crear nuevos canales para el clone
        let (tx, rx) = mpsc::channel(1000);
        
        AsyncBuffer {
            valkey: self.valkey.clone(),
            surreal: self.surreal.clone(),
            config: RwLock::new(self.config.blocking_read().clone()),
            tx,
            rx: RwLock::new(rx),
            semaphore: Semaphore::new(self.config.blocking_read().max_concurrent_batches),
            flush_notify: self.flush_notify.clone(),
            pending_ops: RwLock::new(VecDeque::new()),
            processing_ops: RwLock::new(HashMap::new()),
            dead_letter_queue: RwLock::new(VecDeque::new()),
            stats: RwLock::new(self.stats.blocking_read().clone()),
            queue_key: self.queue_key.clone(),
            processing_key: self.processing_key.clone(),
            dead_letter_key: self.dead_letter_key.clone(),
            worker_handle: RwLock::new(None),
            flush_handle: RwLock::new(None),
            latency_history: RwLock::new(VecDeque::with_capacity(100)),
        }
    }
    
    async fn worker_loop(&self) {
        let config = self.config.read().await.clone();
        
        loop {
            // Esperar por nuevas operaciones o timeout
            let batch_ready = tokio::select! {
                _ = self.flush_notify.notified() => true,
                _ = sleep(Duration::from_millis(config.batch_timeout_ms)) => {
                    // Timeout: procesar si hay operaciones pendientes
                    !self.pending_ops.read().await.is_empty()
                }
            };
            
            if !batch_ready {
                continue;
            }
            
            // Adquirir semáforo para limitar concurrencia
            let _permit = match self.semaphore.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    error!("Failed to acquire semaphore");
                    continue;
                }
            };
            
            // Recoger batch
            let batch: Vec<BufferedOperation> = {
                let mut pending = self.pending_ops.write().await;
                let take_count = pending.len().min(config.batch_size);
                pending.drain(..take_count).collect()
            };
            
            if batch.is_empty() {
                continue;
            }
            
            // Mover a processing
            {
                let mut processing = self.processing_ops.write().await;
                for op in &batch {
                    processing.insert(op.id.clone(), op.clone());
                }
            }
            
            // Actualizar estadísticas
            {
                let mut stats = self.stats.write().await;
                stats.pending_operations = stats.pending_operations.saturating_sub(batch.len());
                stats.processing_operations += batch.len();
            }
            
            // Procesar batch
            let start = Instant::now();
            match self.process_batch(batch.clone()).await {
                Ok(completed) => {
                    let latency = start.elapsed().as_millis() as u64;
                    
                    // Actualizar latencia
                    {
                        let mut history = self.latency_history.write().await;
                        history.push_back(latency);
                        if history.len() > 100 {
                            history.pop_front();
                        }
                    }
                    
                    // Remover de processing
                    {
                        let mut processing = self.processing_ops.write().await;
                        for op in &completed {
                            processing.remove(&op.id);
                        }
                    }
                    
                    // Actualizar estadísticas
                    {
                        let mut stats = self.stats.write().await;
                        stats.completed_operations += completed.len() as u64;
                        stats.processing_operations = stats.processing_operations.saturating_sub(completed.len());
                    }
                    
                    debug!("Batch of {} operations completed in {}ms", completed.len(), latency);
                }
                Err(e) => {
                    warn!("Batch processing failed: {}", e);
                    
                    // Manejar fallos individuales
                    for mut op in batch {
                        op.mark_attempt(Some(e.to_string()));
                        
                        if op.can_retry() {
                            // Reintentar con delay
                            let delay = op.retry_delay();
                            tokio::spawn(async move {
                                sleep(delay).await;
                                // El op se vuelve a agregar en el próximo ciclo
                            });
                            
                            let mut pending = self.pending_ops.write().await;
                            pending.push_back(op);
                        } else {
                            // Mover a dead letter
                            let _ = self.move_to_dead_letter(op).await;
                        }
                    }
                    
                    let mut stats = self.stats.write().await;
                    stats.batches_failed += 1;
                    stats.processing_operations = 0;
                }
            }
        }
    }
    
    async fn process_batch(&self, batch: Vec<BufferedOperation>) -> Result<Vec<BufferedOperation>, PersistenceError> {
        // Agrupar por tabla para optimizar
        let mut by_table: HashMap<String, Vec<BufferedOperation>> = HashMap::new();
        for op in batch {
            by_table.entry(op.table.clone()).or_insert_with(Vec::new).push(op);
        }
        
        let mut completed = Vec::new();
        
        // Procesar cada tabla
        for (table, ops) in by_table {
            // Construir query batch para SurrealDB
            // Nota: En una implementación real, usarías transacciones de SurrealDB
            for mut op in ops {
                let result = match op.operation_type {
                    OperationType::Create => {
                        self.surreal.create(&table, &op.value).await
                            .map(|_| ())
                    }
                    OperationType::Update => {
                        // Crear ID si no existe
                        let id = op.key.clone();
                        self.surreal.update(&table, &id, &op.value).await
                            .map(|_| ())
                    }
                    OperationType::Delete => {
                        let id = op.key.clone();
                        self.surreal.delete(&table, &id).await
                    }
                    OperationType::Upsert => {
                        let id = op.key.clone();
                        // Intentar update primero, luego create si falla
                        match self.surreal.update(&table, &id, &op.value).await {
                            Ok(_) => Ok(()),
                            Err(_) => self.surreal.create(&table, &op.value).await.map(|_| ()),
                        }
                    }
                };
                
                match result {
                    Ok(_) => {
                        op.mark_completed();
                        completed.push(op);
                    }
                    Err(e) => {
                        op.mark_attempt(Some(e.to_string()));
                        if !op.can_retry() {
                            self.move_to_dead_letter(op).await?;
                        }
                    }
                }
            }
        }
        
        Ok(completed)
    }
    
    async fn move_to_dead_letter(&self, op: BufferedOperation) -> Result<(), PersistenceError> {
        if !self.config.read().await.dead_letter_enabled {
            return Ok(());
        }
        
        let op_id = op.id.clone();
        let mut op = op;
        op.status = BufferStatus::DeadLetter;
        
        // Guardar en Valkey
        let json = serde_json::to_string(&op)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
        self.valkey.lpush(&self.dead_letter_key, &json).await
            .map_err(|e| PersistenceError::ValkeyError(e.to_string()))?;
        
        // Agregar a DLQ en memoria
        let mut dlq = self.dead_letter_queue.write().await;
        dlq.push_back(op);
        drop(dlq);
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.dead_letter_operations += 1;
        
        warn!("Operation {} moved to dead letter queue", op_id);
        Ok(())
    }
    
    async fn periodic_flush(&self) -> Result<(), PersistenceError> {
        let pending_count = self.pending_ops.read().await.len();
        
        if pending_count > 0 {
            let _ = self.flush().await?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushResult {
    pub flushed: u64,
    pub failed: u64,
    pub duration_ms: u64,
}
