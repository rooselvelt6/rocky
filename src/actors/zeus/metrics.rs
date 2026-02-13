// src/actors/zeus/metrics.rs
// OLYMPUS v15 - Zeus Metrics
// Métricas avanzadas con históricos, Prometheus-compatible y alertas

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use tracing::{info, warn};

use super::GodName;

/// Métricas principales de Zeus
#[derive(Debug, Clone)]
pub struct ZeusMetrics {
    // Timestamps
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub last_metrics_export: chrono::DateTime<chrono::Utc>,

    // Contadores globales (Atomic)
    pub total_messages: Arc<AtomicU64>,
    pub total_errors: Arc<AtomicU64>,
    pub total_restarts: Arc<AtomicU64>,
    pub total_recoveries: Arc<AtomicU64>,
    pub total_panics: Arc<AtomicU64>,
    pub total_dead_letters: Arc<AtomicU64>,

    // Métricas de actores
    pub actor_metrics: Arc<RwLock<HashMap<GodName, ActorMetrics>>>,

    // Histórico temporal
    pub historical_data: Arc<RwLock<VecDeque<HistoricalSnapshot>>>,

    // Alertas basadas en thresholds
    pub alert_thresholds: Arc<RwLock<AlertThresholds>>,
    pub active_alerts: Arc<RwLock<Vec<MetricAlert>>>,

    // Métricas de sistema
    pub system_metrics: Arc<RwLock<SystemMetrics>>,

    // Métricas de la Trinidad
    pub trinity_metrics: Arc<RwLock<TrinityMetrics>>,

    // Configuración
    pub retention_hours: u64,
    pub snapshot_interval_seconds: u64,
}

/// Métricas por actor
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActorMetrics {
    pub actor: GodName,
    pub message_count: u64,
    pub error_count: u64,
    pub restart_count: u64,
    pub last_message_time: Option<chrono::DateTime<chrono::Utc>>,
    pub average_processing_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub uptime_seconds: u64,
    pub status: String,
}

/// Snapshot histórico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_messages: u64,
    pub total_errors: u64,
    pub total_restarts: u64,
    pub healthy_actors: usize,
    pub dead_actors: usize,
    pub system_load: f64,
    pub memory_usage_mb: f64,
}

/// Thresholds para alertas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_error_rate_percent: f64,
    pub max_restart_rate_per_minute: u64,
    pub max_memory_usage_mb: f64,
    pub max_cpu_usage_percent: f64,
    pub max_latency_ms: u64,
    pub min_healthy_actors_percent: f64,
    pub max_dead_letters_per_minute: u64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_error_rate_percent: 5.0,
            max_restart_rate_per_minute: 10,
            max_memory_usage_mb: 1024.0,
            max_cpu_usage_percent: 80.0,
            max_latency_ms: 1000,
            min_healthy_actors_percent: 80.0,
            max_dead_letters_per_minute: 100,
        }
    }
}

/// Alerta de métricas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAlert {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
    pub metric: String,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
    pub acknowledged: bool,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Métricas del sistema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub open_connections: u64,
    pub active_threads: u64,
    pub gc_collections: u64,
}

/// Métricas de la Trinidad (Zeus, Hades, Poseidón)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrinityMetrics {
    pub zeus_health_score: f64,
    pub hades_health_score: f64,
    pub poseidon_health_score: f64,
    pub trinity_status: TrinityStatus,
    pub last_trinity_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrinityStatus {
    AllHealthy,
    OneDegraded { actor: GodName },
    OneDown { actor: GodName },
    MultipleIssues,
    Critical,
}

impl Default for TrinityStatus {
    fn default() -> Self {
        TrinityStatus::AllHealthy
    }
}

impl ZeusMetrics {
    pub fn new() -> Self {
        let now = Utc::now();
        
        Self {
            start_time: now,
            last_health_check: now,
            last_metrics_export: now,
            
            total_messages: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            total_restarts: Arc::new(AtomicU64::new(0)),
            total_recoveries: Arc::new(AtomicU64::new(0)),
            total_panics: Arc::new(AtomicU64::new(0)),
            total_dead_letters: Arc::new(AtomicU64::new(0)),
            
            actor_metrics: Arc::new(RwLock::new(HashMap::new())),
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            
            alert_thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            trinity_metrics: Arc::new(RwLock::new(TrinityMetrics::default())),
            
            retention_hours: 24,
            snapshot_interval_seconds: 60,
        }
    }
    
    pub fn with_retention(retention_hours: u64) -> Self {
        let mut metrics = Self::new();
        metrics.retention_hours = retention_hours;
        metrics
    }

    // ==================== Operaciones Atómicas ====================

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

    pub fn increment_panics(&self) {
        self.total_panics.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_dead_letters(&self) {
        self.total_dead_letters.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_total_messages(&self) -> u64 {
        self.total_messages.load(Ordering::SeqCst)
    }

    pub fn get_total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::SeqCst)
    }

    pub fn get_total_restarts(&self) -> u64 {
        self.total_restarts.load(Ordering::SeqCst)
    }

    pub fn get_total_recoveries(&self) -> u64 {
        self.total_recoveries.load(Ordering::SeqCst)
    }

    pub fn get_error_rate(&self) -> f64 {
        let messages = self.get_total_messages() as f64;
        let errors = self.get_total_errors() as f64;
        
        if messages > 0.0 {
            (errors / messages) * 100.0
        } else {
            0.0
        }
    }

    // ==================== Métricas por Actor ====================

    pub async fn update_actor_metrics(&self, actor: GodName, update: ActorMetricsUpdate) {
        let mut metrics = self.actor_metrics.write().await;
        
        let actor_metrics = metrics.entry(actor).or_insert_with(|| ActorMetrics {
            actor,
            ..Default::default()
        });
        
        if let Some(count) = update.message_count {
            actor_metrics.message_count = count;
            actor_metrics.last_message_time = Some(Utc::now());
        }
        
        if let Some(errors) = update.error_count {
            actor_metrics.error_count = errors;
        }
        
        if let Some(restarts) = update.restart_count {
            actor_metrics.restart_count = restarts;
        }
        
        if let Some(avg_time) = update.average_processing_time_ms {
            actor_metrics.average_processing_time_ms = avg_time;
        }
        
        if let Some(memory) = update.memory_usage_mb {
            actor_metrics.memory_usage_mb = memory;
        }
        
        if let Some(cpu) = update.cpu_usage_percent {
            actor_metrics.cpu_usage_percent = cpu;
        }
        
        if let Some(uptime) = update.uptime_seconds {
            actor_metrics.uptime_seconds = uptime;
        }
        
        if let Some(status) = update.status {
            actor_metrics.status = status;
        }
    }

    pub async fn get_actor_metrics(&self, actor: GodName) -> Option<ActorMetrics> {
        let metrics = self.actor_metrics.read().await;
        metrics.get(&actor).cloned()
    }

    pub async fn get_all_actor_metrics(&self) -> Vec<ActorMetrics> {
        let metrics = self.actor_metrics.read().await;
        metrics.values().cloned().collect()
    }

    pub async fn remove_actor_metrics(&self, actor: GodName) {
        let mut metrics = self.actor_metrics.write().await;
        metrics.remove(&actor);
    }

    // ==================== Histórico ====================

    pub async fn record_snapshot(&self, healthy_actors: usize, dead_actors: usize) {
        let mut history = self.historical_data.write().await;
        let system = self.system_metrics.read().await;
        
        let snapshot = HistoricalSnapshot {
            timestamp: Utc::now(),
            total_messages: self.get_total_messages(),
            total_errors: self.get_total_errors(),
            total_restarts: self.get_total_restarts(),
            healthy_actors,
            dead_actors,
            system_load: system.cpu_usage_percent,
            memory_usage_mb: system.memory_usage_mb,
        };
        
        history.push_back(snapshot);
        
        // Mantener solo snapshots dentro del período de retención
        let cutoff = Utc::now() - chrono::Duration::hours(self.retention_hours as i64);
        while let Some(front) = history.front() {
            if front.timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
        
        // Limitar tamaño máximo
        while history.len() > 10000 {
            history.pop_front();
        }
    }

    pub async fn get_historical_data(
        &self, 
        since: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<usize>
    ) -> Vec<HistoricalSnapshot> {
        let history = self.historical_data.read().await;
        let mut data: Vec<_> = history.iter().cloned().collect();
        
        if let Some(since_time) = since {
            data.retain(|s| s.timestamp >= since_time);
        }
        
        if let Some(limit_count) = limit {
            data = data.into_iter().rev().take(limit_count).rev().collect();
        }
        
        data
    }

    pub async fn get_historical_summary(&self) -> HistoricalSummary {
        let history = self.historical_data.read().await;
        
        if history.is_empty() {
            return HistoricalSummary::default();
        }
        
        let total_snapshots = history.len();
        let avg_healthy: f64 = history.iter().map(|s| s.healthy_actors as f64).sum::<f64>() / total_snapshots as f64;
        let avg_errors: f64 = history.iter().map(|s| s.total_errors as f64).sum::<f64>() / total_snapshots as f64;
        let max_load = history.iter().map(|s| s.system_load).fold(0.0, f64::max);
        let max_memory = history.iter().map(|s| s.memory_usage_mb).fold(0.0, f64::max);
        
        HistoricalSummary {
            total_snapshots,
            avg_healthy_actors: avg_healthy,
            avg_error_count: avg_errors,
            peak_system_load: max_load,
            peak_memory_usage_mb: max_memory,
            time_range_hours: self.retention_hours,
        }
    }

    // ==================== Alertas ====================

    pub async fn check_thresholds(&self) -> Vec<MetricAlert> {
        let thresholds = self.alert_thresholds.read().await.clone();
        let mut alerts = Vec::new();
        let _now = Utc::now();
        
        // Verificar tasa de errores
        let error_rate = self.get_error_rate();
        if error_rate > thresholds.max_error_rate_percent {
            alerts.push(self.create_alert(
                &thresholds,
                AlertSeverity::Error,
                "error_rate",
                error_rate,
                thresholds.max_error_rate_percent,
                format!("Error rate is {:.2}% (threshold: {:.2}%)", error_rate, thresholds.max_error_rate_percent),
            ));
        }
        
        // Verificar uso de memoria
        let system = self.system_metrics.read().await;
        if system.memory_usage_mb > thresholds.max_memory_usage_mb {
            alerts.push(self.create_alert(
                &thresholds,
                AlertSeverity::Warning,
                "memory_usage",
                system.memory_usage_mb,
                thresholds.max_memory_usage_mb,
                format!("Memory usage is {:.0} MB (threshold: {:.0} MB)", system.memory_usage_mb, thresholds.max_memory_usage_mb),
            ));
        }
        
        // Verificar CPU
        if system.cpu_usage_percent > thresholds.max_cpu_usage_percent {
            alerts.push(self.create_alert(
                &thresholds,
                AlertSeverity::Warning,
                "cpu_usage",
                system.cpu_usage_percent,
                thresholds.max_cpu_usage_percent,
                format!("CPU usage is {:.1}% (threshold: {:.1}%)", system.cpu_usage_percent, thresholds.max_cpu_usage_percent),
            ));
        }
        
        // Añadir a alertas activas
        if !alerts.is_empty() {
            let mut active = self.active_alerts.write().await;
            for alert in &alerts {
                // Evitar duplicados
                if !active.iter().any(|a| a.metric == alert.metric && !a.resolved) {
                    active.push(alert.clone());
                }
            }
        }
        
        alerts
    }

    fn create_alert(
        &self,
        _thresholds: &AlertThresholds,
        severity: AlertSeverity,
        metric: &str,
        current_value: f64,
        threshold: f64,
        message: String,
    ) -> MetricAlert {
        MetricAlert {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            severity,
            metric: metric.to_string(),
            current_value,
            threshold,
            message,
            acknowledged: false,
            resolved: false,
            resolved_at: None,
        }
    }

    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.active_alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    pub async fn resolve_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.active_alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id && !a.resolved) {
            alert.resolved = true;
            alert.resolved_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    pub async fn get_active_alerts(&self) -> Vec<MetricAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.iter().filter(|a| !a.resolved).cloned().collect()
    }

    pub async fn get_all_alerts(&self, limit: Option<usize>) -> Vec<MetricAlert> {
        let alerts = self.active_alerts.read().await;
        let mut data: Vec<_> = alerts.iter().cloned().collect();
        data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(l) = limit {
            data.truncate(l);
        }
        
        data
    }

    pub async fn update_thresholds(&self, thresholds: AlertThresholds) {
        let mut t = self.alert_thresholds.write().await;
        *t = thresholds;
        info!("⚡ Zeus: Alert thresholds updated");
    }

    // ==================== Métricas de Sistema ====================

    pub async fn update_system_metrics(&self, metrics: SystemMetrics) {
        let mut system = self.system_metrics.write().await;
        *system = metrics;
    }

    pub async fn get_system_metrics(&self) -> SystemMetrics {
        self.system_metrics.read().await.clone()
    }

    // ==================== Métricas de la Trinidad ====================

    pub async fn update_trinity_metrics(&self, metrics: TrinityMetrics) {
        let mut trinity = self.trinity_metrics.write().await;
        *trinity = metrics;
    }

    pub async fn get_trinity_metrics(&self) -> TrinityMetrics {
        self.trinity_metrics.read().await.clone()
    }

    // ==================== Exportación Prometheus ====================

    pub async fn export_prometheus_format(&self) -> String {
        let mut output = String::new();
        let now = Utc::now();
        
        // Métricas globales
        output.push_str(&format!("# HELP olympus_messages_total Total messages processed\n"));
        output.push_str(&format!("# TYPE olympus_messages_total counter\n"));
        output.push_str(&format!("olympus_messages_total {}\n\n", self.get_total_messages()));
        
        output.push_str(&format!("# HELP olympus_errors_total Total errors\n"));
        output.push_str(&format!("# TYPE olympus_errors_total counter\n"));
        output.push_str(&format!("olympus_errors_total {}\n\n", self.get_total_errors()));
        
        output.push_str(&format!("# HELP olympus_restarts_total Total restarts\n"));
        output.push_str(&format!("# TYPE olympus_restarts_total counter\n"));
        output.push_str(&format!("olympus_restarts_total {}\n\n", self.get_total_restarts()));
        
        output.push_str(&format!("# HELP olympus_recoveries_total Total recoveries\n"));
        output.push_str(&format!("# TYPE olympus_recoveries_total counter\n"));
        output.push_str(&format!("olympus_recoveries_total {}\n\n", self.get_total_recoveries()));
        
        output.push_str(&format!("# HELP olympus_error_rate Error rate percentage\n"));
        output.push_str(&format!("# TYPE olympus_error_rate gauge\n"));
        output.push_str(&format!("olympus_error_rate {:.4}\n\n", self.get_error_rate()));
        
        output.push_str(&format!("# HELP olympus_uptime_seconds System uptime\n"));
        output.push_str(&format!("# TYPE olympus_uptime_seconds gauge\n"));
        output.push_str(&format!("olympus_uptime_seconds {}\n\n", (now - self.start_time).num_seconds()));
        
        // Métricas por actor
        let actor_metrics = self.actor_metrics.read().await;
        
        output.push_str(&format!("# HELP olympus_actor_messages Actor message count\n"));
        output.push_str(&format!("# TYPE olympus_actor_messages gauge\n"));
        for (actor, metrics) in actor_metrics.iter() {
            output.push_str(&format!("olympus_actor_messages{{actor=\"{}\"}} {}\n", actor, metrics.message_count));
        }
        output.push_str("\n");
        
        output.push_str(&format!("# HELP olympus_actor_errors Actor error count\n"));
        output.push_str(&format!("# TYPE olympus_actor_errors gauge\n"));
        for (actor, metrics) in actor_metrics.iter() {
            output.push_str(&format!("olympus_actor_errors{{actor=\"{}\"}} {}\n", actor, metrics.error_count));
        }
        output.push_str("\n");
        
        output.push_str(&format!("# HELP olympus_actor_memory_mb Actor memory usage\n"));
        output.push_str(&format!("# TYPE olympus_actor_memory_mb gauge\n"));
        for (actor, metrics) in actor_metrics.iter() {
            output.push_str(&format!("olympus_actor_memory_mb{{actor=\"{}\"}} {:.2}\n", actor, metrics.memory_usage_mb));
        }
        output.push_str("\n");
        
        // Métricas del sistema
        let system = self.system_metrics.read().await;
        output.push_str(&format!("# HELP olympus_system_memory_mb System memory usage\n"));
        output.push_str(&format!("# TYPE olympus_system_memory_mb gauge\n"));
        output.push_str(&format!("olympus_system_memory_mb {:.2}\n\n", system.memory_usage_mb));
        
        output.push_str(&format!("# HELP olympus_system_cpu_percent System CPU usage\n"));
        output.push_str(&format!("# TYPE olympus_system_cpu_percent gauge\n"));
        output.push_str(&format!("olympus_system_cpu_percent {:.2}\n\n", system.cpu_usage_percent));
        
        output.push_str(&format!("# HELP olympus_system_active_threads Active threads\n"));
        output.push_str(&format!("# TYPE olympus_system_active_threads gauge\n"));
        output.push_str(&format!("olympus_system_active_threads {}\n\n", system.active_threads));
        
        // Métricas de alertas
        let active_alerts = self.get_active_alerts().await;
        output.push_str(&format!("# HELP olympus_active_alerts Number of active alerts\n"));
        output.push_str(&format!("# TYPE olympus_active_alerts gauge\n"));
        output.push_str(&format!("olympus_active_alerts {}\n\n", active_alerts.len()));
        
        // Métricas de la Trinidad
        let trinity = self.trinity_metrics.read().await;
        output.push_str(&format!("# HELP olympus_trinity_zeus_health Zeus health score\n"));
        output.push_str(&format!("# TYPE olympus_trinity_zeus_health gauge\n"));
        output.push_str(&format!("olympus_trinity_zeus_health {:.2}\n\n", trinity.zeus_health_score));
        
        output.push_str(&format!("# HELP olympus_trinity_hades_health Hades health score\n"));
        output.push_str(&format!("# TYPE olympus_trinity_hades_health gauge\n"));
        output.push_str(&format!("olympus_trinity_hades_health {:.2}\n\n", trinity.hades_health_score));
        
        output.push_str(&format!("# HELP olympus_trinity_poseidon_health Poseidon health score\n"));
        output.push_str(&format!("# TYPE olympus_trinity_poseidon_health gauge\n"));
        output.push_str(&format!("olympus_trinity_poseidon_health {:.2}\n", trinity.poseidon_health_score));
        
        output
    }

    // ==================== Summary ====================

    pub async fn get_summary(&self) -> MetricsSummary {
        let actor_metrics = self.actor_metrics.read().await;
        let system = self.system_metrics.read().await;
        let active_alerts = self.get_active_alerts().await;
        let trinity = self.trinity_metrics.read().await;
        
        MetricsSummary {
            uptime_seconds: (Utc::now() - self.start_time).num_seconds() as u64,
            total_messages: self.get_total_messages(),
            total_errors: self.get_total_errors(),
            total_restarts: self.get_total_restarts(),
            total_recoveries: self.get_total_recoveries(),
            error_rate: self.get_error_rate(),
            total_actors: actor_metrics.len(),
            active_actors: actor_metrics.values().filter(|m| m.status == "running").count(),
            avg_recovery_time_ms: 0, // Se calcularía desde histórico
            dead_letters: self.total_dead_letters.load(Ordering::SeqCst),
            system_memory_mb: system.memory_usage_mb,
            system_cpu_percent: system.cpu_usage_percent,
            active_alerts: active_alerts.len(),
            trinity_status: trinity.trinity_status.clone(),
        }
    }

    // ==================== Background Tasks ====================

    pub fn start_snapshot_loop(&self, interval_seconds: u64) {
        let metrics = self.clone();
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
            
            loop {
                ticker.tick().await;
                
                let actor_metrics = metrics.actor_metrics.read().await;
                let healthy = actor_metrics.values().filter(|m| m.status == "running").count();
                let dead = actor_metrics.values().filter(|m| m.status == "dead").count();
                drop(actor_metrics);
                
                metrics.record_snapshot(healthy, dead).await;
                
                // Verificar thresholds
                let alerts = metrics.check_thresholds().await;
                if !alerts.is_empty() {
                    for alert in &alerts {
                        warn!("⚡ Zeus: Alert - {}: {}", alert.metric, alert.message);
                    }
                }
            }
        });
    }
}

/// Update de métricas de actor
#[derive(Debug, Clone, Default)]
pub struct ActorMetricsUpdate {
    pub message_count: Option<u64>,
    pub error_count: Option<u64>,
    pub restart_count: Option<u64>,
    pub average_processing_time_ms: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub uptime_seconds: Option<u64>,
    pub status: Option<String>,
}

/// Resumen de métricas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: u64,
    pub total_messages: u64,
    pub total_errors: u64,
    pub total_restarts: u64,
    pub total_recoveries: u64,
    pub error_rate: f64,
    pub total_actors: usize,
    pub active_actors: usize,
    pub avg_recovery_time_ms: u64,
    pub dead_letters: u64,
    pub system_memory_mb: f64,
    pub system_cpu_percent: f64,
    pub active_alerts: usize,
    pub trinity_status: TrinityStatus,
}

/// Resumen histórico
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoricalSummary {
    pub total_snapshots: usize,
    pub avg_healthy_actors: f64,
    pub avg_error_count: f64,
    pub peak_system_load: f64,
    pub peak_memory_usage_mb: f64,
    pub time_range_hours: u64,
}

impl Default for ZeusMetrics {
    fn default() -> Self {
        Self::new()
    }
}
