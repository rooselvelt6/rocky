#[cfg(not(feature = "ssr"))]
use crate::models::Thing;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkinColor {
    White,
    Mixed,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdmissionType {
    Urgent,
    Programmed,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "ssr")]
    pub id: Option<RecordId>,
    #[cfg(not(feature = "ssr"))]
    pub id: Option<Thing>,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String, // ISO 8601 date YYYY-MM-DD
    pub gender: String,
    pub hospital_admission_date: String, // ISO 8601 datetime
    pub uci_admission_date: String,      // ISO 8601 datetime
    pub skin_color: SkinColor,
    pub principal_diagnosis: String,
    pub mechanical_ventilation: bool,
    pub uci_history: bool,
    pub transfer_from_other_center: bool,
    pub admission_type: AdmissionType,
    pub invasive_processes: bool,
    pub created_at: String,
    pub integrity_hash: String, // HADES: El Hilo Rojo de integridad
}

impl Patient {
    pub fn new(
        first_name: String,
        last_name: String,
        date_of_birth: String,
        gender: String,
        hospital_admission_date: String,
        uci_admission_date: String,
        skin_color: SkinColor,
        principal_diagnosis: String,
        mechanical_ventilation: bool,
        uci_history: bool,
        transfer_from_other_center: bool,
        admission_type: AdmissionType,
        invasive_processes: bool,
    ) -> Self {
        Self {
            id: None,
            first_name,
            last_name,
            date_of_birth,
            gender,
            hospital_admission_date,
            uci_admission_date,
            skin_color,
            principal_diagnosis,
            mechanical_ventilation,
            uci_history,
            transfer_from_other_center,
            admission_type,
            invasive_processes,
            created_at: chrono::Utc::now().to_rfc3339(),
            integrity_hash: String::new(), // Se calculará antes de guardar
        }
    }

    pub fn days_in_hospital(&self) -> i64 {
        let hospital_admission = chrono::DateTime::parse_from_rfc3339(
            &self.hospital_admission_date,
        )
        .unwrap_or_else(|_| chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap());
        let uci_admission = chrono::DateTime::parse_from_rfc3339(&self.uci_admission_date)
            .unwrap_or_else(|_| {
                chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap()
            });

        let duration = uci_admission.signed_duration_since(hospital_admission);
        duration.num_days().max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_in_hospital_calculation() {
        let patient = Patient::new(
            "Test".to_string(),
            "User".to_string(),
            "1990-01-01".to_string(),
            "Male".to_string(),
            "2023-10-01T10:00:00Z".to_string(),
            "2023-10-05T10:00:00Z".to_string(),
            SkinColor::White,
            "Diagnosis".to_string(),
            false,
            false,
            false,
            AdmissionType::Urgent,
            false,
        );
        patient.integrity_hash = "fake_hash".to_string(); // Mock for test

        assert_eq!(patient.days_in_hospital(), 4);
    }
}
