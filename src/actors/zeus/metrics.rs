// src/actors/zeus/metrics.rs
// OLYMPUS v13 - Zeus Metrics
// Métricas globales del Olimpo

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Métricas de Zeus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeusMetrics {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,

    pub total_messages: AtomicU64,
    pub total_errors: AtomicU64,
    pub total_restarts: AtomicU64,
    pub total_recoveries: AtomicU64,

    pub healthy_actors: usize,
    pub dead_actors: usize,
    pub degraded_actors: usize,

    pub average_recovery_time_ms: AtomicU64,
    pub last_recovery_time_ms: AtomicU64,

    pub dead_letters_count: AtomicU64,

    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl Default for ZeusMetrics {
    fn default() -> Self {
        Self {
            start_time: chrono::Utc::now(),
            last_health_check: chrono::Utc::now(),
            total_messages: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_restarts: AtomicU64::new(0),
            total_recoveries: AtomicU64::new(0),
            healthy_actors: 0,
            dead_actors: 0,
            degraded_actors: 0,
            average_recovery_time_ms: AtomicU64::new(0),
            last_recovery_time_ms: AtomicU64::new(0),
            dead_letters_count: AtomicU64::new(0),
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
        }
    }
}

impl ZeusMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_messages(&self) {
        self.total_messages.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_restarts(&self) {
        self.total_restarts.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_recoveries(&self) {
        self.total_recoveries.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_dead_letters(&self) {
        self.dead_letters_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_recovery_time(&self, ms: u64) {
        self.last_recovery_time_ms.store(ms, Ordering::SeqCst);

        // Update average
        let total = self.total_recoveries.load(Ordering::SeqCst);
        let current_avg = self.average_recovery_time_ms.load(Ordering::SeqCst);
        let new_avg = if total > 0 {
            ((current_avg * (total - 1)) + ms) / total
        } else {
            ms
        };
        self.average_recovery_time_ms
            .store(new_avg, Ordering::SeqCst);
    }

    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            uptime_seconds: (chrono::Utc::now() - self.start_time).num_seconds() as u64,
            total_messages: self.total_messages.load(Ordering::SeqCst),
            total_errors: self.total_errors.load(Ordering::SeqCst),
            total_restarts: self.total_restarts.load(Ordering::SeqCst),
            total_recoveries: self.total_recoveries.load(Ordering::SeqCst),
            healthy_actors: self.healthy_actors,
            dead_actors: self.dead_actors,
            avg_recovery_time_ms: self.average_recovery_time_ms.load(Ordering::SeqCst),
            dead_letters: self.dead_letters_count.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: u64,
    pub total_messages: u64,
    pub total_errors: u64,
    pub total_restarts: u64,
    pub total_recoveries: u64,
    pub healthy_actors: usize,
    pub dead_actors: usize,
    pub avg_recovery_time_ms: u64,
    pub dead_letters: u64,
}
