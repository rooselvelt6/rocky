use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::sql::Thing;

#[cfg(not(feature = "ssr"))]
type Thing = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApacheAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub patient: Option<Thing>, // Reference to patient

    // Physiological parameters
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

    // Results
    pub score: u8,
    pub predicted_mortality: f32,
    pub severity: String,
    pub recommendation: String,

    // Metadata
    pub assessed_by: String,
    pub assessed_at: String, // ISO8601 timestamp
}

impl ApacheAssessment {
    pub fn new(
        temperature: f32,
        mean_arterial_pressure: i32,
        heart_rate: i32,
        respiratory_rate: i32,
        oxygenation_type: String,
        oxygenation_value: i32,
        arterial_ph: f32,
        serum_sodium: i32,
        serum_potassium: f32,
        serum_creatinine: f32,
        hematocrit: f32,
        white_blood_count: f32,
        glasgow_coma_score: u8,
        age: u8,
        chronic_health: String,
        score: u8,
        predicted_mortality: f32,
        severity: String,
        recommendation: String,
    ) -> Self {
        Self {
            id: None,
            patient: None,
            temperature,
            mean_arterial_pressure,
            heart_rate,
            respiratory_rate,
            oxygenation_type,
            oxygenation_value,
            arterial_ph,
            serum_sodium,
            serum_potassium,
            serum_creatinine,
            hematocrit,
            white_blood_count,
            glasgow_coma_score,
            age,
            chronic_health,
            score,
            predicted_mortality,
            severity,
            recommendation,
            assessed_by: "System".to_string(),
            assessed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
