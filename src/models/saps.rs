use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::sql::Thing;

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
pub struct SapsAssessment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<Thing>, // Link to Patient record

    // SAPS II parameters
    pub age: u8,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub temperature: f32,
    pub pao2_fio2: Option<i32>,
    pub urinary_output: f32,
    pub serum_urea: f32,
    pub white_blood_count: f32,
    pub serum_potassium: f32,
    pub serum_sodium: i32,
    pub serum_bicarbonate: f32,
    pub bilirubin: f32,
    pub glasgow: u8,
    pub chronic_disease: String,
    pub admission_type: String,

    // Results
    pub score: u8,
    pub predicted_mortality: f32,
    pub severity: String,
    pub recommendation: String,

    // Metadata
    pub assessed_by: String,
    pub assessed_at: String, // ISO8601 timestamp
}

impl SapsAssessment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        age: u8,
        heart_rate: i32,
        systolic_bp: i32,
        temperature: f32,
        pao2_fio2: Option<i32>,
        urinary_output: f32,
        serum_urea: f32,
        white_blood_count: f32,
        serum_potassium: f32,
        serum_sodium: i32,
        serum_bicarbonate: f32,
        bilirubin: f32,
        glasgow: u8,
        chronic_disease: String,
        admission_type: String,
        score: u8,
        predicted_mortality: f32,
        severity: String,
        recommendation: String,
    ) -> Self {
        Self {
            id: None,
            patient_id: None,
            age,
            heart_rate,
            systolic_bp,
            temperature,
            pao2_fio2,
            urinary_output,
            serum_urea,
            white_blood_count,
            serum_potassium,
            serum_sodium,
            serum_bicarbonate,
            bilirubin,
            glasgow,
            chronic_disease,
            admission_type,
            score,
            predicted_mortality,
            severity,
            recommendation,
            assessed_by: "System".to_string(),
            assessed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
