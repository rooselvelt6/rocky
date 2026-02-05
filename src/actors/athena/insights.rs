// src/actors/athena/insights.rs
// Clinical Insight Generator

use serde::{Deserialize, Serialize};
use super::analysis::{PatientAnalysisResult, TrendAnalysis, TrendDirection};
use super::predictions::{DeteriorationPrediction, RecoveryPrediction, DeteriorationSeverity};
use super::scales::{GlasgowResult, ApacheResult, SofaResult, News2Result};

/// Insight Generator for clinical decision support
#[derive(Debug, Clone)]
pub struct InsightGenerator;

impl InsightGenerator {
    /// Generate comprehensive clinical insights
    pub fn generate_insights(
        analysis: &PatientAnalysisResult,
        deterioration: &DeteriorationPrediction,
        recovery: &RecoveryPrediction,
        trend: Option<&TrendAnalysis>,
    ) -> ClinicalInsights {
        let mut insights = Vec::new();
        let mut alerts = Vec::new();
        let mut interventions = Vec::new();

        // Risk-based insights
        if analysis.overall_risk >= 80.0 {
            insights.push(ClinicalInsight {
                category: InsightCategory::Risk,
                priority: InsightPriority::Critical,
                title: "Critical Risk Level".to_string(),
                description: format!(
                    "Patient has very high overall risk score ({:.1}%). Immediate attention required.",
                    analysis.overall_risk
                ),
                evidence: analysis.critical_factors.clone(),
                recommendations: vec![
                    "Activate rapid response team".to_string(),
                    "Consider ICU transfer".to_string(),
                ],
            });
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                message: "CRITICAL: Patient at very high risk of deterioration".to_string(),
                action_required: "Immediate physician review required".to_string(),
            });
        } else if analysis.overall_risk >= 60.0 {
            insights.push(ClinicalInsight {
                category: InsightCategory::Risk,
                priority: InsightPriority::High,
                title: "High Risk Level".to_string(),
                description: format!(
                    "Patient has high overall risk score ({:.1}%).",
                    analysis.overall_risk
                ),
                evidence: analysis.critical_factors.clone(),
                recommendations: vec![
                    "Increase monitoring frequency".to_string(),
                    "Alert ICU of potential admission".to_string(),
                ],
            });
        }

        // Deterioration insights
        if deterioration.probability >= 70.0 {
            insights.push(ClinicalInsight {
                category: InsightCategory::Prediction,
                priority: InsightPriority::Critical,
                title: "High Deterioration Risk".to_string(),
                description: format!(
                    "Patient has {:.1}% probability of deterioration within {}.",
                    deterioration.probability, deterioration.time_window
                ),
                evidence: deterioration.risk_factors.clone(),
                recommendations: deterioration.recommended_actions.clone(),
            });
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                message: format!("HIGH RISK: {:.0}% chance of deterioration", deterioration.probability),
                action_required: "Prepare for escalation of care".to_string(),
            });
        } else if deterioration.probability >= 50.0 {
            insights.push(ClinicalInsight {
                category: InsightCategory::Prediction,
                priority: InsightPriority::High,
                title: "Moderate Deterioration Risk".to_string(),
                description: format!(
                    "Patient has {:.1}% probability of deterioration.",
                    deterioration.probability
                ),
                evidence: deterioration.risk_factors.clone(),
                recommendations: deterioration.recommended_actions.clone(),
            });
        }

        // Recovery insights
        if recovery.probability >= 70.0 {
            insights.push(ClinicalInsight {
                category: InsightCategory::Prognosis,
                priority: InsightPriority::Info,
                title: "Favorable Prognosis".to_string(),
                description: format!(
                    "Patient has {:.1}% probability of recovery. Expected timeline: {}.",
                    recovery.probability, recovery.expected_timeline
                ),
                evidence: recovery.favorable_factors.clone(),
                recommendations: vec![
                    "Continue current management".to_string(),
                    "Plan for step-down care".to_string(),
                ],
            });
        }

        // Trend insights
        if let Some(trend_data) = trend {
            match trend_data.direction {
                TrendDirection::Deteriorating => {
                    insights.push(ClinicalInsight {
                        category: InsightCategory::Trend,
                        priority: InsightPriority::High,
                        title: "Deteriorating Trend Detected".to_string(),
                        description: format!(
                            "Patient showing deterioration at {:.1} risk points per day.",
                            trend_data.velocity
                        ),
                        evidence: trend_data.concerning_trends.clone(),
                        recommendations: vec![
                            "Urgent review of treatment plan".to_string(),
                            "Consider escalation of care".to_string(),
                        ],
                    });
                    alerts.push(Alert {
                        severity: AlertSeverity::High,
                        message: "Patient condition worsening".to_string(),
                        action_required: "Review and adjust treatment plan".to_string(),
                    });
                }
                TrendDirection::Improving => {
                    insights.push(ClinicalInsight {
                        category: InsightCategory::Trend,
                        priority: InsightPriority::Info,
                        title: "Improving Trend".to_string(),
                        description: "Patient showing signs of improvement.".to_string(),
                        evidence: trend_data.positive_trends.clone(),
                        recommendations: vec![
                            "Continue current management".to_string(),
                            "Consider reducing intensity of monitoring if sustained".to_string(),
                        ],
                    });
                }
                TrendDirection::Stable => {
                    insights.push(ClinicalInsight {
                        category: InsightCategory::Trend,
                        priority: InsightPriority::Low,
                        title: "Stable Condition".to_string(),
                        description: "Patient condition stable.".to_string(),
                        evidence: vec!["No significant changes in risk score".to_string()],
                        recommendations: vec!["Continue current monitoring".to_string()],
                    });
                }
            }
        }

        // Correlation insights
        for correlation in &analysis.correlation_insights {
            insights.push(ClinicalInsight {
                category: InsightCategory::Analysis,
                priority: InsightPriority::Medium,
                title: "Scale Correlation".to_string(),
                description: correlation.clone(),
                evidence: vec![],
                recommendations: vec![],
            });
        }

        // Generate interventions
        interventions.extend(Self::generate_interventions(analysis, deterioration));

        // Sort insights by priority
        insights.sort_by(|a, b| {
            let a_val = match a.priority {
                InsightPriority::Critical => 0,
                InsightPriority::High => 1,
                InsightPriority::Medium => 2,
                InsightPriority::Low => 3,
                InsightPriority::Info => 4,
            };
            let b_val = match b.priority {
                InsightPriority::Critical => 0,
                InsightPriority::High => 1,
                InsightPriority::Medium => 2,
                InsightPriority::Low => 3,
                InsightPriority::Info => 4,
            };
            a_val.cmp(&b_val)
        });

        ClinicalInsights {
            patient_id: analysis.patient_id.clone(),
            insights,
            alerts,
            interventions,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Generate scale-specific insights
    pub fn generate_scale_insight(
        scale_type: ScaleType,
        glasgow: Option<&GlasgowResult>,
        apache: Option<&ApacheResult>,
        sofa: Option<&SofaResult>,
        news2: Option<&News2Result>,
    ) -> Option<ClinicalInsight> {
        match scale_type {
            ScaleType::Glasgow => {
                if let Some(g) = glasgow {
                    Some(ClinicalInsight {
                        category: InsightCategory::Scale,
                        priority: if g.score <= 8 {
                            InsightPriority::Critical
                        } else if g.score <= 12 {
                            InsightPriority::High
                        } else {
                            InsightPriority::Low
                        },
                        title: format!("Glasgow Coma Scale: {}", g.score),
                        description: format!("{} - {}", g.severity, g.diagnosis),
                        evidence: vec![
                            format!("Eye response: {}", "score"),
                            format!("Verbal response: {}", "score"),
                            format!("Motor response: {}", "score"),
                        ],
                        recommendations: vec![g.recommendation.clone()],
                    })
                } else {
                    None
                }
            }
            ScaleType::Apache => {
                if let Some(a) = apache {
                    Some(ClinicalInsight {
                        category: InsightCategory::Scale,
                        priority: if a.score >= 25 {
                            InsightPriority::Critical
                        } else if a.score >= 15 {
                            InsightPriority::High
                        } else {
                            InsightPriority::Medium
                        },
                        title: format!("APACHE II Score: {}", a.score),
                        description: format!(
                            "{} - Predicted mortality: {:.1}%",
                            a.severity, a.predicted_mortality
                        ),
                        evidence: vec![format!("Score: {}/71", a.score)],
                        recommendations: vec![a.recommendation.clone()],
                    })
                } else {
                    None
                }
            }
            ScaleType::Sofa => {
                if let Some(s) = sofa {
                    Some(ClinicalInsight {
                        category: InsightCategory::Scale,
                        priority: if s.score >= 10 {
                            InsightPriority::Critical
                        } else if s.score >= 7 {
                            InsightPriority::High
                        } else {
                            InsightPriority::Medium
                        },
                        title: format!("SOFA Score: {}", s.score),
                        description: format!("{} organ dysfunction", s.severity),
                        evidence: vec![format!("Score: {}/24", s.score)],
                        recommendations: vec![s.recommendation.clone()],
                    })
                } else {
                    None
                }
            }
            ScaleType::News2 => {
                if let Some(n) = news2 {
                    Some(ClinicalInsight {
                        category: InsightCategory::Scale,
                        priority: if n.score >= 7 {
                            InsightPriority::Critical
                        } else if n.score >= 5 {
                            InsightPriority::High
                        } else {
                            InsightPriority::Low
                        },
                        title: format!("NEWS2 Score: {}", n.score),
                        description: format!("Risk level: {:?}", n.risk_level),
                        evidence: vec![format!("Score: {}/20", n.score)],
                        recommendations: vec![n.recommendation.clone()],
                    })
                } else {
                    None
                }
            }
        }
    }

    fn generate_interventions(
        analysis: &PatientAnalysisResult,
        deterioration: &DeteriorationPrediction,
    ) -> Vec<Intervention> {
        let mut interventions = Vec::new();

        // Based on deterioration severity
        match deterioration.severity {
            DeteriorationSeverity::Critical => {
                interventions.push(Intervention {
                    category: InterventionCategory::Monitoring,
                    urgency: InterventionUrgency::Immediate,
                    description: "Initiate continuous monitoring".to_string(),
                    rationale: "Critical deterioration risk".to_string(),
                });
                interventions.push(Intervention {
                    category: InterventionCategory::Escalation,
                    urgency: InterventionUrgency::Immediate,
                    description: "Activate rapid response team".to_string(),
                    rationale: "Patient requires immediate expert assessment".to_string(),
                });
            }
            DeteriorationSeverity::High => {
                interventions.push(Intervention {
                    category: InterventionCategory::Monitoring,
                    urgency: InterventionUrgency::Urgent,
                    description: "Increase monitoring to hourly".to_string(),
                    rationale: "High risk of deterioration".to_string(),
                });
            }
            _ => {}
        }

        // Based on specific critical factors
        for factor in &analysis.critical_factors {
            if factor.contains("consciousness") || factor.contains("brain injury") {
                interventions.push(Intervention {
                    category: InterventionCategory::Assessment,
                    urgency: InterventionUrgency::Urgent,
                    description: "Neurological assessment every 1-2 hours".to_string(),
                    rationale: "Altered consciousness detected".to_string(),
                });
            }
            if factor.contains("organ dysfunction") {
                interventions.push(Intervention {
                    category: InterventionCategory::Treatment,
                    urgency: InterventionUrgency::Urgent,
                    description: "Optimize organ support".to_string(),
                    rationale: "Multi-organ dysfunction present".to_string(),
                });
            }
        }

        interventions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalInsights {
    pub patient_id: String,
    pub insights: Vec<ClinicalInsight>,
    pub alerts: Vec<Alert>,
    pub interventions: Vec<Intervention>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalInsight {
    pub category: InsightCategory,
    pub priority: InsightPriority,
    pub title: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightCategory {
    Risk,
    Prediction,
    Prognosis,
    Trend,
    Analysis,
    Scale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InsightPriority {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub message: String,
    pub action_required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub category: InterventionCategory,
    pub urgency: InterventionUrgency,
    pub description: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionCategory {
    Monitoring,
    Assessment,
    Treatment,
    Escalation,
    Consultation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterventionUrgency {
    Immediate,
    Urgent,
    Soon,
    Routine,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScaleType {
    Glasgow,
    Apache,
    Sofa,
    News2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_critical_insights() {
        let analysis = PatientAnalysisResult {
            patient_id: "test-001".to_string(),
            overall_risk: 85.0,
            predicted_los: 14,
            critical_factors: vec!["High predicted mortality".to_string()],
            scale_correlations: HashMap::new(),
            recommendations: vec![],
            correlation_insights: vec![],
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };

        let deterioration = DeteriorationPrediction {
            probability: 75.0,
            confidence_interval: (65.0, 85.0),
            severity: DeteriorationSeverity::Critical,
            risk_factors: vec![],
            time_window: "0-6 hours".to_string(),
            recommended_actions: vec![],
        };

        let recovery = RecoveryPrediction {
            probability: 20.0,
            confidence_interval: (10.0, 30.0),
            expected_timeline: ">4 weeks".to_string(),
            favorable_factors: vec![],
            barriers_to_recovery: vec![],
        };

        let insights = InsightGenerator::generate_insights(&analysis, &deterioration, &recovery, None);
        
        assert!(!insights.insights.is_empty());
        assert!(!insights.alerts.is_empty());
        assert_eq!(insights.insights[0].priority, InsightPriority::Critical);
    }
}
