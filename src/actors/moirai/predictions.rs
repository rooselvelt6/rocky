// src/actors/moirai/predictions.rs
// OLYMPUS v15 - Motor de Predicciones Clínicas

use crate::actors::moirai::threads::{FateOutcome, PatientThread};
use crate::errors::ActorError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Motor de predicciones
#[derive(Debug, Clone)]
pub struct PredictionEngine;

impl PredictionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Predice outcome basado en datos clínicos
    pub fn predict_outcome(
        &self,
        patient_id: &str,
        clinical_data: &serde_json::Value,
        prediction_type: PredictionType,
    ) -> Result<ClinicalPrediction, ActorError> {
        // Extraer scores
        let apache = clinical_data
            .get("apache_ii")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sofa = clinical_data
            .get("sofa")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let news2 = clinical_data
            .get("news2")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let saps = clinical_data
            .get("saps")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Calcular riesgo basado en scores
        let risk_score = self.calculate_risk_score(apache, sofa, news2, saps);

        // Determinar outcome predicho
        let predicted_outcome = if risk_score.mortality_risk > 0.8 {
            FateOutcome::Tragic
        } else if risk_score.mortality_risk < 0.2 && risk_score.recovery_probability > 0.7 {
            FateOutcome::Heroic
        } else {
            FateOutcome::Forgotten
        };

        let confidence = self.calculate_confidence(apache, sofa, saps);

        let prediction = ClinicalPrediction {
            id: format!("pred_{}_{}", patient_id, Utc::now().timestamp_millis()),
            patient_id: patient_id.to_string(),
            prediction_type,
            predicted_outcome,
            risk_assessment: risk_score,
            confidence,
            created_at: Utc::now(),
            valid_until: Utc::now() + Duration::hours(24),
            factors: self.identify_risk_factors(clinical_data),
        };

        Ok(prediction)
    }

    /// Calcula score de riesgo compuesto
    fn calculate_risk_score(
        &self,
        apache: f64,
        sofa: f64,
        news2: f64,
        saps: f64,
    ) -> RiskAssessment {
        // Normalizar scores a 0-1
        let apache_norm = (apache / 71.0).min(1.0); // APACHE II max 71
        let sofa_norm = (sofa / 24.0).min(1.0); // SOFA max 24
        let news2_norm = (news2 / 20.0).min(1.0); // NEWS2 max 20
        let saps_norm = (saps / 163.0).min(1.0); // SAPS II max 163

        // Ponderar (APACHE y SOFA tienen más peso)
        let mortality_risk =
            (apache_norm * 0.35 + sofa_norm * 0.30 + saps_norm * 0.20 + news2_norm * 0.15).min(1.0);

        let recovery_probability = 1.0 - mortality_risk;

        RiskAssessment {
            mortality_risk,
            recovery_probability,
            deterioration_risk: (sofa_norm * 0.5 + news2_norm * 0.5).min(1.0),
            readmission_risk: (apache_norm * 0.4 + saps_norm * 0.3).min(1.0),
        }
    }

    /// Calcula confianza de la predicción
    fn calculate_confidence(&self, apache: f64, sofa: f64, saps: f64) -> f64 {
        // Más datos = más confianza
        let data_completeness = if apache > 0.0 && sofa > 0.0 && saps > 0.0 {
            0.9
        } else if apache > 0.0 && sofa > 0.0 {
            0.75
        } else if apache > 0.0 {
            0.6
        } else {
            0.4
        };

        data_completeness
    }

    /// Identifica factores de riesgo
    fn identify_risk_factors(&self, clinical_data: &serde_json::Value) -> Vec<String> {
        let mut factors = Vec::new();

        if let Some(apache) = clinical_data.get("apache_ii").and_then(|v| v.as_f64()) {
            if apache > 20.0 {
                factors.push("APACHE II elevado".to_string());
            }
        }

        if let Some(sofa) = clinical_data.get("sofa").and_then(|v| v.as_f64()) {
            if sofa > 8.0 {
                factors.push("SOFA elevado".to_string());
            }
        }

        if let Some(news2) = clinical_data.get("news2").and_then(|v| v.as_f64()) {
            if news2 > 7.0 {
                factors.push("NEWS2 crítico".to_string());
            }
        }

        if let Some(age) = clinical_data.get("age").and_then(|v| v.as_f64()) {
            if age > 75.0 {
                factors.push("Edad avanzada".to_string());
            }
        }

        factors
    }

    /// Predice tiempo de estancia UCI
    pub fn predict_length_of_stay(&self, thread: &PatientThread) -> Result<Duration, ActorError> {
        let apache = thread.latest_apache().unwrap_or(10) as f64;
        let sofa = thread.latest_sofa().unwrap_or(5) as f64;

        // Fórmula simplificada: más score = más días
        let base_days = 3.0;
        let apache_factor = apache / 10.0;
        let sofa_factor = sofa / 5.0;

        let predicted_days = base_days + apache_factor + sofa_factor;

        Ok(Duration::days(predicted_days as i64))
    }

    /// Genera recomendaciones basadas en predicciones
    pub fn generate_recommendations(
        &self,
        thread: &PatientThread,
        predictions: &[ClinicalPrediction],
    ) -> Result<Vec<String>, ActorError> {
        let mut recommendations = Vec::new();

        if let Some(latest) = predictions.last() {
            if latest.risk_assessment.mortality_risk > 0.5 {
                recommendations.push("Considerar cuidados paliativos".to_string());
                recommendations.push("Revisar objetivos de tratamiento".to_string());
            }

            if latest.risk_assessment.deterioration_risk > 0.6 {
                recommendations.push("Monitoreo intensivo recomendado".to_string());
                recommendations.push("Preparar plan de contingencia".to_string());
            }

            if latest.predicted_outcome == FateOutcome::Heroic {
                recommendations.push("Continuar tratamiento actual".to_string());
                recommendations.push("Considerar desescalada".to_string());
            }
        }

        Ok(recommendations)
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Tipos de predicciones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionType {
    RiesgoMortality,
    RecoveryProbability,
    DeteriorationRisk,
    LengthOfStay,
    ReadmissionRisk,
}

/// Predicción clínica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalPrediction {
    pub id: String,
    pub patient_id: String,
    pub prediction_type: PredictionType,
    pub predicted_outcome: FateOutcome,
    pub risk_assessment: RiskAssessment,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub factors: Vec<String>,
}

/// Evaluación de riesgo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub mortality_risk: f64,
    pub recovery_probability: f64,
    pub deterioration_risk: f64,
    pub readmission_risk: f64,
}

impl RiskAssessment {
    /// Obtiene el riesgo general (máximo)
    pub fn overall_risk(&self) -> f64 {
        self.mortality_risk
            .max(self.deterioration_risk)
            .max(self.readmission_risk)
    }

    /// Verifica si es alto riesgo
    pub fn is_high_risk(&self) -> bool {
        self.overall_risk() > 0.7
    }
}
