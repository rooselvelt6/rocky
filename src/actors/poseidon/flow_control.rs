// src/actors/poseidon/flow_control.rs
// OLYMPUS v13 - Poseidon Flow Controller
// Control de flujo de datos

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub messages_sent: u64,
    pub messages_buffered: u64,
    pub messages_dropped: u64,
    pub current_rate: f64,
    pub average_latency_ms: u64,
}

impl Default for FlowMetrics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_buffered: 0,
            messages_dropped: 0,
            current_rate: 0.0,
            average_latency_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlowController {
    metrics: Arc<FlowMetrics>,
    max_rate: Arc<AtomicU64>,
    current_load: Arc<AtomicUsize>,
    max_buffer_size: Arc<AtomicUsize>,
}

impl FlowController {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(FlowMetrics::default()),
            max_rate: Arc::new(AtomicU64::new(10000)), // 10k messages/sec
            current_load: Arc::new(AtomicUsize::new(0)),
            max_buffer_size: Arc::new(AtomicUsize::new(100000)),
        }
    }

    pub fn can_send(&self) -> bool {
        self.current_load.load(Ordering::Relaxed) < self.max_buffer_size.load(Ordering::Relaxed)
    }

    pub fn record_send(&self) {
        self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.current_load.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_buffer(&self) {
        self.metrics
            .messages_buffered
            .fetch_add(1, Ordering::Relaxed);
        self.current_load.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_drop(&self) {
        self.metrics
            .messages_dropped
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_complete(&self) {
        self.current_load.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn set_max_rate(&self, rate: u64) {
        self.max_rate.store(rate, Ordering::Relaxed);
    }

    pub fn set_max_buffer(&self, size: usize) {
        self.max_buffer_size.store(size, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> FlowMetrics {
        FlowMetrics {
            messages_sent: self.metrics.messages_sent.load(Ordering::Relaxed),
            messages_buffered: self.metrics.messages_buffered.load(Ordering::Relaxed),
            messages_dropped: self.metrics.messages_dropped.load(Ordering::Relaxed),
            current_rate: self.metrics.current_rate,
            average_latency_ms: self.metrics.average_latency_ms,
        }
    }
}
