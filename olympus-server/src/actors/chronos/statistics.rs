// src/actors/chronos/statistics.rs
// OLYMPUS v15 - Métricas y estadísticas del scheduler

use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Métricas del scheduler de Chronos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    /// Total de tareas programadas
    pub tasks_scheduled: u64,
    /// Total de tareas ejecutadas
    pub tasks_executed: u64,
    /// Total de ejecuciones exitosas
    pub tasks_successful: u64,
    /// Total de ejecuciones fallidas
    pub tasks_failed: u64,
    /// Total de tareas canceladas
    pub tasks_cancelled: u64,
    /// Historial de latencias de ejecución (ms)
    pub execution_latencies: VecDeque<u64>,
    /// Métricas por hora (timestamp -> métricas)
    pub hourly_metrics: HashMap<String, HourlyMetrics>,
    /// Timestamp de inicio
    pub started_at: DateTime<Utc>,
}

impl Default for SchedulerMetrics {
    fn default() -> Self {
        Self {
            tasks_scheduled: 0,
            tasks_executed: 0,
            tasks_successful: 0,
            tasks_failed: 0,
            tasks_cancelled: 0,
            execution_latencies: VecDeque::with_capacity(100),
            hourly_metrics: HashMap::new(),
            started_at: Utc::now(),
        }
    }
}

impl SchedulerMetrics {
    /// Registra la programación de una tarea
    pub fn record_scheduled(&mut self) {
        self.tasks_scheduled += 1;
        self.update_hourly_metrics(|h| h.tasks_scheduled += 1);
    }

    /// Registra una ejecución exitosa con latencia
    pub fn record_success(&mut self, latency_ms: u64) {
        self.tasks_executed += 1;
        self.tasks_successful += 1;

        // Mantener solo las últimas 100 latencias
        if self.execution_latencies.len() >= 100 {
            self.execution_latencies.pop_front();
        }
        self.execution_latencies.push_back(latency_ms);

        self.update_hourly_metrics(|h| {
            h.tasks_executed += 1;
            h.tasks_successful += 1;
        });
    }

    /// Registra una ejecución fallida
    pub fn record_failure(&mut self) {
        self.tasks_executed += 1;
        self.tasks_failed += 1;

        self.update_hourly_metrics(|h| {
            h.tasks_executed += 1;
            h.tasks_failed += 1;
        });
    }

    /// Registra una tarea cancelada
    pub fn record_cancelled(&mut self) {
        self.tasks_cancelled += 1;
        self.update_hourly_metrics(|h| h.tasks_cancelled += 1);
    }

    /// Calcula la latencia promedio
    pub fn average_latency(&self) -> f64 {
        if self.execution_latencies.is_empty() {
            0.0
        } else {
            let sum: u64 = self.execution_latencies.iter().sum();
            sum as f64 / self.execution_latencies.len() as f64
        }
    }

    /// Calcula la latencia percentil (p95)
    pub fn percentile_latency(&self, percentile: f64) -> u64 {
        if self.execution_latencies.is_empty() {
            return 0;
        }

        let mut sorted: Vec<u64> = self.execution_latencies.iter().cloned().collect();
        sorted.sort_unstable();

        let index = ((percentile / 100.0) * sorted.len() as f64) as usize;
        sorted.get(index).cloned().unwrap_or(0)
    }

    /// Calcula la tasa de éxito
    pub fn success_rate(&self) -> f64 {
        if self.tasks_executed == 0 {
            1.0 // 100% si no hay ejecuciones
        } else {
            self.tasks_successful as f64 / self.tasks_executed as f64
        }
    }

    /// Tiempo de actividad en segundos
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }

    /// Obtiene métricas de las últimas N horas
    pub fn recent_hourly_metrics(&self, hours: usize) -> Vec<(&String, &HourlyMetrics)> {
        let cutoff = Utc::now() - Duration::hours(hours as i64);

        self.hourly_metrics
            .iter()
            .filter(|(_, m)| m.hour >= cutoff)
            .collect()
    }

    /// Actualiza las métricas horarias actuales
    fn update_hourly_metrics<F>(&mut self, update: F)
    where
        F: FnOnce(&mut HourlyMetrics),
    {
        let hour_key = Utc::now().format("%Y-%m-%d-%H").to_string();
        let now = Utc::now();
        // Crear timestamp del inicio de la hora actual
        let hour_start = now - Duration::seconds(now.minute() as i64 * 60 + now.second() as i64);

        let metrics = self
            .hourly_metrics
            .entry(hour_key.clone())
            .or_insert_with(|| HourlyMetrics::new(hour_start));

        update(metrics);
    }
}

/// Métricas por hora
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetrics {
    /// Inicio de la hora
    pub hour: DateTime<Utc>,
    /// Tareas programadas en esta hora
    pub tasks_scheduled: u64,
    /// Tareas ejecutadas en esta hora
    pub tasks_executed: u64,
    /// Ejecuciones exitosas
    pub tasks_successful: u64,
    /// Ejecuciones fallidas
    pub tasks_failed: u64,
    /// Tareas canceladas
    pub tasks_cancelled: u64,
    /// Latencia promedio en esta hora
    pub average_latency_ms: f64,
}

impl HourlyMetrics {
    /// Crea nuevas métricas horarias
    pub fn new(hour: DateTime<Utc>) -> Self {
        Self {
            hour,
            tasks_scheduled: 0,
            tasks_executed: 0,
            tasks_successful: 0,
            tasks_failed: 0,
            tasks_cancelled: 0,
            average_latency_ms: 0.0,
        }
    }
}

/// Estadísticas de ejecución de tareas
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskExecutionStats {
    /// Total de ejecuciones
    pub total_executions: u64,
    /// Ejecuciones exitosas
    pub successful_executions: u64,
    /// Ejecuciones fallidas
    pub failed_executions: u64,
    /// Tiempo total de ejecución (ms)
    pub total_duration_ms: u64,
    /// Duración mínima (ms)
    pub min_duration_ms: u64,
    /// Duración máxima (ms)
    pub max_duration_ms: u64,
    /// Última ejecución
    pub last_execution: Option<DateTime<Utc>>,
}

impl TaskExecutionStats {
    /// Crea nuevas estadísticas
    pub fn new() -> Self {
        Self {
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            ..Default::default()
        }
    }

    /// Registra una ejecución exitosa
    pub fn record_success(&mut self, duration_ms: u64) {
        self.total_executions += 1;
        self.successful_executions += 1;
        self.total_duration_ms += duration_ms;
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
        self.last_execution = Some(Utc::now());
    }

    /// Registra una ejecución fallida
    pub fn record_failure(&mut self, duration_ms: u64) {
        self.total_executions += 1;
        self.failed_executions += 1;
        self.total_duration_ms += duration_ms;
        self.last_execution = Some(Utc::now());
    }

    /// Duración promedio
    pub fn average_duration(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.total_executions as f64
        }
    }

    /// Tasa de éxito
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            1.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }
}

/// Reporte de estado del scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerReport {
    /// Timestamp del reporte
    pub generated_at: DateTime<Utc>,
    /// Métricas generales
    pub metrics: SchedulerMetrics,
    /// Top tareas más ejecutadas
    pub top_tasks: Vec<TaskSummary>,
    /// Tareas con mayor tasa de fallo
    pub problematic_tasks: Vec<TaskSummary>,
    /// Recomendaciones
    pub recommendations: Vec<String>,
}

impl SchedulerReport {
    /// Genera un reporte completo
    pub fn generate(metrics: SchedulerMetrics) -> Self {
        let recommendations = Self::generate_recommendations(&metrics);

        Self {
            generated_at: Utc::now(),
            metrics,
            top_tasks: Vec::new(), // Se llenaría con datos adicionales
            problematic_tasks: Vec::new(),
            recommendations,
        }
    }

    /// Genera recomendaciones basadas en métricas
    fn generate_recommendations(metrics: &SchedulerMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Verificar tasa de éxito
        if metrics.success_rate() < 0.95 {
            recommendations.push(
                "La tasa de éxito está por debajo del 95%. Revisar tareas fallidas.".to_string(),
            );
        }

        // Verificar latencia
        let avg_latency = metrics.average_latency();
        if avg_latency > 5000.0 {
            recommendations.push(format!(
                "Latencia promedio elevada ({:.0}ms). Considerar optimizar tareas.",
                avg_latency
            ));
        }

        // Verificar tareas fallidas
        if metrics.tasks_failed > 0 && metrics.tasks_failed > metrics.tasks_executed / 10 {
            recommendations
                .push("Más del 10% de tareas están fallando. Investigar causas.".to_string());
        }

        recommendations
    }
}

/// Resumen de una tarea para reportes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub name: String,
    pub total_executions: u64,
    pub success_rate: f64,
    pub average_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_metrics() {
        let mut metrics = SchedulerMetrics::default();

        metrics.record_scheduled();
        metrics.record_scheduled();
        assert_eq!(metrics.tasks_scheduled, 2);

        metrics.record_success(1500);
        metrics.record_success(2000);
        assert_eq!(metrics.tasks_successful, 2);
        assert_eq!(metrics.average_latency(), 1750.0);

        metrics.record_failure();
        assert_eq!(metrics.tasks_failed, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_task_execution_stats() {
        let mut stats = TaskExecutionStats::new();

        stats.record_success(1000);
        stats.record_success(2000);
        stats.record_failure(500);

        assert_eq!(stats.total_executions, 3);
        assert_eq!(stats.successful_executions, 2);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.average_duration(), 3500.0 / 3.0);
    }

    #[test]
    fn test_hourly_metrics() {
        let mut metrics = SchedulerMetrics::default();

        metrics.record_scheduled();
        metrics.record_success(1000);

        let hour_key = Utc::now().format("%Y-%m-%d-%H").to_string();
        assert!(metrics.hourly_metrics.contains_key(&hour_key));
    }

    #[test]
    fn test_scheduler_report() {
        let mut metrics = SchedulerMetrics::default();

        // Crear métricas problemáticas
        for _ in 0..5 {
            metrics.record_scheduled();
        }
        for _ in 0..3 {
            metrics.record_failure();
        }
        for _ in 0..2 {
            metrics.record_success(6000); // Latencia alta
        }

        let report = SchedulerReport::generate(metrics);

        assert!(!report.recommendations.is_empty());
        assert!(report.recommendations.iter().any(|r| r.contains("éxito")));
        assert!(report
            .recommendations
            .iter()
            .any(|r| r.contains("Latencia")));
    }
}
