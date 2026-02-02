/// Athena v12 - Diosa de la Sabiduría y Estrategia
/// Controladora central de escalas clínicas con IA

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalInsight {
    pub patient_id: String,
    pub scale_type: ScaleType,
    pub score: u16,
    pub severity: ClinicalSeverity,
    pub risk_level: f64,
    pub confidence: f64,
    pub recommendation: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleType {
    Glasgow,
    Apache,
    Sofa,
    Saps,
    News2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClinicalSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAnalysis {
    pub patient_id: String,
    pub overall_risk: f64,
    pub predicted_los: u32, // Length of Stay
    pub critical_factors: Vec<String>,
    pub scale_correlations: HashMap<String, f64>,
    pub trajectory_prediction: TrajectoryPrediction,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPrediction {
    pub time_horizon_hours: u32,
    pub deterioration_probability: f64,
    pub recovery_probability: f64,
    pub confidence_interval: (f64, f64),
    pub key_indicators: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AthenaV12 {
    insights: HashMap<String, Vec<ClinicalInsight>>,
    analysis_cache: HashMap<String, PatientAnalysis>,
}

impl AthenaV12 {
    pub fn new() -> Self {
        Self {
            insights: HashMap::new(),
            analysis_cache: HashMap::new(),
        }
    }

    pub async fn analyze_clinical_assessment(
        &mut self,
        patient_id: &str,
        scale_type: ScaleType,
        score: u16,
        additional_data: Option<serde_json::Value>,
    ) -> ClinicalInsight {
        let (severity, risk_level, recommendation) = self.calculate_clinical_metrics(&scale_type, score);
        
        let insight = ClinicalInsight {
            patient_id: patient_id.to_string(),
            scale_type,
            score,
            severity,
            risk_level,
            confidence: self.calculate_confidence(&scale_type, score, additional_data.as_ref()),
            recommendation,
            timestamp: Utc::now(),
        };
        
        // Guardar insight
        let patient_insights = self.insights.entry(patient_id.to_string()).or_insert_with(Vec::new);
        patient_insights.push(insight.clone());
        
        tracing::info!("🦉 Athena: Nuevo análisis clínico para paciente {} - {} = {} (Riesgo: {:.2})",
                     patient_id, format!("{:?}", scale_type), score, risk_level);
        
        insight
    }

    pub async fn get_patient_analysis(&mut self, patient_id: &str) -> PatientAnalysis {
        // Si ya existe en caché, retornar
        if let Some(cached) = self.analysis_cache.get(patient_id) {
            return cached.clone();
        }
        
        // Análisis completo del paciente
        let insights = self.insights.get(patient_id).unwrap_or(&Vec::new());
        
        let analysis = if insights.is_empty() {
            // Análisis sin datos
            PatientAnalysis {
                patient_id: patient_id.to_string(),
                overall_risk: 0.5,
                predicted_los: 3,
                critical_factors: vec![],
                scale_correlations: HashMap::new(),
                trajectory_prediction: TrajectoryPrediction {
                    time_horizon_hours: 24,
                    deterioration_probability: 0.1,
                    recovery_probability: 0.9,
                    confidence_interval: (0.8, 0.95),
                    key_indicators: vec![],
                },
                recommendations: vec![
                    "Realizar evaluación inicial completa".to_string(),
                    "Monitorear signos vitales".to_string(),
                ],
            }
        } else {
            // Análisis basado en datos existentes
            self.perform_comprehensive_analysis(patient_id, insights)
        };
        
        // Guardar en caché
        self.analysis_cache.insert(patient_id.to_string(), analysis.clone());
        
        analysis
    }

    fn calculate_clinical_metrics(&self, scale_type: &ScaleType, score: u16) -> (ClinicalSeverity, f64, String) {
        match (scale_type, score) {
            (ScaleType::Glasgow, score) => {
                let severity = if score >= 13 { ClinicalSeverity::Low }
                             else if score >= 9 { ClinicalSeverity::Moderate }
                             else if score >= 6 { ClinicalSeverity::High }
                             else { ClinicalSeverity::Critical };
                
                let risk_level = match score {
                    15 => 0.1, 14 => 0.15, 13 => 0.2,
                    12 => 0.25, 11 => 0.3, 10 => 0.4,
                    9 => 0.5, 8 => 0.6, 7 => 0.7,
                    6 => 0.8, 5 => 0.85, 4 => 0.9,
                    3 => 0.95, _ => 1.0,
                };
                
                let recommendation = match score {
                    15 => "Conciencia normal. Monitoreo rutinario".to_string(),
                    14..=13 => "Conciencia mínimamente alterada. Continuar observación".to_string(),
                    12..=9 => "Conciencia moderadamente alterada. Evaluar causa".to_string(),
                    8..=6 => "Conciencia gravemente alterada. Intervención urgente".to_string(),
                    _ => "Conciencia muy alterada o coma. Emergencia médica inmediata".to_string(),
                };
                
                (severity, risk_level, recommendation)
            }
            (ScaleType::Apache, score) => {
                let severity = if score <= 10 { ClinicalSeverity::Low }
                             else if score <= 20 { ClinicalSeverity::Moderate }
                             else if score <= 30 { ClinicalSeverity::High }
                             else { ClinicalSeverity::Critical };
                
                let mortality_risk = (score as f64 - 1.0) / 71.0 * 100.0;
                
                let recommendation = match score {
                    0..=10 => "Riesgo bajo. Cuidado estándar".to_string(),
                    11..=20 => "Riesgo moderado. Monitoreo cercano".to_string(),
                    21..=30 => "Riesgo alto. Considerar terapia intensiva".to_string(),
                    _ => "Riesgo muy alto. Atención crítica inmediata".to_string(),
                };
                
                (severity, mortality_risk, recommendation)
            }
            (ScaleType::Sofa, score) => {
                let severity = if score <= 6 { ClinicalSeverity::Low }
                             else if score <= 12 { ClinicalSeverity::Moderate }
                             else if score <= 18 { ClinicalSeverity::High }
                             else { ClinicalSeverity::Critical };
                
                let failure_risk = score as f64 / 24.0;
                
                let recommendation = match score {
                    0..=6 => "Falla orgánica mínima. Monitoreo rutinario".to_string(),
                    7..=12 => "Falla orgánica moderada. Optimizar soporte".to_string(),
                    13..=18 => "Falla orgánica severa. Intervención urgente".to_string(),
                    _ => "Falla multi-orgánica. Soporte vital máximo".to_string(),
                };
                
                (severity, failure_risk, recommendation)
            }
            (ScaleType::Saps, score) => {
                let severity = if score <= 20 { ClinicalSeverity::Low }
                             else if score <= 40 { ClinicalSeverity::Moderate }
                             else if score <= 60 { ClinicalSeverity::High }
                             else { ClinicalSeverity::Critical };
                
                let mortality_risk = match score {
                    0..=20 => 0.02, 21..=40 => 0.08,
                    41..=60 => 0.15, 61..=80 => 0.25,
                    _ => 0.40,
                };
                
                let recommendation = match score {
                    0..=20 => "Bajo riesgo. Cuidado general".to_string(),
                    21..=40 => "Riesgo moderado. Observación cercana".to_string(),
                    41..=60 => "Riesgo alto. Considerar UCI".to_string(),
                    _ => "Riesgo muy alto. UCI urgente".to_string(),
                };
                
                (severity, mortality_risk, recommendation)
            }
            (ScaleType::News2, score) => {
                let severity = if score <= 4 { ClinicalSeverity::Low }
                             else if score <= 6 { ClinicalSeverity::Moderate }
                             else if score <= 7 { ClinicalSeverity::High }
                             else { ClinicalSeverity::Critical };
                
                let risk_level = score as f64 / 15.0;
                
                let recommendation = match score {
                    0..=4 => "Bajo riesgo. Continuar monitoreo".to_string(),
                    5..=6 => "Riesgo bajo-medio. Aumentar frecuencia".to_string(),
                    7 => "Riesgo alto. Evaluación médica urgente".to_string(),
                    _ => "Riesgo crítico. Equipo de respuesta inmediata".to_string(),
                };
                
                (severity, risk_level, recommendation)
            }
        }
    }

    fn calculate_confidence(&self, _scale_type: &ScaleType, _score: u16, _additional_data: Option<&serde_json::Value>) -> f64 {
        // Para v12, usar confianza fija. En v13 esto sería basado en ML
        0.85
    }

    fn perform_comprehensive_analysis(&self, patient_id: &str, insights: &[ClinicalInsight]) -> PatientAnalysis {
        // Agrupar insights por tipo de escala
        let mut scale_scores: HashMap<ScaleType, Vec<u16>> = HashMap::new();
        
        for insight in insights {
            scale_scores.entry(insight.scale_type.clone()).or_insert_with(Vec::new).push(insight.score);
        }
        
        // Calcular correlaciones
        let mut scale_correlations = HashMap::new();
        if let (Some(glasgow_scores), Some(apache_scores)) = (scale_scores.get(&ScaleType::Glasgow), scale_scores.get(&ScaleType::Apache)) {
            if let (Some(&last_glasgow), Some(&last_apache)) = (glasgow_scores.last(), apache_scores.last()) {
                let correlation = self.calculate_glasgow_apache_correlation(*last_glasgow, *last_apache);
                scale_correlations.insert("glasgow_apache".to_string(), correlation);
            }
        }
        
        // Calcular riesgo general
        let overall_risk = self.calculate_overall_risk(insights);
        
        // Factores críticos
        let critical_factors = self.identify_critical_factors(insights);
        
        // Predicción de trayectoria
        let trajectory_prediction = self.predict_trajectory(insights, overall_risk);
        
        // Recomendaciones específicas
        let recommendations = self.generate_recommendations(insights, overall_risk);
        
        PatientAnalysis {
            patient_id: patient_id.to_string(),
            overall_risk,
            predicted_los: self.predict_los(overall_risk),
            critical_factors,
            scale_correlations,
            trajectory_prediction,
            recommendations,
        }
    }

    fn calculate_glasgow_apache_correlation(&self, glasgow: u16, apache: u16) -> f64 {
        // Correlación negativa esperada: menor Glasgow = mayor APACHE
        let expected_correlation = -0.7;
        
        // Normalizar scores
        let normalized_glasgow = (15.0 - glasgow as f64) / 12.0;
        let normalized_apache = apache as f64 / 71.0;
        
        // Correlación simple
        let correlation = if normalized_glasgow > 0.5 && normalized_apache > 0.5 {
            expected_correlation
        } else {
            0.3
        };
        
        correlation
    }

    fn calculate_overall_risk(&self, insights: &[ClinicalInsight]) -> f64 {
        if insights.is_empty() {
            return 0.5; // Riesgo moderado por defecto
        }
        
        let total_risk: f64 = insights.iter().map(|i| i.risk_level).sum();
        let avg_risk = total_risk / insights.len() as f64;
        
        // Ponderar por severidad
        let mut weighted_risk = 0.0;
        for insight in insights {
            let severity_weight = match insight.severity {
                ClinicalSeverity::Critical => 2.0,
                ClinicalSeverity::High => 1.5,
                ClinicalSeverity::Moderate => 1.0,
                ClinicalSeverity::Low => 0.5,
            };
            weighted_risk += insight.risk_level * severity_weight;
        }
        
        let avg_weighted_risk = weighted_risk / insights.len() as f64;
        
        (avg_risk + avg_weighted_risk) / 2.0
    }

    fn identify_critical_factors(&self, insights: &[ClinicalInsight]) -> Vec<String> {
        let mut factors = Vec::new();
        
        for insight in insights {
            if insight.risk_level > 0.7 {
                factors.push(format!("{} con alto riesgo: {} ({})", 
                    format!("{:?}", insight.scale_type), 
                    insight.score, 
                    insight.severity));
            }
        }
        
        factors
    }

    fn predict_trajectory(&self, insights: &[ClinicalInsight], overall_risk: f64) -> TrajectoryPrediction {
        let time_horizon = 24; // 24 horas
        let base_deterioration = overall_risk;
        
        let (deterioration_prob, recovery_prob) = if overall_risk < 0.3 {
            (0.1, 0.9)
        } else if overall_risk < 0.6 {
            (0.3, 0.7)
        } else {
            (0.6, 0.4)
        };
        
        let confidence_interval = (
            (deterioration_prob - 0.1).max(0.0),
            (deterioration_prob + 0.1).min(1.0),
        );
        
        TrajectoryPrediction {
            time_horizon_hours: time_horizon,
            deterioration_probability: deterioration_prob,
            recovery_probability: recovery_prob,
            confidence_interval,
            key_indicators: vec![
                "Riesgo general del paciente".to_string(),
                "Tendencia de scores clínicos".to_string(),
                "Factores de riesgo identificados".to_string(),
            ],
        }
    }

    fn predict_los(&self, overall_risk: f64) -> u32 {
        // Predicción simple de Length of Stay
        if overall_risk < 0.2 {
            2  // 2 días
        } else if overall_risk < 0.4 {
            4  // 4 días
        } else if overall_risk < 0.6 {
            7  // 1 semana
        } else if overall_risk < 0.8 {
            14 // 2 semanas
        } else {
            21 // 3 semanas
        }
    }

    fn generate_recommendations(&self, insights: &[ClinicalInsight], overall_risk: f64) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Recomendaciones basadas en riesgo general
        if overall_risk > 0.7 {
            recommendations.push("Monitoreo intensivo continuo requerido".to_string());
            recommendations.push("Considerar traslado a unidad de cuidados críticos".to_string());
            recommendations.push("Evaluación multidisciplinaria urgente".to_string());
        } else if overall_risk > 0.4 {
            recommendations.push("Monitoreo frecuente (cada 2-4 horas)".to_string());
            recommendations.push("Optimizar soporte orgánico".to_string());
            recommendations.push("Evaluar necesidad de intervenciones terapéuticas".to_string());
        } else {
            recommendations.push("Monitoreo estándar (cada 6-8 horas)".to_string());
            recommendations.push("Continuar tratamiento actual".to_string());
            recommendations.push("Preparar plan de alta progresivo".to_string());
        }
        
        // Recomendaciones específicas por escala
        for insight in insights {
            match insight.severity {
                ClinicalSeverity::Critical => {
                    recommendations.push(format!("Acción inmediata requerida por {}: {}", 
                        format!("{:?}", insight.scale_type), insight.recommendation));
                }
                ClinicalSeverity::High => {
                    recommendations.push(format!("Intervención prioritaria para {}: {}", 
                        format!("{:?}", insight.scale_type), insight.recommendation));
                }
                _ => {}
            }
        }
        
        recommendations
    }

    pub fn get_patient_insights(&self, patient_id: &str) -> Option<&Vec<ClinicalInsight>> {
        self.insights.get(patient_id)
    }

    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
        tracing::info!("🦉 Athena: Caché de análisis limpiado");
    }
}

impl Default for AthenaV12 {
    fn default() -> Self {
        Self::new()
    }
}