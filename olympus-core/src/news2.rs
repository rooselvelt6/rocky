use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct News2Assessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>,
    pub respiratory_rate: i32,
    pub oxygen_saturation: i32,
    pub supplemental_oxygen: bool,
    pub temperature: i32,
    pub systolic_bp: i32,
    pub heart_rate: i32,
    pub consciousness: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}
