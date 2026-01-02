use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Id {
    #[serde(rename = "String")]
    pub string: String,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Thing {
    pub tb: String,
    pub id: Id,
}

#[cfg(not(feature = "ssr"))]
impl ToString for Thing {
    fn to_string(&self) -> String {
        format!("{}:{}", self.tb, self.id.string)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofaAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "ssr")]
    pub id: Option<RecordId>,
    #[cfg(not(feature = "ssr"))]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "ssr")]
    pub patient_id: Option<RecordId>, // Link to Patient record
    #[cfg(not(feature = "ssr"))]
    pub patient_id: Option<Thing>,

    // SOFA parameters
    pub pao2_fio2: i32,
    pub platelets: i32,
    pub bilirubin: f32,
    pub cardiovascular: String,
    pub glasgow: u8,
    pub renal: String,

    // Results
    pub score: u8,
    pub severity: String,
    pub recommendation: String,

    // Metadata
    pub assessed_by: String,
    pub assessed_at: String, // ISO8601 timestamp
}

impl SofaAssessment {
    pub fn new(
        pao2_fio2: i32,
        platelets: i32,
        bilirubin: f32,
        cardiovascular: String,
        glasgow: u8,
        renal: String,
        score: u8,
        severity: String,
        recommendation: String,
    ) -> Self {
        Self {
            id: None,
            patient_id: None,
            pao2_fio2,
            platelets,
            bilirubin,
            cardiovascular,
            glasgow,
            renal,
            score,
            severity,
            recommendation,
            assessed_by: "System".to_string(),
            assessed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
