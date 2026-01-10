#[cfg(not(feature = "ssr"))]
use crate::models::Thing;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "ssr")]
    pub id: Option<RecordId>,
    #[cfg(not(feature = "ssr"))]
    pub id: Option<Thing>,
    pub hospital_name: String,
    pub primary_color: String,   // HEX e.g. #4F46E5
    pub secondary_color: String, // HEX
    pub accent_color: String,    // HEX
    pub logo_url: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            id: None,
            hospital_name: "Unidad de Cuidados Intensivos".to_string(),
            primary_color: "#4F46E5".to_string(),   // Indigo 600
            secondary_color: "#1E1B4B".to_string(), // Indigo 950
            accent_color: "#FACC15".to_string(),    // Yellow 400
            logo_url: None,
            updated_at: chrono::Utc::now(),
        }
    }
}
