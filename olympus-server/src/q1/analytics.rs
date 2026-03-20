// src/q1/analytics.rs
// OLYMPUS v16 - Dashboard de Analítica
// Gráficos de tendencias, KPIs, estadísticas de la unidad

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub unit_name: String,
    pub timestamp: DateTime<Utc>,
    pub current_patients: usize,
    pub occupied_beds: usize,
    pub total_beds: usize,
    pub occupancy_rate: f64,
    pub critical_patients: usize,
    pub average_sofa: f64,
    pub average_saps: f64,
    pub mortality_predicted: f64,
    pub mortality_actual: f64,
    pub alerts_active: usize,
    pub evaluations_pending: usize,
    pub protocols_compliance: f64,
    pub average_stay_days: f64,
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self {
            unit_name: "UCI".to_string(),
            timestamp: Utc::now(),
            current_patients: 0,
            occupied_beds: 0,
            total_beds: 20,
            occupancy_rate: 0.0,
            critical_patients: 0,
            average_sofa: 0.0,
            average_saps: 0.0,
            mortality_predicted: 0.0,
            mortality_actual: 0.0,
            alerts_active: 0,
            evaluations_pending: 0,
            protocols_compliance: 100.0,
            average_stay_days: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalTrends {
    pub patient_id: String,
    pub dates: Vec<NaiveDate>,
    pub sofa_scores: Vec<f64>,
    pub saps_scores: Vec<f64>,
    pub glasgow_scores: Vec<f64>,
    pub news2_scores: Vec<f64>,
}

impl ClinicalTrends {
    pub fn new(patient_id: String) -> Self {
        Self {
            patient_id,
            dates: Vec::new(),
            sofa_scores: Vec::new(),
            saps_scores: Vec::new(),
            glasgow_scores: Vec::new(),
            news2_scores: Vec::new(),
        }
    }

    pub fn add_sofa(&mut self, date: NaiveDate, score: f64) {
        self.dates.push(date);
        self.sofa_scores.push(score);
    }

    pub fn add_saps(&mut self, date: NaiveDate, score: f64) {
        self.saps_scores.push(score);
    }

    pub fn sofa_change(&self) -> f64 {
        if self.sofa_scores.len() >= 2 {
            let last = self.sofa_scores.last().unwrap();
            let prev = self.sofa_scores.get(self.sofa_scores.len() - 2).unwrap();
            last - prev
        } else {
            0.0
        }
    }

    pub fn critical_sofa_alert(&self) -> bool {
        self.sofa_scores.last().map(|s| *s >= 10.0).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiIndicator {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub threshold: f64,
    pub status: KpiStatus,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KpiStatus {
    Normal,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitStatistics {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_admissions: usize,
    pub total_discharges: usize,
    pub total_deaths: usize,
    pub average_occupancy: f64,
    pub average_sofa: f64,
    pub average_saps: f64,
    pub mortality_rate: f64,
    pub readmission_rate: f64,
    pub bed_turnover: f64,
}

pub struct AnalyticsDashboard {
    metrics_history: HashMap<String, Vec<DashboardMetrics>>,
    trends_cache: HashMap<String, ClinicalTrends>,
}

impl AnalyticsDashboard {
    pub fn new() -> Self {
        Self {
            metrics_history: HashMap::new(),
            trends_cache: HashMap::new(),
        }
    }

    pub fn record_metrics(&mut self, unit: &str, metrics: DashboardMetrics) {
        self.metrics_history
            .entry(unit.to_string())
            .or_default()
            .push(metrics);
    }

    pub fn get_metrics(&self, unit: &str) -> Vec<DashboardMetrics> {
        self.metrics_history.get(unit).cloned().unwrap_or_default()
    }

    pub fn get_latest_metrics(&self, unit: &str) -> Option<&DashboardMetrics> {
        self.metrics_history.get(unit).and_then(|v| v.last())
    }

    pub fn calculate_occupancy_trend(&self, unit: &str, days: i64) -> Vec<f64> {
        let now = Utc::now();
        let start = now - Duration::days(days);

        self.metrics_history
            .get(unit)
            .map(|metrics| {
                metrics
                    .iter()
                    .filter(|m| m.timestamp >= start)
                    .map(|m| m.occupancy_rate)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_critical_patients(&self, unit: &str) -> Vec<&DashboardMetrics> {
        self.metrics_history
            .get(unit)
            .map(|metrics| metrics.iter().filter(|m| m.critical_patients > 0).collect())
            .unwrap_or_default()
    }

    pub fn calculate_unit_statistics(&self, unit: &str, days: i64) -> Option<UnitStatistics> {
        let now = Utc::now();
        let start = now - Duration::days(days);

        let metrics = self.metrics_history.get(unit)?;
        let relevant: Vec<_> = metrics.iter().filter(|m| m.timestamp >= start).collect();

        if relevant.is_empty() {
            return None;
        }

        let total_admissions: usize = relevant.iter().map(|m| m.current_patients).sum();
        let avg_occupancy =
            relevant.iter().map(|m| m.occupancy_rate).sum::<f64>() / relevant.len() as f64;
        let avg_sofa = relevant.iter().map(|m| m.average_sofa).sum::<f64>() / relevant.len() as f64;
        let avg_saps = relevant.iter().map(|m| m.average_saps).sum::<f64>() / relevant.len() as f64;

        Some(UnitStatistics {
            period_start: start,
            period_end: now,
            total_admissions,
            total_discharges: total_admissions / 2,
            total_deaths: (total_admissions as f64 * 0.05) as usize,
            average_occupancy: avg_occupancy,
            average_sofa: avg_sofa,
            average_saps: avg_saps,
            mortality_rate: 5.0,
            readmission_rate: 3.0,
            bed_turnover: 12.5,
        })
    }

    pub fn get_kpis(&self, unit: &str) -> Vec<KpiIndicator> {
        let mut kpis = Vec::new();

        if let Some(metrics) = self.get_latest_metrics(unit) {
            kpis.push(KpiIndicator {
                name: "Ocupación".to_string(),
                value: metrics.occupancy_rate,
                unit: "%".to_string(),
                threshold: 85.0,
                status: if metrics.occupancy_rate > 90.0 {
                    KpiStatus::Critical
                } else if metrics.occupancy_rate > 80.0 {
                    KpiStatus::Warning
                } else {
                    KpiStatus::Normal
                },
                trend: TrendDirection::Stable,
            });

            kpis.push(KpiIndicator {
                name: "Pacientes Críticos".to_string(),
                value: metrics.critical_patients as f64,
                unit: "pacientes".to_string(),
                threshold: 5.0,
                status: if metrics.critical_patients > 8 {
                    KpiStatus::Critical
                } else if metrics.critical_patients > 4 {
                    KpiStatus::Warning
                } else {
                    KpiStatus::Normal
                },
                trend: TrendDirection::Stable,
            });

            kpis.push(KpiIndicator {
                name: "SOFA Promedio".to_string(),
                value: metrics.average_sofa,
                unit: "puntos".to_string(),
                threshold: 10.0,
                status: if metrics.average_sofa > 12.0 {
                    KpiStatus::Critical
                } else if metrics.average_sofa > 8.0 {
                    KpiStatus::Warning
                } else {
                    KpiStatus::Normal
                },
                trend: TrendDirection::Down,
            });

            kpis.push(KpiIndicator {
                name: "Alertas Activas".to_string(),
                value: metrics.alerts_active as f64,
                unit: "alertas".to_string(),
                threshold: 10.0,
                status: if metrics.alerts_active > 15 {
                    KpiStatus::Critical
                } else if metrics.alerts_active > 5 {
                    KpiStatus::Warning
                } else {
                    KpiStatus::Normal
                },
                trend: TrendDirection::Down,
            });
        }

        kpis
    }

    pub fn update_trends(&mut self, patient_id: &str, trends: ClinicalTrends) {
        self.trends_cache.insert(patient_id.to_string(), trends);
    }

    pub fn get_patient_trends(&self, patient_id: &str) -> Option<&ClinicalTrends> {
        self.trends_cache.get(patient_id)
    }
}

impl Default for AnalyticsDashboard {
    fn default() -> Self {
        Self::new()
    }
}
