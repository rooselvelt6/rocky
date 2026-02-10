// src/actors/moirai/trajectories.rs
// OLYMPUS v15 - Analizador de Trayectorias Clínicas

use crate::actors::moirai::threads::{ThreadEvent, TrajectoryPoint, TrajectoryTrend};
use crate::errors::ActorError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Analizador de trayectorias
#[derive(Debug, Clone)]
pub struct TrajectoryAnalyzer;

impl TrajectoryAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analiza eventos y determina trayectoria
    pub fn analyze(&self, events: &[ThreadEvent]) -> Result<TrajectoryPoint, ActorError> {
        if events.len() < 2 {
            return Ok(self.default_point());
        }

        let evaluations: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                ThreadEvent::ScaleEvaluation {
                    scale_type,
                    score,
                    timestamp,
                } => Some((*timestamp, scale_type.clone(), *score)),
                _ => None,
            })
            .collect();

        if evaluations.len() < 2 {
            return Ok(self.default_point());
        }

        let trend = self.calculate_trend(&evaluations);
        let velocity = self.calculate_velocity(&evaluations);
        let severity = self.calculate_severity(&evaluations);

        Ok(TrajectoryPoint {
            timestamp: Utc::now(),
            trend,
            velocity,
            direction: self.trend_direction(trend),
            severity_score: severity,
            risk_factors: Vec::new(),
        })
    }

    fn calculate_trend(&self, evaluations: &[(DateTime<Utc>, String, i32)]) -> TrajectoryTrend {
        let sofa_scores: Vec<_> = evaluations
            .iter()
            .filter(|(_, scale, _)| scale == "SOFA")
            .map(|(ts, _, score)| (*ts, *score))
            .collect();

        if sofa_scores.len() < 2 {
            return TrajectoryTrend::Stable;
        }

        let first = sofa_scores[0].1 as f64;
        let last = sofa_scores[sofa_scores.len() - 1].1 as f64;
        let change = last - first;
        let hours = (sofa_scores[sofa_scores.len() - 1].0 - sofa_scores[0].0).num_hours() as f64;

        if hours < 1.0 {
            return TrajectoryTrend::Stable;
        }

        let rate = change / hours;

        match rate {
            r if r < -1.5 => TrajectoryTrend::ImprovingFast,
            r if r < -0.5 => TrajectoryTrend::ImprovingSlow,
            r if r > 1.5 => TrajectoryTrend::DeterioratingFast,
            r if r > 0.5 => TrajectoryTrend::DeterioratingSlow,
            _ => {
                if last > 15.0 {
                    TrajectoryTrend::Critical
                } else {
                    TrajectoryTrend::Stable
                }
            }
        }
    }

    fn calculate_velocity(&self, evaluations: &[(DateTime<Utc>, String, i32)]) -> f64 {
        let sofa: Vec<_> = evaluations
            .iter()
            .filter(|(_, scale, _)| scale == "SOFA")
            .map(|(_, _, score)| *score as f64)
            .collect();

        if sofa.len() < 2 {
            return 0.0;
        }

        let total_change = (sofa[sofa.len() - 1] - sofa[0]).abs();
        total_change / sofa.len() as f64
    }

    fn calculate_severity(&self, evaluations: &[(DateTime<Utc>, String, i32)]) -> f64 {
        let latest_sofa = evaluations
            .iter()
            .filter(|(_, scale, _)| scale == "SOFA")
            .map(|(_, _, score)| *score)
            .last()
            .unwrap_or(0);

        (latest_sofa as f64 / 24.0).min(1.0)
    }

    fn trend_direction(&self, trend: TrajectoryTrend) -> String {
        match trend {
            TrajectoryTrend::ImprovingFast => "Mejorando rápidamente",
            TrajectoryTrend::ImprovingSlow => "Mejorando lentamente",
            TrajectoryTrend::Stable => "Estable",
            TrajectoryTrend::DeterioratingSlow => "Empeorando lentamente",
            TrajectoryTrend::DeterioratingFast => "Empeorando rápidamente",
            TrajectoryTrend::Critical => "Crítico",
        }
        .to_string()
    }

    fn default_point(&self) -> TrajectoryPoint {
        TrajectoryPoint {
            timestamp: Utc::now(),
            trend: TrajectoryTrend::Stable,
            velocity: 0.0,
            direction: "Estable".to_string(),
            severity_score: 0.5,
            risk_factors: Vec::new(),
        }
    }
}

impl Default for TrajectoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
