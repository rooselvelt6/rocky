use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::sql::Thing;

#[cfg(not(feature = "ssr"))]
type Thing = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String, // ISO 8601 date YYYY-MM-DD
    pub gender: String,
    pub admission_date: String, // ISO 8601 datetime
    pub admission_diagnosis: String,
    pub created_at: String,
}

impl Patient {
    pub fn new(
        first_name: String,
        last_name: String,
        date_of_birth: String,
        gender: String,
        admission_diagnosis: String,
    ) -> Self {
        Self {
            id: None,
            first_name,
            last_name,
            date_of_birth,
            gender,
            admission_date: chrono::Utc::now().to_rfc3339(),
            admission_diagnosis,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
