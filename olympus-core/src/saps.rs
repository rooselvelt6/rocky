use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapsAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>,
    pub age: i32,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub temperature: i32,
    pub pao2_fio2: i32,
    pub urinary_output: i32,
    pub blood_urea: i32,
    pub white_blood_cell: i32,
    pub potassium: i32,
    pub sodium: i32,
    pub bicarbonate: i32,
    pub bilirubin: f32,
    pub glasgow_score: i32,
    pub chronic_disease: bool,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}
