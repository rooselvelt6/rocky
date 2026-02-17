use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofaAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>,
    pub respiratory: i32,
    pub coagulation: i32,
    pub liver: i32,
    pub cardiovascular: i32,
    pub neurological: i32,
    pub renal: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}
