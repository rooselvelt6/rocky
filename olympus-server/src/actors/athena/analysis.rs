// src/actors/athena/analysis.rs
// Clinical Analysis Engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::scales::{GlasgowResult, ApacheResult, SofaResult, News2Result};

/// Clinical Analysis Engine
#[derive(Debug, Clone)]
pub struct ClinicalAnalyzer;

impl ClinicalAnalyzer {
    /// Perform comprehensive multi-factorial risk analysis
    pub fn analyze_patient(data: PatientAnalysisData) -> PatientAnalysisResult {
        let mut critical_factors = Vec::new();
        let mut scale_correlations = HashMap::new();
        let mut recommendations = Vec::new();
        
        // Analyze Glasgow score
        if let Some(ref glasgow) = data.glasgow {
            scale_correlations.insert("glasgow".to_string(), glasgow.score as f64 / 15.0);
            if glasgow.score < 13 {
                critical_factors.push(format!("Altered consciousness (GCS: {})", glasgow.score));
                recommendations.push("Neurological monitoring every 1-2 hours".to_string());
            }
            if glasgow.score <= 8 {
                critical_factors.push("Severe brain injury - airway at risk".to_string());
                recommendations.push("Consider intubation for airway protection".to_string());
            }
        }

        // Analyze APACHE II score
        if let Some(ref apache) = data.apache {
            scale_correlations.insert("apache".to_string(), apache.score as f64 / 71.0);
            if apache.score >= 15 {
                critical_factors.push(format!("High APACHE II score: {} (mortality: {:.1}%)", 
                    apache.score, apache.predicted_mortality));
                recommendations.push("ICU admission strongly recommended".to_string());
            }
            if apache.predicted_mortality > 50.0 {
                critical_factors.push("Predicted mortality > 50%".to_string());
                recommendations.push("Discuss goals of care with family".to_string());
            }
        }

        // Analyze SOFA score
        if let Some(ref sofa) = data.sofa {
            scale_correlations.insert("sofa".to_string(), sofa.score as f64 / 24.0);
            if sofa.score >= 10 {
                critical_factors.push(format!("High organ dysfunction (SOFA: {})", sofa.score));
                recommendations.push("Aggressive organ support required".to_string());
            }
            
            // Track SOFA trend if available
            if let Some(previous_sofa) = data.previous_sofa_score {
                let delta = sofa.score as i16 - previous_sofa as i16;
                if delta >= 2 {
                    critical_factors.push(format!("Worsening organ function (SOFA +{})", delta));
                    recommendations.push("Escalate level of care urgently".to_string());
                }
            }
        }

        // Analyze NEWS2 score
        if let Some(ref news2) = data.news2 {
            scale_correlations.insert("news2".to_string(), news2.score as f64 / 20.0);
            if news2.score >= 7 {
                critical_factors.push(format!("High NEWS2 score: {}", news2.score));
                recommendations.push("Emergency assessment by critical care team".to_string());
            }
        }

        // Calculate overall risk score (0-100)
        let overall_risk = Self::calculate_overall_risk(&scale_correlations, &data);

        // Predict length of stay
        let predicted_los = Self::predict_length_of_stay(&data, overall_risk);

        // Add general recommendations based on overall risk
        if overall_risk >= 80.0 {
            recommendations.push("Consider transfer to tertiary care center".to_string());
            recommendations.push("Activate rapid response team".to_string());
        } else if overall_risk >= 60.0 {
            recommendations.push("Increase monitoring frequency to every 1-2 hours".to_string());
        }

        // Detect correlations between scales
        let correlation_insights = Self::analyze_correlations(&scale_correlations);
        
        PatientAnalysisResult {
            patient_id: data.patient_id,
            overall_risk,
            predicted_los,
            critical_factors,
            scale_correlations,
            recommendations,
            correlation_insights,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Calculate overall risk score (0-100)
    fn calculate_overall_risk(correlations: &HashMap<String, f64>, data: &PatientAnalysisData) -> f64 {
        let mut risk = 0.0;
        let mut weight_sum = 0.0;

        // Weight each scale appropriately
        if let Some(&glasgow_norm) = correlations.get("glasgow") {
            // Glasgow is inverted (lower is worse)
            risk += (1.0 - glasgow_norm) * 30.0; // 30% weight
            weight_sum += 30.0;
        }

        if let Some(&apache_norm) = correlations.get("apache") {
            risk += apache_norm * 35.0; // 35% weight
            weight_sum += 35.0;
        }

        if let Some(&sofa_norm) = correlations.get("sofa") {
            risk += sofa_norm * 25.0; // 25% weight
            weight_sum += 25.0;
        }

        if let Some(&news2_norm) = correlations.get("news2") {
            risk += news2_norm * 10.0; // 10% weight
            weight_sum += 10.0;
        }

        // Age factor
        if let Some(age) = data.age {
            let age_factor = match age {
                0..=44 => 0.0,
                45..=54 => 5.0,
                55..=64 => 10.0,
                65..=74 => 15.0,
                _ => 20.0,
            };
            risk += age_factor * 0.5; // Moderate age influence
        }

        // Normalize to 0-100
        if weight_sum > 0.0 {
            (risk / weight_sum * 100.0).min(100.0)
        } else {
            50.0 // Default moderate risk if no data
        }
    }

    /// Predict length of stay in days
    fn predict_length_of_stay(data: &PatientAnalysisData, overall_risk: f64) -> u32 {
        let mut base_los = match overall_risk {
            r if r >= 80.0 => 14,
            r if r >= 60.0 => 10,
            r if r >= 40.0 => 7,
            r if r >= 20.0 => 4,
            _ => 2,
        };

        // Adjust for APACHE score
        if let Some(ref apache) = data.apache {
            if apache.score >= 25 {
                base_los += 7;
            } else if apache.score >= 15 {
                base_los += 3;
            }
        }

        // Adjust for SOFA score
        if let Some(ref sofa) = data.sofa {
            if sofa.score >= 10 {
                base_los += 5;
            }
        }

        // Adjust for age
        if let Some(age) = data.age {
            if age >= 75 {
                base_los += 3;
            } else if age >= 65 {
                base_los += 2;
            }
        }

        base_los
    }

    /// Analyze correlations between different scales
    fn analyze_correlations(correlations: &HashMap<String, f64>) -> Vec<String> {
        let mut insights = Vec::new();

        // Check for concordance between scales
        if let (Some(&apache), Some(&sofa)) = (correlations.get("apache"), correlations.get("sofa")) {
            if (apache - sofa).abs() < 0.2 {
                insights.push("APACHE and SOFA scores concordant - reliable risk assessment".to_string());
            } else if apache > sofa + 0.3 {
                insights.push("APACHE higher than SOFA - consider chronic health burden".to_string());
            } else if sofa > apache + 0.3 {
                insights.push("SOFA higher than APACHE - acute organ dysfunction predominant".to_string());
            }
        }

        // Check neurological vs systemic
        if let (Some(&glasgow), Some(&sofa)) = (correlations.get("glasgow"), correlations.get("sofa")) {
            let glasgow_risk = 1.0 - glasgow; // Invert Glasgow
            if glasgow_risk > 0.6 && sofa < 0.4 {
                insights.push("Isolated neurological injury - systemic organs relatively preserved".to_string());
            }
        }

        // Check acute deterioration
        if let (Some(&news2), Some(&apache)) = (correlations.get("news2"), correlations.get("apache")) {
            if news2 > 0.7 && apache < 0.5 {
                insights.push("High NEWS2 with moderate APACHE - possible acute deterioration".to_string());
            }
        }

        insights
    }

    /// Identify trending patterns (requires historical data)
    pub fn analyze_trends(current: &PatientAnalysisResult, historical: &[PatientAnalysisResult]) -> TrendAnalysis {
        let mut trend = TrendAnalysis {
            direction: TrendDirection::Stable,
            velocity: 0.0,
            concerning_trends: Vec::new(),
            positive_trends: Vec::new(),
        };

        if historical.is_empty() {
            return trend;
        }

        // Calculate risk velocity (change per day)
        let time_span = historical.len() as f64;
        let risk_delta = current.overall_risk - historical[0].overall_risk;
        trend.velocity = risk_delta / time_span;

        // Determine direction
        trend.direction = if trend.velocity > 5.0 {
            TrendDirection::Deteriorating
        } else if trend.velocity < -5.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        // Identify concerning trends
        if trend.velocity > 10.0 {
            trend.concerning_trends.push("Rapid deterioration detected".to_string());
        }

        // Check for increasing critical factors
        if current.critical_factors.len() > historical.last().map(|h| h.critical_factors.len()).unwrap_or(0) {
            trend.concerning_trends.push("New critical factors emerged".to_string());
        }

        // Check for positive trends
        if trend.velocity < -5.0 {
            trend.positive_trends.push("Patient showing improvement".to_string());
        }

        if current.critical_factors.len() < historical.last().map(|h| h.critical_factors.len()).unwrap_or(100) {
            trend.positive_trends.push("Critical factors resolving".to_string());
        }

        trend
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAnalysisData {
    pub patient_id: String,
    pub age: Option<u8>,
    pub glasgow: Option<GlasgowResult>,
    pub apache: Option<ApacheResult>,
    pub sofa: Option<SofaResult>,
    pub news2: Option<News2Result>,
    pub previous_sofa_score: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAnalysisResult {
    pub patient_id: String,
    pub overall_risk: f64,
    pub predicted_los: u32,
    pub critical_factors: Vec<String>,
    pub scale_correlations: HashMap<String, f64>,
    pub recommendations: Vec<String>,
    pub correlation_insights: Vec<String>,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub velocity: f64, // Risk change per day
    pub concerning_trends: Vec<String>,
    pub positive_trends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Deteriorating,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::athena::scales::*;

    #[test]
    fn test_high_risk_analysis() {
        let glasgow = GlasgowCalculator::calculate(2, 2, 3);
        let sofa_params = SofaParams {
            pao2_fio2: 150,
            platelets: 40,
            bilirubin: 8.0,
            cardiovascular: "dopamine_high".to_string(),
            glasgow: 7,
            renal: "creatinine_very_high".to_string(),
        };
        let sofa = SofaCalculator::calculate(sofa_params);

        let data = PatientAnalysisData {
            patient_id: "test-001".to_string(),
            age: Some(75),
            glasgow: Some(glasgow),
            apache: None,
            sofa: Some(sofa),
            news2: None,
            previous_sofa_score: None,
        };

        let result = ClinicalAnalyzer::analyze_patient(data);
        assert!(result.overall_risk > 60.0);
        assert!(!result.critical_factors.is_empty());
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_low_risk_analysis() {
        let glasgow = GlasgowCalculator::calculate(4, 5, 6);
        
        let data = PatientAnalysisData {
            patient_id: "test-002".to_string(),
            age: Some(35),
            glasgow: Some(glasgow),
            apache: None,
            sofa: None,
            news2: None,
            previous_sofa_score: None,
        };

        let result = ClinicalAnalyzer::analyze_patient(data);
        assert!(result.overall_risk < 40.0);
    }
}
