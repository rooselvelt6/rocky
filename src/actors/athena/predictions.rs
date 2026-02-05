// src/actors/athena/predictions.rs
// Predictive Analytics Engine

use serde::{Deserialize, Serialize};
use super::analysis::{PatientAnalysisResult, TrendDirection};

/// Prediction Engine for clinical outcomes
#[derive(Debug, Clone)]
pub struct PredictionEngine;

impl PredictionEngine {
    /// Predict risk of deterioration
    pub fn predict_deterioration(analysis: &PatientAnalysisResult) -> DeteriorationPrediction {
        let mut probability = 0.0;
        let mut risk_factors = Vec::new();

        // Base probability from overall risk
        probability += analysis.overall_risk * 0.4; // 40% weight

        // Critical factors increase probability
        let critical_count = analysis.critical_factors.len() as f64;
        probability += (critical_count * 10.0).min(30.0); // Up to 30% from critical factors

        // Specific risk factors
        for factor in &analysis.critical_factors {
            if factor.contains("mortality") {
                probability += 15.0;
                risk_factors.push("High predicted mortality".to_string());
            }
            if factor.contains("Severe brain injury") {
                probability += 10.0;
                risk_factors.push("Severe neurological injury".to_string());
            }
            if factor.contains("organ dysfunction") {
                probability += 12.0;
                risk_factors.push("Multi-organ dysfunction".to_string());
            }
            if factor.contains("Worsening") {
                probability += 20.0;
                risk_factors.push("Deteriorating trend".to_string());
            }
        }

        // Scale-specific risk factors
        if let Some(&apache) = analysis.scale_correlations.get("apache") {
            if apache > 0.7 {
                probability += 10.0;
                risk_factors.push("Very high APACHE score".to_string());
            }
        }

        if let Some(&sofa) = analysis.scale_correlations.get("sofa") {
            if sofa > 0.6 {
                probability += 8.0;
                risk_factors.push("Significant organ dysfunction".to_string());
            }
        }

        // Cap at 95%
        probability = probability.min(95.0);

        // Calculate confidence interval (±10% for now, could be more sophisticated)
        let confidence_interval = (
            (probability - 10.0).max(0.0),
            (probability + 10.0).min(100.0),
        );

        let severity = match probability {
            p if p >= 70.0 => DeteriorationSeverity::Critical,
            p if p >= 50.0 => DeteriorationSeverity::High,
            p if p >= 30.0 => DeteriorationSeverity::Moderate,
            _ => DeteriorationSeverity::Low,
        };

        let time_window = Self::estimate_deterioration_window(probability);

        DeteriorationPrediction {
            probability,
            confidence_interval,
            severity,
            risk_factors,
            time_window,
            recommended_actions: Self::generate_deterioration_actions(probability),
        }
    }

    /// Predict recovery probability
    pub fn predict_recovery(analysis: &PatientAnalysisResult) -> RecoveryPrediction {
        // Recovery is inverse of risk
        let mut probability = 100.0 - analysis.overall_risk;

        // Adjust for critical factors (each reduces recovery chance)
        let critical_penalty = analysis.critical_factors.len() as f64 * 5.0;
        probability -= critical_penalty;

        // Positive indicators
        if analysis.critical_factors.is_empty() {
            probability += 10.0;
        }

        // Check for good prognostic signs
        if let Some(&glasgow) = analysis.scale_correlations.get("glasgow") {
            if glasgow > 0.8 { // GCS > 12
                probability += 10.0;
            }
        }

        probability = probability.clamp(5.0, 95.0);

        let confidence_interval = (
            (probability - 15.0).max(0.0),
            (probability + 15.0).min(100.0),
        );

        let expected_timeline = match probability {
            p if p >= 70.0 => "3-7 days".to_string(),
            p if p >= 50.0 => "1-2 weeks".to_string(),
            p if p >= 30.0 => "2-4 weeks".to_string(),
            _ => ">4 weeks or uncertain".to_string(),
        };

        RecoveryPrediction {
            probability,
            confidence_interval,
            expected_timeline,
            favorable_factors: Self::identify_favorable_factors(analysis),
            barriers_to_recovery: analysis.critical_factors.clone(),
        }
    }

    /// Predict patient trajectory over time
    pub fn predict_trajectory(
        analysis: &PatientAnalysisResult,
        time_horizon_hours: u32,
    ) -> TrajectoryPrediction {
        let deterioration = Self::predict_deterioration(analysis);
        let recovery = Self::predict_recovery(analysis);

        let predicted_outcome = if deterioration.probability > 60.0 {
            "Likely deterioration - escalation of care needed".to_string()
        } else if recovery.probability > 60.0 {
            "Likely improvement - continue current management".to_string()
        } else {
            "Uncertain trajectory - close monitoring required".to_string()
        };

        let key_indicators = Self::identify_key_indicators(analysis, time_horizon_hours);

        let monitoring_frequency = match deterioration.severity {
            DeteriorationSeverity::Critical => "Continuous monitoring".to_string(),
            DeteriorationSeverity::High => "Every 1 hour".to_string(),
            DeteriorationSeverity::Moderate => "Every 2-4 hours".to_string(),
            DeteriorationSeverity::Low => "Every 4-6 hours".to_string(),
        };

        TrajectoryPrediction {
            time_horizon_hours,
            predicted_outcome,
            deterioration_risk: deterioration.probability,
            recovery_probability: recovery.probability,
            key_indicators,
            monitoring_frequency,
            decision_points: Self::generate_decision_points(time_horizon_hours, &deterioration),
        }
    }

    // Helper functions

    fn estimate_deterioration_window(probability: f64) -> String {
        match probability {
            p if p >= 70.0 => "0-6 hours".to_string(),
            p if p >= 50.0 => "6-24 hours".to_string(),
            p if p >= 30.0 => "24-72 hours".to_string(),
            _ => ">72 hours".to_string(),
        }
    }

    fn generate_deterioration_actions(probability: f64) -> Vec<String> {
        let mut actions = Vec::new();

        if probability >= 70.0 {
            actions.push("Activate rapid response team immediately".to_string());
            actions.push("Prepare for ICU transfer".to_string());
            actions.push("Notify attending physician urgently".to_string());
            actions.push("Ensure resuscitation equipment at bedside".to_string());
        } else if probability >= 50.0 {
            actions.push("Increase monitoring frequency to hourly".to_string());
            actions.push("Alert ICU of potential admission".to_string());
            actions.push("Review and optimize current treatment".to_string());
        } else if probability >= 30.0 {
            actions.push("Increase monitoring to every 2-4 hours".to_string());
            actions.push("Ensure early warning score tracking".to_string());
        } else {
            actions.push("Continue standard monitoring".to_string());
        }

        actions
    }

    fn identify_favorable_factors(analysis: &PatientAnalysisResult) -> Vec<String> {
        let mut factors = Vec::new();

        if let Some(&glasgow) = analysis.scale_correlations.get("glasgow") {
            if glasgow > 0.8 {
                factors.push("Good neurological status".to_string());
            }
        }

        if analysis.critical_factors.is_empty() {
            factors.push("No critical risk factors identified".to_string());
        }

        if analysis.overall_risk < 40.0 {
            factors.push("Low overall risk score".to_string());
        }

        if factors.is_empty() {
            factors.push("Limited favorable factors - guarded prognosis".to_string());
        }

        factors
    }

    fn identify_key_indicators(analysis: &PatientAnalysisResult, _hours: u32) -> Vec<String> {
        let mut indicators = Vec::new();

        indicators.push("Glasgow Coma Scale".to_string());
        indicators.push("Vital signs (HR, BP, RR, SpO2)".to_string());
        indicators.push("Urine output".to_string());

        if analysis.scale_correlations.contains_key("sofa") {
            indicators.push("Organ function markers".to_string());
            indicators.push("Lactate levels".to_string());
        }

        if analysis.overall_risk > 60.0 {
            indicators.push("Arterial blood gas".to_string());
            indicators.push("Hemodynamic parameters".to_string());
        }

        indicators
    }

    fn generate_decision_points(hours: u32, deterioration: &DeteriorationPrediction) -> Vec<DecisionPoint> {
        let mut points = Vec::new();

        match deterioration.severity {
            DeteriorationSeverity::Critical => {
                points.push(DecisionPoint {
                    time_hours: 1,
                    decision: "Reassess need for ICU transfer".to_string(),
                    criteria: "Any further deterioration".to_string(),
                });
                points.push(DecisionPoint {
                    time_hours: 6,
                    decision: "Review treatment response".to_string(),
                    criteria: "Improvement in vital signs or scores".to_string(),
                });
            }
            DeteriorationSeverity::High => {
                points.push(DecisionPoint {
                    time_hours: 4,
                    decision: "Reassess clinical status".to_string(),
                    criteria: "Stability of vital signs".to_string(),
                });
                points.push(DecisionPoint {
                    time_hours: 12,
                    decision: "Determine level of care needed".to_string(),
                    criteria: "Trend in early warning scores".to_string(),
                });
            }
            _ => {
                points.push(DecisionPoint {
                    time_hours: hours.min(24),
                    decision: "Routine reassessment".to_string(),
                    criteria: "Overall clinical trajectory".to_string(),
                });
            }
        }

        points
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeteriorationPrediction {
    pub probability: f64,
    pub confidence_interval: (f64, f64),
    pub severity: DeteriorationSeverity,
    pub risk_factors: Vec<String>,
    pub time_window: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeteriorationSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPrediction {
    pub probability: f64,
    pub confidence_interval: (f64, f64),
    pub expected_timeline: String,
    pub favorable_factors: Vec<String>,
    pub barriers_to_recovery: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPrediction {
    pub time_horizon_hours: u32,
    pub predicted_outcome: String,
    pub deterioration_risk: f64,
    pub recovery_probability: f64,
    pub key_indicators: Vec<String>,
    pub monitoring_frequency: String,
    pub decision_points: Vec<DecisionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub time_hours: u32,
    pub decision: String,
    pub criteria: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_high_deterioration_risk() {
        let analysis = PatientAnalysisResult {
            patient_id: "test-001".to_string(),
            overall_risk: 80.0,
            predicted_los: 14,
            critical_factors: vec![
                "High predicted mortality".to_string(),
                "Severe brain injury".to_string(),
            ],
            scale_correlations: HashMap::new(),
            recommendations: vec![],
            correlation_insights: vec![],
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };

        let prediction = PredictionEngine::predict_deterioration(&analysis);
        assert!(prediction.probability > 50.0);
        assert_eq!(prediction.severity, DeteriorationSeverity::Critical);
    }

    #[test]
    fn test_recovery_prediction() {
        let analysis = PatientAnalysisResult {
            patient_id: "test-002".to_string(),
            overall_risk: 20.0,
            predicted_los: 3,
            critical_factors: vec![],
            scale_correlations: HashMap::new(),
            recommendations: vec![],
            correlation_insights: vec![],
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };

        let prediction = PredictionEngine::predict_recovery(&analysis);
        assert!(prediction.probability > 60.0);
    }
}
