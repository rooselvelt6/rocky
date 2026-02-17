use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlasgowAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>,
    pub eye_response: u8,
    pub verbal_response: u8,
    pub motor_response: u8,
    pub score: u8,
    pub diagnosis: String,
    pub recommendation: String,
    pub assessed_by: String,
    pub assessed_at: String,
}

impl GlasgowAssessment {
    pub fn new(
        eye: u8,
        verbal: u8,
        motor: u8,
        score: u8,
        diagnosis: String,
        recommendation: String,
    ) -> Self {
        Self {
            id: None,
            patient_id: None,
            eye_response: eye,
            verbal_response: verbal,
            motor_response: motor,
            score,
            diagnosis,
            recommendation,
            assessed_by: "System".to_string(),
            assessed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
