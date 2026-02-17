use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentHistory {
    pub assessment_type: ScaleType,
    pub data: serde_json::Value,
    pub score: i32,
    pub severity: String,
    pub assessed_by: String,
    pub assessed_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScaleType {
    Glasgow,
    Sofa,
    Apache,
    Saps,
    News2,
}

impl ScaleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScaleType::Glasgow => "Glasgow",
            ScaleType::Sofa => "SOFA",
            ScaleType::Apache => "APACHE II",
            ScaleType::Saps => "SAPS II",
            ScaleType::News2 => "NEWS2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientHistory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub patient_id: String,
    pub clinical_history_number: String,
    pub patient_name: String,
    pub assessments: Vec<AssessmentHistory>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl PatientHistory {
    pub fn new(patient_id: String, clinical_history_number: String, patient_name: String) -> Self {
        Self {
            id: None,
            patient_id,
            clinical_history_number,
            patient_name,
            assessments: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
        }
    }

    pub fn add_assessment(&mut self, assessment: AssessmentHistory) {
        self.assessments.push(assessment);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    pub fn latest_assessment(&self, scale_type: &ScaleType) -> Option<&AssessmentHistory> {
        self.assessments
            .iter()
            .rev()
            .find(|a| &a.assessment_type == scale_type)
    }
}
