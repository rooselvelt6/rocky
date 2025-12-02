use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::sql::Thing;

#[cfg(not(feature = "ssr"))]
type Thing = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlasgowAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub patient: Option<Thing>, // Reference to patient (optional for now)
    pub eye_response: u8,
    pub verbal_response: u8,
    pub motor_response: u8,
    pub score: u8,
    pub diagnosis: String,
    pub recommendation: String,
    pub assessed_by: String,
    pub assessed_at: String, // ISO8601 timestamp
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
            patient: None, // Can be set later
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
