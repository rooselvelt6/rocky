#[cfg(not(feature = "ssr"))]
use crate::models::Thing;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkinColor {
    VeryFair,
    Fair,
    Olive,
    Brown,
    DarkBrown,
    Black,
}

impl Default for SkinColor {
    fn default() -> Self {
        SkinColor::Fair
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdmissionType {
    Elective,
    Urgent,
    Transfer,
}

impl Default for AdmissionType {
    fn default() -> Self {
        AdmissionType::Urgent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CivilStatus {
    Single,
    Married,
    Divorced,
    Widowed,
    Cohabiting,
}

impl Default for CivilStatus {
    fn default() -> Self {
        CivilStatus::Single
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Male
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum YesNo {
    Yes,
    No,
}

impl Default for YesNo {
    fn default() -> Self {
        YesNo::No
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FamilyMember {
    pub name: String,
    pub relationship: String,
    pub phone: String,
    pub email: Option<String>,
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
    pub identity_card: String,
    pub nationality: String,
    pub civil_status: CivilStatus,
    pub gender: Gender,
    pub date_of_birth: String,
    pub address: String,

    pub clinical_history_number: String,
    pub hospital_admission_date: String,
    pub uci_admission_date: String,
    pub hospital_stay_days: Option<i64>,

    pub skin_color: SkinColor,

    pub diagnosis: String,
    pub diagnosis_huapa: Option<String>,
    pub diagnosis_uci: Option<String>,
    pub admission_summary: Option<String>,
    pub medical_history: Option<String>,

    pub transfer_migration: YesNo,
    pub uci_history: YesNo,
    pub mechanical_ventilation: YesNo,
    pub admission_type: AdmissionType,
    pub invasive_processes: Option<String>,

    pub physical_exam_hospital: Option<String>,
    pub physical_exam_uci: Option<String>,

    pub family_members: Vec<FamilyMember>,

    pub created_at: String,
    pub updated_at: Option<String>,
    pub integrity_hash: String,
}

impl Patient {
    pub fn new(
        first_name: String,
        last_name: String,
        identity_card: String,
        nationality: String,
        civil_status: CivilStatus,
        gender: Gender,
        date_of_birth: String,
        address: String,
        clinical_history_number: String,
        hospital_admission_date: String,
        uci_admission_date: String,
        skin_color: SkinColor,
        diagnosis: String,
        transfer_migration: YesNo,
        uci_history: YesNo,
        mechanical_ventilation: YesNo,
        admission_type: AdmissionType,
        invasive_processes: Option<String>,
    ) -> Self {
        let hospital_stay_days =
            Self::calculate_hospital_stay(&hospital_admission_date, &uci_admission_date);

        Self {
            id: None,
            first_name,
            last_name,
            identity_card,
            nationality,
            civil_status,
            gender,
            date_of_birth,
            address,
            clinical_history_number,
            hospital_admission_date,
            uci_admission_date,
            hospital_stay_days,
            skin_color,
            diagnosis,
            diagnosis_huapa: None,
            diagnosis_uci: None,
            admission_summary: None,
            medical_history: None,
            transfer_migration,
            uci_history,
            mechanical_ventilation,
            admission_type,
            invasive_processes,
            physical_exam_hospital: None,
            physical_exam_uci: None,
            family_members: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
            integrity_hash: String::new(),
        }
    }

    fn calculate_hospital_stay(hospital_date: &str, uci_date: &str) -> Option<i64> {
        let hospital = chrono::DateTime::parse_from_rfc3339(hospital_date)
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(hospital_date, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .ok()?;
        let uci = chrono::DateTime::parse_from_rfc3339(uci_date)
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(uci_date, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .ok()?;

        Some(uci.signed_duration_since(hospital).num_days().max(0))
    }

    pub fn calculate_stay_days(&mut self) {
        self.hospital_stay_days =
            Self::calculate_hospital_stay(&self.hospital_admission_date, &self.uci_admission_date);
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
