use crate::models::patient::{Patient, SkinColor, AdmissionType, CivilStatus, Gender, YesNo, FamilyMember};
use crate::server_functions::db::{get_db, DbConfig};
use leptos::server_fn::ServerFnError;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize)]
struct DbPatient {
    first_name: String,
    last_name: String,
    identity_card: String,
    nationality: String,
    civil_status: String,
    gender: String,
    date_of_birth: String,
    address: String,
    clinical_history_number: String,
    hospital_admission_date: String,
    uci_admission_date: String,
    hospital_stay_days: Option<i64>,
    skin_color: String,
    diagnosis: String,
    diagnosis_huapa: Option<String>,
    diagnosis_uci: Option<String>,
    admission_summary: Option<String>,
    medical_history: Option<String>,
    transfer_migration: String,
    uci_history: String,
    mechanical_ventilation: String,
    admission_type: String,
    invasive_processes: Option<String>,
    physical_exam_hospital: Option<String>,
    physical_exam_uci: Option<String>,
    family_members: Vec<FamilyMember>,
    created_at: String,
    updated_at: Option<String>,
    integrity_hash: String,
}

impl From<DbPatient> for Patient {
    fn from(p: DbPatient) -> Self {
        Patient {
            id: None,
            first_name: p.first_name,
            last_name: p.last_name,
            identity_card: p.identity_card,
            nationality: p.nationality,
            civil_status: match p.civil_status.as_str() {
                "Single" => CivilStatus::Single,
                "Married" => CivilStatus::Married,
                "Divorced" => CivilStatus::Divorced,
                "Widowed" => CivilStatus::Widowed,
                "Cohabiting" => CivilStatus::Cohabiting,
                _ => CivilStatus::Single,
            },
            gender: match p.gender.as_str() {
                "Male" => Gender::Male,
                "Female" => Gender::Female,
                "Other" => Gender::Other,
                _ => Gender::Male,
            },
            date_of_birth: p.date_of_birth,
            address: p.address,
            clinical_history_number: p.clinical_history_number,
            hospital_admission_date: p.hospital_admission_date,
            uci_admission_date: p.uci_admission_date,
            hospital_stay_days: p.hospital_stay_days,
            skin_color: match p.skin_color.as_str() {
                "VeryFair" => SkinColor::VeryFair,
                "Fair" => SkinColor::Fair,
                "Olive" => SkinColor::Olive,
                "Brown" => SkinColor::Brown,
                "DarkBrown" => SkinColor::DarkBrown,
                "Black" => SkinColor::Black,
                _ => SkinColor::Fair,
            },
            diagnosis: p.diagnosis,
            diagnosis_huapa: p.diagnosis_huapa,
            diagnosis_uci: p.diagnosis_uci,
            admission_summary: p.admission_summary,
            medical_history: p.medical_history,
            transfer_migration: match p.transfer_migration.as_str() {
                "Yes" => YesNo::Yes,
                _ => YesNo::No,
            },
            uci_history: match p.uci_history.as_str() {
                "Yes" => YesNo::Yes,
                _ => YesNo::No,
            },
            mechanical_ventilation: match p.mechanical_ventilation.as_str() {
                "Yes" => YesNo::Yes,
                _ => YesNo::No,
            },
            admission_type: match p.admission_type.as_str() {
                "Elective" => AdmissionType::Elective,
                "Urgent" => AdmissionType::Urgent,
                "Transfer" => AdmissionType::Transfer,
                _ => AdmissionType::Urgent,
            },
            invasive_processes: p.invasive_processes,
            physical_exam_hospital: p.physical_exam_hospital,
            physical_exam_uci: p.physical_exam_uci,
            family_members: p.family_members,
            created_at: p.created_at,
            updated_at: p.updated_at,
            integrity_hash: p.integrity_hash,
        }
    }
}

impl From<Patient> for DbPatient {
    fn from(p: Patient) -> Self {
        DbPatient {
            first_name: p.first_name,
            last_name: p.last_name,
            identity_card: p.identity_card,
            nationality: p.nationality,
            civil_status: format!("{:?}", p.civil_status),
            gender: format!("{:?}", p.gender),
            date_of_birth: p.date_of_birth,
            address: p.address,
            clinical_history_number: p.clinical_history_number,
            hospital_admission_date: p.hospital_admission_date,
            uci_admission_date: p.uci_admission_date,
            hospital_stay_days: p.hospital_stay_days,
            skin_color: format!("{:?}", p.skin_color),
            diagnosis: p.diagnosis,
            diagnosis_huapa: p.diagnosis_huapa,
            diagnosis_uci: p.diagnosis_uci,
            admission_summary: p.admission_summary,
            medical_history: p.medical_history,
            transfer_migration: format!("{:?}", p.transfer_migration),
            uci_history: format!("{:?}", p.uci_history),
            mechanical_ventilation: format!("{:?}", p.mechanical_ventilation),
            admission_type: format!("{:?}", p.admission_type),
            invasive_processes: p.invasive_processes,
            physical_exam_hospital: p.physical_exam_hospital,
            physical_exam_uci: p.physical_exam_uci,
            family_members: p.family_members,
            created_at: p.created_at,
            updated_at: p.updated_at,
            integrity_hash: p.integrity_hash,
        }
    }
}

fn get_demo_patients() -> Vec<Patient> {
    vec![
        Patient {
            id: None,
            first_name: "Juan".to_string(),
            last_name: "Pérez".to_string(),
            identity_card: "V12345678".to_string(),
            nationality: "Venezolano".to_string(),
            civil_status: CivilStatus::Married,
            gender: Gender::Male,
            date_of_birth: "1960-05-15".to_string(),
            address: "Caracas, Venezuela".to_string(),
            clinical_history_number: "HC-001".to_string(),
            hospital_admission_date: "2026-01-10T10:00:00Z".to_string(),
            uci_admission_date: "2026-01-10T14:00:00Z".to_string(),
            hospital_stay_days: Some(4),
            skin_color: SkinColor::Fair,
            diagnosis: "Neumonía severa".to_string(),
            diagnosis_huapa: Some("Neumonía adquirida en comunidad".to_string()),
            diagnosis_uci: Some("Insuficiencia respiratoria aguda".to_string()),
            admission_summary: Some("Paciente ingresa por dificultad respiratoria".to_string()),
            medical_history: Some("Hipertensión arterial, Diabetes tipo 2".to_string()),
            transfer_migration: YesNo::No,
            uci_history: YesNo::No,
            mechanical_ventilation: YesNo::Yes,
            admission_type: AdmissionType::Urgent,
            invasive_processes: Some("Intubación orotraqueal".to_string()),
            physical_exam_hospital: Some("Taquipnea, febril".to_string()),
            physical_exam_uci: Some("Sedado, ventilado mecánicamente".to_string()),
            family_members: vec![
                FamilyMember {
                    name: "María Pérez".to_string(),
                    relationship: "Esposa".to_string(),
                    phone: "+584121234567".to_string(),
                    email: Some("maria@email.com".to_string()),
                }
            ],
            created_at: "2026-01-10T10:00:00Z".to_string(),
            updated_at: None,
            integrity_hash: "abc123".to_string(),
        },
        Patient {
            id: None,
            first_name: "María".to_string(),
            last_name: "García".to_string(),
            identity_card: "V87654321".to_string(),
            nationality: "Venezolano".to_string(),
            civil_status: CivilStatus::Single,
            gender: Gender::Female,
            date_of_birth: "1975-08-22".to_string(),
            address: "Maracaibo, Venezuela".to_string(),
            clinical_history_number: "HC-002".to_string(),
            hospital_admission_date: "2026-02-01T08:00:00Z".to_string(),
            uci_admission_date: "2026-02-01T12:00:00Z".to_string(),
            hospital_stay_days: Some(2),
            skin_color: SkinColor::Olive,
            diagnosis: "Postquirúrgico - CABG".to_string(),
            diagnosis_huapa: Some("Enfermedad coronaria".to_string()),
            diagnosis_uci: Some("Postoperatorio de cirugía cardíaca".to_string()),
            admission_summary: Some("Intervención quirúrgica programada".to_string()),
            medical_history: Some("Cardiopatía isquémica".to_string()),
            transfer_migration: YesNo::No,
            uci_history: YesNo::Yes,
            mechanical_ventilation: YesNo::No,
            admission_type: AdmissionType::Elective,
            invasive_processes: Some("Cirugía de revascularización".to_string()),
            physical_exam_hospital: Some("Estable postquirúrgico".to_string()),
            physical_exam_uci: Some("En despertar, estable hemodinámicamente".to_string()),
            family_members: vec![
                FamilyMember {
                    name: "Carlos García".to_string(),
                    relationship: "Hermano".to_string(),
                    phone: "+584241234567".to_string(),
                    email: None,
                }
            ],
            created_at: "2026-02-01T08:00:00Z".to_string(),
            updated_at: None,
            integrity_hash: "def456".to_string(),
        },
    ]
}

#[leptos::server(GetPatients, "/api")]
pub async fn get_patients() -> Result<Vec<Patient>, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let results: Vec<DbPatient> = client
            .query("SELECT * FROM patient ORDER BY created_at DESC")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(results.into_iter().map(Patient::from).collect())
    } else {
        Ok(get_demo_patients())
    }
}

#[leptos::server(GetPatient, "/api")]
pub async fn get_patient(id: String) -> Result<Option<Patient>, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let results: Vec<DbPatient> = client
            .query("SELECT * FROM patient WHERE id = $id")
            .bind(("id", format!("patient:{}", id)))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(results.into_iter().next().map(Patient::from))
    } else {
        let patients = get_demo_patients();
        Ok(patients.into_iter().find(|_| true))
    }
}

#[leptos::server(CreatePatient, "/api")]
pub async fn create_patient(patient: Patient) -> Result<String, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    let mut db_patient: DbPatient = patient.into();
    
    db_patient.created_at = chrono::Utc::now().to_rfc3339();
    
    if let Some(ref client) = *guard {
        let result: Option<DbPatient> = client
            .create("patient")
            .content(db_patient)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(result.map(|p| p.first_name).unwrap_or_else(|| "created".to_string()))
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(id)
    }
}

#[leptos::server(UpdatePatient, "/api")]
pub async fn update_patient(id: String, patient: Patient) -> Result<bool, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    let mut db_patient: DbPatient = patient.into();
    db_patient.updated_at = Some(chrono::Utc::now().to_rfc3339());
    
    if let Some(ref client) = *guard {
        let _: Option<DbPatient> = client
            .update(("patient", &id))
            .content(db_patient)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(true)
    } else {
        Ok(true)
    }
}

#[leptos::server(DeletePatient, "/api")]
pub async fn delete_patient(id: String) -> Result<bool, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let _: Option<DbPatient> = client
            .delete(("patient", &id))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(true)
    } else {
        Ok(true)
    }
}

#[leptos::server(SearchPatients, "/api")]
pub async fn search_patients(query: String) -> Result<Vec<Patient>, ServerFnError> {
    let db = get_db().await;
    let guard = db.read().await;
    let query_lower = query.to_lowercase();
    
    if let Some(ref client) = *guard {
        let results: Vec<DbPatient> = client
            .query("SELECT * FROM patient WHERE string::lowercase(first_name) CONTAINS $q OR string::lowercase(last_name) CONTAINS $q OR string::lowercase(diagnosis) CONTAINS $q OR string::lowercase(identity_card) CONTAINS $q")
            .bind(("q", query_lower))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(results.into_iter().map(Patient::from).collect())
    } else {
        let all_patients = get_demo_patients();
        Ok(all_patients.into_iter().filter(|p| {
            p.first_name.to_lowercase().contains(&query_lower)
                || p.last_name.to_lowercase().contains(&query_lower)
                || p.diagnosis.to_lowercase().contains(&query_lower)
                || p.identity_card.to_lowercase().contains(&query_lower)
        }).collect())
    }
}
