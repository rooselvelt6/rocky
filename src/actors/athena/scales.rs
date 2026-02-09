// src/actors/athena/scales.rs
// Clinical Scale Calculation Engine

use serde::{Deserialize, Serialize};
use crate::models::{
    glasgow::GlasgowAssessment,
    apache::ApacheAssessment,
    sofa::SofaAssessment,
    news2::{News2Assessment, ConsciousnessLevel, News2RiskLevel},
};

/// Glasgow Coma Scale Calculator
#[derive(Debug, Clone)]
pub struct GlasgowCalculator;

impl GlasgowCalculator {
    pub fn calculate(eye: u8, verbal: u8, motor: u8) -> GlasgowResult {
        let score = eye + verbal + motor;
        
        let (severity, diagnosis, recommendation) = match score {
            3..=8 => (
                "Severe".to_string(),
                "Severe brain injury".to_string(),
                "Immediate ICU admission, intubation likely required".to_string(),
            ),
            9..=12 => (
                "Moderate".to_string(),
                "Moderate brain injury".to_string(),
                "Close monitoring, consider ICU admission".to_string(),
            ),
            13..=14 => (
                "Mild".to_string(),
                "Mild brain injury".to_string(),
                "Frequent neurological assessments, monitor for deterioration".to_string(),
            ),
            15 => (
                "Normal".to_string(),
                "Normal consciousness".to_string(),
                "Continue routine monitoring".to_string(),
            ),
            _ => (
                "Invalid".to_string(),
                "Invalid score".to_string(),
                "Verify assessment parameters".to_string(),
            ),
        };

        GlasgowResult {
            score,
            severity,
            diagnosis: diagnosis.clone(),
            recommendation: recommendation.clone(),
            assessment: GlasgowAssessment::new(
                eye,
                verbal,
                motor,
                score,
                diagnosis.clone(),
                recommendation.clone(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlasgowResult {
    pub score: u8,
    pub severity: String,
    pub diagnosis: String,
    pub recommendation: String,
    pub assessment: GlasgowAssessment,
}

/// APACHE II Score Calculator
#[derive(Debug, Clone)]
pub struct ApacheCalculator;

impl ApacheCalculator {
    pub fn calculate(params: ApacheParams) -> ApacheResult {
        let mut score = 0u8;

        // Temperature (rectal)
        score += match params.temperature {
            t if t >= 41.0 => 4,
            t if t >= 39.0 => 3,
            t if t >= 38.5 => 1,
            t if t >= 36.0 => 0,
            t if t >= 34.0 => 1,
            t if t >= 32.0 => 2,
            t if t >= 30.0 => 3,
            _ => 4,
        };

        // Mean Arterial Pressure
        score += match params.mean_arterial_pressure {
            map if map >= 160 => 4,
            map if map >= 130 => 3,
            map if map >= 110 => 2,
            map if map >= 70 => 0,
            map if map >= 50 => 2,
            _ => 4,
        };

        // Heart Rate
        score += match params.heart_rate {
            hr if hr >= 180 => 4,
            hr if hr >= 140 => 3,
            hr if hr >= 110 => 2,
            hr if hr >= 70 => 0,
            hr if hr >= 55 => 2,
            hr if hr >= 40 => 3,
            _ => 4,
        };

        // Respiratory Rate
        score += match params.respiratory_rate {
            rr if rr >= 50 => 4,
            rr if rr >= 35 => 3,
            rr if rr >= 25 => 1,
            rr if rr >= 12 => 0,
            rr if rr >= 10 => 1,
            rr if rr >= 6 => 2,
            _ => 4,
        };

        // Oxygenation (A-aDO2 or PaO2)
        score += match params.oxygenation_value {
            o if o >= 500 => 4,
            o if o >= 350 => 3,
            o if o >= 200 => 2,
            _ => 0,
        };

        // Arterial pH
        score += match params.arterial_ph {
            ph if ph >= 7.7 => 4,
            ph if ph >= 7.6 => 3,
            ph if ph >= 7.5 => 1,
            ph if ph >= 7.33 => 0,
            ph if ph >= 7.25 => 2,
            ph if ph >= 7.15 => 3,
            _ => 4,
        };

        // Serum Sodium
        score += match params.serum_sodium {
            na if na >= 180 => 4,
            na if na >= 160 => 3,
            na if na >= 155 => 2,
            na if na >= 150 => 1,
            na if na >= 130 => 0,
            na if na >= 120 => 2,
            na if na >= 111 => 3,
            _ => 4,
        };

        // Serum Potassium
        score += match params.serum_potassium {
            k if k >= 7.0 => 4,
            k if k >= 6.0 => 3,
            k if k >= 5.5 => 1,
            k if k >= 3.5 => 0,
            k if k >= 3.0 => 1,
            k if k >= 2.5 => 2,
            _ => 4,
        };

        // Serum Creatinine (double points if acute renal failure)
        let creat_score = match params.serum_creatinine {
            cr if cr >= 3.5 => 4,
            cr if cr >= 2.0 => 3,
            cr if cr >= 1.5 => 2,
            _ => 0,
        };
        score += creat_score;

        // Hematocrit
        score += match params.hematocrit {
            hct if hct >= 60.0 => 4,
            hct if hct >= 50.0 => 2,
            hct if hct >= 46.0 => 1,
            hct if hct >= 30.0 => 0,
            hct if hct >= 20.0 => 2,
            _ => 4,
        };

        // White Blood Count
        score += match params.white_blood_count {
            wbc if wbc >= 40.0 => 4,
            wbc if wbc >= 20.0 => 2,
            wbc if wbc >= 15.0 => 1,
            wbc if wbc >= 3.0 => 0,
            wbc if wbc >= 1.0 => 2,
            _ => 4,
        };

        // Glasgow Coma Score (15 - GCS)
        score += 15 - params.glasgow_coma_score;

        // Age points
        score += match params.age {
            age if age >= 75 => 6,
            age if age >= 65 => 5,
            age if age >= 55 => 3,
            age if age >= 45 => 2,
            _ => 0,
        };

        // Chronic health points
        score += match params.chronic_health.as_str() {
            "elective_surgery" => 2,
            "emergency_surgery" | "non_operative" => 5,
            _ => 0,
        };

        // Calculate mortality
        let predicted_mortality = Self::calculate_mortality(score);

        let (severity, recommendation) = match score {
            0..=4 => ("Low risk".to_string(), "Standard monitoring".to_string()),
            5..=9 => ("Low-moderate risk".to_string(), "Increased monitoring frequency".to_string()),
            10..=14 => ("Moderate risk".to_string(), "Consider ICU admission".to_string()),
            15..=19 => ("Moderate-high risk".to_string(), "ICU admission recommended".to_string()),
            20..=24 => ("High risk".to_string(), "ICU admission required, aggressive treatment".to_string()),
            25..=29 => ("Very high risk".to_string(), "ICU required, consider advanced life support".to_string()),
            30..=34 => ("Extremely high risk".to_string(), "ICU required, maximal intervention".to_string()),
            _ => ("Critical".to_string(), "ICU required, discuss goals of care".to_string()),
        };

        ApacheResult {
            score,
            predicted_mortality,
            severity: severity.clone(),
            recommendation: recommendation.clone(),
            assessment: ApacheAssessment::new(
                params.temperature,
                params.mean_arterial_pressure,
                params.heart_rate,
                params.respiratory_rate,
                params.oxygenation_type,
                params.oxygenation_value,
                params.arterial_ph,
                params.serum_sodium,
                params.serum_potassium,
                params.serum_creatinine,
                params.hematocrit,
                params.white_blood_count,
                params.glasgow_coma_score,
                params.age,
                params.chronic_health,
                score,
                predicted_mortality,
                severity.clone(),
                recommendation.clone(),
            ),
        }
    }

    fn calculate_mortality(score: u8) -> f32 {
        // APACHE II mortality prediction (approximate)
        match score {
            0..=4 => 4.0,
            5..=9 => 8.0,
            10..=14 => 15.0,
            15..=19 => 25.0,
            20..=24 => 40.0,
            25..=29 => 55.0,
            30..=34 => 73.0,
            _ => 85.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApacheParams {
    pub temperature: f32,
    pub mean_arterial_pressure: i32,
    pub heart_rate: i32,
    pub respiratory_rate: i32,
    pub oxygenation_type: String,
    pub oxygenation_value: i32,
    pub arterial_ph: f32,
    pub serum_sodium: i32,
    pub serum_potassium: f32,
    pub serum_creatinine: f32,
    pub hematocrit: f32,
    pub white_blood_count: f32,
    pub glasgow_coma_score: u8,
    pub age: u8,
    pub chronic_health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApacheResult {
    pub score: u8,
    pub predicted_mortality: f32,
    pub severity: String,
    pub recommendation: String,
    pub assessment: ApacheAssessment,
}

/// SOFA Score Calculator
#[derive(Debug, Clone)]
pub struct SofaCalculator;

impl SofaCalculator {
    pub fn calculate(params: SofaParams) -> SofaResult {
        let mut score = 0u8;

        // Respiration (PaO2/FiO2)
        score += match params.pao2_fio2 {
            pf if pf < 100 => 4,
            pf if pf < 200 => 3,
            pf if pf < 300 => 2,
            pf if pf < 400 => 1,
            _ => 0,
        };

        // Coagulation (Platelets)
        score += match params.platelets {
            plt if plt < 20 => 4,
            plt if plt < 50 => 3,
            plt if plt < 100 => 2,
            plt if plt < 150 => 1,
            _ => 0,
        };

        // Liver (Bilirubin)
        score += match params.bilirubin {
            bil if bil >= 12.0 => 4,
            bil if bil >= 6.0 => 3,
            bil if bil >= 2.0 => 2,
            bil if bil >= 1.2 => 1,
            _ => 0,
        };

        // Cardiovascular (MAP or vasopressors)
        score += match params.cardiovascular.as_str() {
            "dopamine_high" | "epi_high" | "norepi_high" => 4,
            "dopamine_low" | "dobutamine" => 3,
            "map_low" => 2,
            "map_normal" => 0,
            _ => 0,
        };

        // CNS (Glasgow Coma Score)
        score += match params.glasgow {
            gcs if gcs < 6 => 4,
            gcs if gcs < 10 => 3,
            gcs if gcs < 13 => 2,
            gcs if gcs < 15 => 1,
            _ => 0,
        };

        // Renal (Creatinine or urine output)
        score += match params.renal.as_str() {
            "creatinine_very_high" => 4,
            "creatinine_high" => 3,
            "creatinine_moderate" => 2,
            "creatinine_mild" => 1,
            _ => 0,
        };

        let (severity, recommendation) = match score {
            0..=6 => ("Low risk".to_string(), "Standard ICU care".to_string()),
            7..=9 => ("Moderate risk".to_string(), "Increased monitoring, optimize organ support".to_string()),
            10..=12 => ("High risk".to_string(), "Aggressive organ support, consider escalation".to_string()),
            _ => ("Very high risk".to_string(), "Maximal organ support, discuss prognosis".to_string()),
        };

        SofaResult {
            score,
            severity: severity.clone(),
            recommendation: recommendation.clone(),
            assessment: SofaAssessment::new(
                params.pao2_fio2,
                params.platelets,
                params.bilirubin,
                params.cardiovascular,
                params.glasgow,
                params.renal,
                score,
                severity,
                recommendation,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofaParams {
    pub pao2_fio2: i32,
    pub platelets: i32,
    pub bilirubin: f32,
    pub cardiovascular: String,
    pub glasgow: u8,
    pub renal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofaResult {
    pub score: u8,
    pub severity: String,
    pub recommendation: String,
    pub assessment: SofaAssessment,
}

/// NEWS2 Score Calculator
#[derive(Debug, Clone)]
pub struct News2Calculator;

impl News2Calculator {
    pub fn calculate(params: News2Params) -> News2Result {
        let mut assessment = News2Assessment {
            id: None,
            patient_id: params.patient_id,
            assessed_at: chrono::Utc::now().to_rfc3339(),
            respiration_rate: params.respiration_rate,
            spo2_scale: params.spo2_scale,
            spo2: params.spo2,
            air_or_oxygen: params.air_or_oxygen,
            systolic_bp: params.systolic_bp,
            heart_rate: params.heart_rate,
            consciousness: params.consciousness,
            temperature: params.temperature,
            score: 0,
            risk_level: News2RiskLevel::Low,
        };

        assessment.calculate_score();

        let recommendation = match assessment.risk_level {
            News2RiskLevel::Low => "Continue routine monitoring".to_string(),
            News2RiskLevel::LowMedium => "Urgent review by ward-based doctor, consider increasing monitoring to hourly".to_string(),
            News2RiskLevel::Medium => "Urgent review by ward-based doctor or acute team nurse, increase monitoring frequency".to_string(),
            News2RiskLevel::High => "Emergency assessment by critical care team, transfer to higher level of care".to_string(),
        };

        News2Result {
            score: assessment.score,
            risk_level: assessment.risk_level.clone(),
            recommendation,
            assessment,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct News2Params {
    pub patient_id: String,
    pub respiration_rate: u8,
    pub spo2_scale: u8,
    pub spo2: u8,
    pub air_or_oxygen: bool,
    pub systolic_bp: u16,
    pub heart_rate: u16,
    pub consciousness: ConsciousnessLevel,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct News2Result {
    pub score: u8,
    pub risk_level: News2RiskLevel,
    pub recommendation: String,
    pub assessment: News2Assessment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glasgow_normal() {
        let result = GlasgowCalculator::calculate(4, 5, 6);
        assert_eq!(result.score, 15);
        assert_eq!(result.severity, "Normal");
    }

    #[test]
    fn test_glasgow_severe() {
        let result = GlasgowCalculator::calculate(1, 1, 2);
        assert_eq!(result.score, 4);
        assert_eq!(result.severity, "Severe");
    }

    #[test]
    fn test_sofa_low_risk() {
        let params = SofaParams {
            pao2_fio2: 450,
            platelets: 200,
            bilirubin: 0.8,
            cardiovascular: "map_normal".to_string(),
            glasgow: 15,
            renal: "normal".to_string(),
        };
        let result = SofaCalculator::calculate(params);
        assert_eq!(result.score, 0);
        assert_eq!(result.severity, "Low risk");
    }
}
