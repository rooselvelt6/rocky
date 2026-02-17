use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub hospital_name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub logo_url: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            id: None,
            hospital_name: "Unidad de Cuidados Intensivos".to_string(),
            primary_color: "#4F46E5".to_string(),
            secondary_color: "#1E1B4B".to_string(),
            accent_color: "#FACC15".to_string(),
            logo_url: None,
            updated_at: chrono::Utc::now(),
        }
    }
}
