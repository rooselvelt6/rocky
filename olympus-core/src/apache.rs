use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApacheAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>,
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
    pub white_blood_cell: f32,
    pub glasgow_score: i32,
    pub age: i32,
    pub chronic_health: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}
