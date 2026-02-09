// src/actors/poseidon/flow_control.rs
// OLYMPUS v15 - Poseidon Flow Controller Real
// Control de flujo con rate limiting dinámico y backpressure

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore, SemaphorePermit, OwnedSemaphorePermit};
use tokio::time::interval;
use tracing::{debug, info};

/// Algoritmo de rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket,
    LeakyBucket,
    FixedWindow,
    SlidingWindow,
}

impl Default for RateLimitAlgorithm {
    fn default() -> Self {
        RateLimitAlgorithm::TokenBucket
    }
}

/// Configuración del Flow Controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    /// Mensajes por segundo permitidos
    pub max_messages_per_second: u64,
    /// Burst permitido (para token bucket)
    pub max_burst_size: usize,
    /// Tamaño máximo del buffer
    pub max_buffer_size: usize,
    /// Tiempo de espera máximo para adquirir permiso
    pub acquire_timeout_ms: u64,
    /// Umbral de alerta de backpressure (0.0 - 1.0)
    pub backpressure_threshold: f64,
    /// Algoritmo de rate limiting
    pub algorithm: RateLimitAlgorithm,
    /// Habilitar métricas automáticas
    pub enable_metrics: bool,
    /// Intervalo de reporte de métricas (segundos)
    pub metrics_interval_secs: u64,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            max_messages_per_second: 10_000,
            max_burst_size: 1000,
            max_buffer_size: 100_000,
            acquire_timeout_ms: 5000,
            backpressure_threshold: 0.8,
            algorithm: RateLimitAlgorithm::TokenBucket,
            enable_metrics: true,
            metrics_interval_secs: 60,
        }
    }
}

/// Métricas de flujo en tiempo real
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub messages_allowed: u64,
    pub messages_denied: u64,
    pub messages_sent: u64,
    pub messages_buffered: u64,
    pub messages_dropped: u64,
    pub current_buffer_size: usize,
    pub current_rate: f64,
    pub average_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub backpressure_active: bool,
    pub backpressure_level: f64, // 0.0 - 1.0
    pub throughput_bps: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Snapshot de métricas históricas
#[derive(Debug, Clone)]
struct LatencySample {
    latency_ms: u64,
    timestamp: Instant,
}

/// Token Bucket para rate limiting
struct TokenBucket {
    /// Tokens disponibles actualmente
    tokens: AtomicUsize,
    /// Capacidad máxima del bucket
    capacity: usize,
    /// Tokens a añadir por intervalo
    tokens_per_interval: usize,
    /// Intervalo de recarga (ms)
    interval_ms: u64,
    /// Último tiempo de recarga
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: usize, tokens_per_second: u64) -> Self {
        // Dividir en intervalos de 100ms
        let interval_ms = 100u64;
        let tokens_per_interval = ((tokens_per_second as f64 / 1000.0) * interval_ms as f64) as usize;
        let tokens_per_interval = tokens_per_interval.max(1);

        Self {
            tokens: AtomicUsize::new(capacity),
            capacity,
            tokens_per_interval,
            interval_ms,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self, tokens: usize) -> bool {
        self.refill().await;

        let current = self.tokens.load(Ordering::Relaxed);
        if current >= tokens {
            let new_tokens = current - tokens;
            self.tokens.store(new_tokens, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    async fn refill(&self) {
        let mut last_refill = self.last_refill.lock().await;
        let elapsed = last_refill.elapsed();
        let intervals = elapsed.as_millis() as u64 / self.interval_ms;

        if intervals > 0 {
            let current = self.tokens.load(Ordering::Relaxed);
            let to_add = (self.tokens_per_interval * intervals as usize).min(self.capacity - current);
            self.tokens.fetch_add(to_add, Ordering::Relaxed);
            *last_refill = Instant::now();
        }
    }

    fn available_tokens(&self) -> usize {
        self.tokens.load(Ordering::Relaxed)
    }
}

/// Leaky Bucket para rate limiting
struct LeakyBucket {
    /// Cola de mensajes pendientes
    queue: Arc<Mutex<VecDeque<Instant>>>,
    /// Tasa de fuga (mensajes por segundo)
    leak_rate: f64,
    /// Capacidad máxima
    capacity: usize,
    /// Última fuga
    last_leak: Mutex<Instant>,
}

impl LeakyBucket {
    fn new(capacity: usize, leak_rate_per_second: u64) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            leak_rate: leak_rate_per_second as f64,
            capacity,
            last_leak: Mutex::new(Instant::now()),
        }
    }

    async fn try_consume(&self) -> bool {
        self.leak().await;

        let mut queue = self.queue.lock().await;
        if queue.len() < self.capacity {
            queue.push_back(Instant::now());
            true
        } else {
            false
        }
    }

    async fn leak(&self) {
        let mut last_leak = self.last_leak.lock().await;
        let elapsed = last_leak.elapsed().as_secs_f64();
        let to_leak = (elapsed * self.leak_rate) as usize;

        if to_leak > 0 {
            let mut queue = self.queue.lock().await;
            for _ in 0..to_leak.min(queue.len()) {
                queue.pop_front();
            }
            *last_leak = Instant::now();
        }
    }

    async fn queue_size(&self) -> usize {
        self.leak().await;
        self.queue.lock().await.len()
    }
}

/// Flow Controller real con rate limiting dinámico
pub struct FlowController {
    config: FlowConfig,
    token_bucket: Option<Arc<TokenBucket>>,
    leaky_bucket: Option<Arc<LeakyBucket>>,
    /// Semáforo para backpressure
    semaphore: Arc<Semaphore>,
    /// Cola de mensajes bufferizados
    buffer: Arc<Mutex<VecDeque<BufferedMessage>>>,
    /// Métricas
    metrics: Arc<FlowMetricsAtomic>,
    /// Historial de latencias
    latency_history: Arc<Mutex<VecDeque<LatencySample>>>,
    /// Tiempo de inicio
    start_time: Instant,
    /// Running
    running: Arc<std::sync::atomic::AtomicBool>,
}

/// Mensaje en buffer
#[derive(Debug, Clone)]
struct BufferedMessage {
    id: String,
    data: serde_json::Value,
    enqueued_at: Instant,
    domain: crate::actors::DivineDomain,
}

/// Métricas atómicas para thread-safety
struct FlowMetricsAtomic {
    messages_allowed: AtomicU64,
    messages_denied: AtomicU64,
    messages_sent: AtomicU64,
    messages_buffered: AtomicU64,
    messages_dropped: AtomicU64,
    bytes_sent: AtomicU64,
    total_latency_ns: AtomicU64,
    latency_samples: AtomicU64,
}

impl Default for FlowMetricsAtomic {
    fn default() -> Self {
        Self {
            messages_allowed: AtomicU64::new(0),
            messages_denied: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_buffered: AtomicU64::new(0),
            messages_dropped: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            latency_samples: AtomicU64::new(0),
        }
    }
}

impl FlowController {
    pub fn new(config: Option<FlowConfig>) -> Self {
        let config = config.unwrap_or_default();
        let max_buffer = config.max_buffer_size;

        let token_bucket = if config.algorithm == RateLimitAlgorithm::TokenBucket {
            Some(Arc::new(TokenBucket::new(
                config.max_burst_size,
                config.max_messages_per_second,
            )))
        } else {
            None
        };

        let leaky_bucket = if config.algorithm == RateLimitAlgorithm::LeakyBucket {
            Some(Arc::new(LeakyBucket::new(
                max_buffer,
                config.max_messages_per_second,
            )))
        } else {
            None
        };

        Self {
            config: config.clone(),
            token_bucket,
            leaky_bucket,
            semaphore: Arc::new(Semaphore::new(max_buffer)),
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer))),
            metrics: Arc::new(FlowMetricsAtomic::default()),
            latency_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            start_time: Instant::now(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Inicia el loop de métricas en background
    pub fn start_metrics_loop(&self) {
        if !self.config.enable_metrics {
            return;
        }

        let metrics = self.metrics.clone();
        let running = self.running.clone();
        let interval_secs = self.config.metrics_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            while running.load(Ordering::Relaxed) {
                ticker.tick().await;

                let allowed = metrics.messages_allowed.load(Ordering::Relaxed);
                let denied = metrics.messages_denied.load(Ordering::Relaxed);
                let sent = metrics.messages_sent.load(Ordering::Relaxed);
                let dropped = metrics.messages_dropped.load(Ordering::Relaxed);
                let buffered = metrics.messages_buffered.load(Ordering::Relaxed);

                let rate = if interval_secs > 0 {
                    (sent as f64) / (interval_secs as f64)
                } else {
                    0.0
                };

                debug!(
                    "📊 Flow Metrics - Allowed: {}, Denied: {}, Sent: {}, Dropped: {}, Buffered: {}, Rate: {:.2}/s",
                    allowed, denied, sent, dropped, buffered, rate
                );
            }
        });
    }

    /// Adquiere un permiso para enviar un mensaje (con semáforo y rate limiting)
    pub async fn acquire_permit(&self) -> Result<FlowPermit, FlowError> {
        let start = Instant::now();

        // 1. Rate limiting
        let allowed = match self.config.algorithm {
            RateLimitAlgorithm::TokenBucket => {
                if let Some(ref bucket) = self.token_bucket {
                    bucket.acquire(1).await
                } else {
                    true
                }
            }
            RateLimitAlgorithm::LeakyBucket => {
                if let Some(ref bucket) = self.leaky_bucket {
                    bucket.try_consume().await
                } else {
                    true
                }
            }
            _ => true, // Fixed y Sliding window no implementados aquí
        };

        if !allowed {
            self.metrics.messages_denied.fetch_add(1, Ordering::Relaxed);
            return Err(FlowError::RateLimited);
        }

        self.metrics.messages_allowed.fetch_add(1, Ordering::Relaxed);

        // 2. Backpressure con semáforo
        let permit = tokio::time::timeout(
            Duration::from_millis(self.config.acquire_timeout_ms),
            self.semaphore.acquire(),
        )
        .await
        .map_err(|_| FlowError::Timeout)?
        .map_err(|_| FlowError::SemaphoreClosed)?;

        let latency = start.elapsed();
        self.record_latency(latency).await;

        Ok(FlowPermit {
            _permit: permit,
            controller: self,
        })
    }

    pub fn try_acquire(&self) -> Option<FlowPermitImmediate> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                self.metrics.messages_allowed.fetch_add(1, Ordering::Relaxed);
                Some(FlowPermitImmediate {
                    _permit: permit,
                })
            }
            Err(_) => {
                self.metrics.messages_denied.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Registra la latencia de una operación
    async fn record_latency(&self, latency: Duration) {
        let ns = latency.as_nanos() as u64;
        self.metrics.total_latency_ns.fetch_add(ns, Ordering::Relaxed);
        self.metrics.latency_samples.fetch_add(1, Ordering::Relaxed);

        let mut history = self.latency_history.lock().await;
        history.push_back(LatencySample {
            latency_ms: latency.as_millis() as u64,
            timestamp: Instant::now(),
        });

        // Mantener solo últimas 1000 muestras
        while history.len() > 1000 {
            history.pop_front();
        }
    }

    /// Bufferiza un mensaje para envío posterior
    pub async fn buffer_message(
        &self,
        domain: crate::actors::DivineDomain,
        data: serde_json::Value,
    ) -> Result<String, FlowError> {
        let mut buffer = self.buffer.lock().await;

        if buffer.len() >= self.config.max_buffer_size {
            self.metrics.messages_dropped.fetch_add(1, Ordering::Relaxed);
            return Err(FlowError::BufferFull);
        }

        let id = uuid::Uuid::new_v4().to_string();
        let msg = BufferedMessage {
            id: id.clone(),
            data,
            enqueued_at: Instant::now(),
            domain,
        };

        buffer.push_back(msg);
        self.metrics.messages_buffered.fetch_add(1, Ordering::Relaxed);

        debug!("📦 Mensaje bufferizado: {} (buffer: {})", id, buffer.len());

        Ok(id)
    }

    /// Obtiene y remueve un mensaje del buffer
    pub async fn pop_buffered(&self) -> Option<(String, crate::actors::DivineDomain, serde_json::Value)> {
        let mut buffer = self.buffer.lock().await;

        if let Some(msg) = buffer.pop_front() {
            Some((msg.id, msg.domain, msg.data))
        } else {
            None
        }
    }

    /// Registra un mensaje enviado exitosamente
    pub fn record_sent(&self, bytes: usize) {
        self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.metrics.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Registra un mensaje droppeado
    pub fn record_dropped(&self) {
        self.metrics.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Obtiene métricas actuales
    pub async fn get_metrics(&self) -> FlowMetrics {
        let messages_allowed = self.metrics.messages_allowed.load(Ordering::Relaxed);
        let messages_denied = self.metrics.messages_denied.load(Ordering::Relaxed);
        let messages_sent = self.metrics.messages_sent.load(Ordering::Relaxed);
        let messages_buffered = self.metrics.messages_buffered.load(Ordering::Relaxed);
        let messages_dropped = self.metrics.messages_dropped.load(Ordering::Relaxed);
        let bytes_sent = self.metrics.bytes_sent.load(Ordering::Relaxed);
        let total_latency_ns = self.metrics.total_latency_ns.load(Ordering::Relaxed);
        let latency_samples = self.metrics.latency_samples.load(Ordering::Relaxed);

        let buffer = self.buffer.lock().await;
        let current_buffer_size = buffer.len();

        // Calcular rate actual (mensajes por segundo desde inicio)
        let elapsed_secs = self.start_time.elapsed().as_secs_f64().max(1.0);
        let current_rate = messages_sent as f64 / elapsed_secs;

        // Calcular latencia promedio
        let average_latency_ms = if latency_samples > 0 {
            (total_latency_ns as f64 / latency_samples as f64) / 1_000_000.0
        } else {
            0.0
        };

        // Calcular throughput
        let throughput_bps = if elapsed_secs > 0.0 {
            (bytes_sent as f64 * 8.0) / elapsed_secs
        } else {
            0.0
        };

        // Calcular nivel de backpressure
        let backpressure_level = if self.config.max_buffer_size > 0 {
            (current_buffer_size as f64) / (self.config.max_buffer_size as f64)
        } else {
            0.0
        };
        let backpressure_level = backpressure_level.min(1.0);
        let backpressure_active = backpressure_level >= self.config.backpressure_threshold;

        // Calcular latencia min/max de las últimas muestras
        let history = self.latency_history.lock().await;
        let (min_latency, max_latency) = history
            .iter()
            .map(|s| s.latency_ms)
            .fold((u64::MAX, u64::MIN), |(min, max), val| {
                (min.min(val), max.max(val))
            });

        FlowMetrics {
            messages_allowed,
            messages_denied,
            messages_sent,
            messages_buffered,
            messages_dropped,
            current_buffer_size,
            current_rate,
            average_latency_ms,
            min_latency_ms: if min_latency == u64::MAX { 0 } else { min_latency },
            max_latency_ms: if max_latency == u64::MIN { 0 } else { max_latency },
            backpressure_active,
            backpressure_level,
            throughput_bps,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verifica si hay backpressure activa
    pub async fn is_backpressure_active(&self) -> bool {
        let metrics = self.get_metrics().await;
        metrics.backpressure_active
    }

    /// Obtiene el nivel de backpressure (0.0 - 1.0)
    pub async fn backpressure_level(&self) -> f64 {
        let metrics = self.get_metrics().await;
        metrics.backpressure_level
    }

    /// Actualiza la configuración dinámicamente
    pub async fn update_config(&mut self, config: FlowConfig) {
        info!("🔄 Actualizando configuración de Flow Controller");

        // Recrear los buckets según el nuevo algoritmo
        self.token_bucket = if config.algorithm == RateLimitAlgorithm::TokenBucket {
            Some(Arc::new(TokenBucket::new(
                config.max_burst_size,
                config.max_messages_per_second,
            )))
        } else {
            None
        };

        self.leaky_bucket = if config.algorithm == RateLimitAlgorithm::LeakyBucket {
            Some(Arc::new(LeakyBucket::new(
                config.max_buffer_size,
                config.max_messages_per_second,
            )))
        } else {
            None
        };

        // Actualizar semáforo
        let new_permits = config.max_buffer_size.saturating_sub(self.config.max_buffer_size);
        if new_permits > 0 {
            self.semaphore = Arc::new(Semaphore::new(config.max_buffer_size));
        }

        self.config = config;
    }

    /// Limpia el buffer
    pub async fn clear_buffer(&self) {
        let mut buffer = self.buffer.lock().await;
        let dropped = buffer.len() as u64;
        buffer.clear();
        self.metrics.messages_dropped.fetch_add(dropped, Ordering::Relaxed);
        info!("🧹 Buffer limpiado, {} mensajes descartados", dropped);
    }

    /// Obtiene el tamaño del buffer
    pub async fn buffer_size(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Detiene el controller
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Verifica si está ejecutándose
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Clone for FlowController {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            token_bucket: self.token_bucket.clone(),
            leaky_bucket: self.leaky_bucket.clone(),
            semaphore: self.semaphore.clone(),
            buffer: self.buffer.clone(),
            metrics: self.metrics.clone(),
            latency_history: self.latency_history.clone(),
            start_time: self.start_time,
            running: self.running.clone(),
        }
    }
}

/// Permiso de flujo adquirido
pub struct FlowPermit<'a> {
    _permit: SemaphorePermit<'a>,
    controller: &'a FlowController,
}

impl<'a> FlowPermit<'a> {
    pub fn record_usage(self, bytes: usize) {
        self.controller.record_sent(bytes);
    }
}

/// Permiso inmediato (no async)
pub struct FlowPermitImmediate {
    _permit: OwnedSemaphorePermit,
}

/// Errores del Flow Controller
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowError {
    RateLimited,
    BufferFull,
    Timeout,
    SemaphoreClosed,
    InvalidConfig,
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowError::RateLimited => write!(f, "Rate limit exceeded"),
            FlowError::BufferFull => write!(f, "Buffer is full"),
            FlowError::Timeout => write!(f, "Acquire timeout"),
            FlowError::SemaphoreClosed => write!(f, "Semaphore closed"),
            FlowError::InvalidConfig => write!(f, "Invalid configuration"),
        }
    }
}

impl std::error::Error for FlowError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket() {
        let bucket = TokenBucket::new(10, 100); // 100 tokens/sec, burst 10

        // Debería poder adquirir 10 tokens inmediatamente
        for _ in 0..10 {
            assert!(bucket.acquire(1).await);
        }

        // El 11vo debería fallar (no hay tokens)
        assert!(!bucket.acquire(1).await);

        // Esperar para recargar
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(bucket.acquire(1).await);
    }

    #[tokio::test]
    async fn test_flow_controller_metrics() {
        let controller = FlowController::new(None);
        
        let metrics = controller.get_metrics().await;
        assert_eq!(metrics.messages_sent, 0);
        assert_eq!(metrics.backpressure_level, 0.0);
    }

    #[test]
    fn test_flow_config_default() {
        let config = FlowConfig::default();
        assert_eq!(config.max_messages_per_second, 10_000);
        assert!(config.backpressure_threshold > 0.0);
    }
}
