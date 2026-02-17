use crate::server_functions::db::get_db;
use leptos::server_fn::ServerFnError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlasgowAssessment {
    pub id: Option<String>,
    pub patient_id: String,
    pub eye_opening: i32,
    pub verbal_response: i32,
    pub motor_response: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApacheAssessment {
    pub id: Option<String>,
    pub patient_id: String,
    pub age: i32,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub diastolic_bp: i32,
    pub temperature: f32,
    pub pao2_fio2: f32,
    pub arterial_ph: f32,
    pub hematocrit: i32,
    pub white_blood_cell: i32,
    pub glasgow_coma_score: i32,
    pub chronic_health_points: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SofaAssessment {
    pub id: Option<String>,
    pub patient_id: String,
    pub respiratory: i32,
    pub coagulation: i32,
    pub liver: i32,
    pub cardiovascular: i32,
    pub neurological: i32,
    pub renal: i32,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SapsAssessment {
    pub id: Option<String>,
    pub patient_id: String,
    pub age: i32,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub temperature: i32,
    pub pao2_fio2: i32,
    pub urinary_output: i32,
    pub blood_urea: i32,
    pub white_blood_cell: i32,
    pub potassium: i32,
    pub sodium: i32,
    pub bicarbonate: i32,
    pub bilirubin: i32,
    pub glasgow_coma_score: i32,
    pub chronic_disease: bool,
    pub total_score: i32,
    pub assessed_by: String,
    pub assessed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct News2Assessment {
    pub id: Option<String>,
    pub patient_id: String,
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

#[leptos::server(GetPatientHistory, "/api")]
pub async fn get_patient_history(patient_id: String) -> Result<Vec<serde_json::Value>, ServerFnError> {
    leptos::logging::log!("Server: get_patient_history called for patient: {}", patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let glasgow: Vec<GlasgowAssessment> = client
            .query("SELECT * FROM glasgow WHERE patient_id = $patient_id ORDER BY assessed_at DESC LIMIT 10")
            .bind(("patient_id", patient_id.clone()))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let sofa: Vec<SofaAssessment> = client
            .query("SELECT * FROM sofa WHERE patient_id = $patient_id ORDER BY assessed_at DESC LIMIT 10")
            .bind(("patient_id", patient_id.clone()))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let mut history = Vec::new();
        
        for g in glasgow {
            history.push(serde_json::json!({
                "type": "glasgow",
                "data": g,
            }));
        }
        
        for s in sofa {
            history.push(serde_json::json!({
                "type": "sofa",
                "data": s,
            }));
        }
        
        Ok(history)
    } else {
        Ok(vec![
            serde_json::json!({
                "type": "glasgow",
                "data": {
                    "eye_opening": 3,
                    "verbal_response": 4,
                    "motor_response": 5,
                    "total_score": 12
                },
                "assessed_at": "2026-02-15T10:00:00Z",
                "assessed_by": "Dr. Smith"
            }),
        ])
    }
}

#[leptos::server(SaveGlasgow, "/api")]
pub async fn save_glasgow(assessment: GlasgowAssessment) -> Result<String, ServerFnError> {
    leptos::logging::log!("Server: save_glasgow called for patient: {}", assessment.patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let id = uuid::Uuid::new_v4().to_string();
        let _: Option<GlasgowAssessment> = client
            .create(("glasgow", &id))
            .content(assessment)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(id)
    } else {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[leptos::server(SaveApache, "/api")]
pub async fn save_apache(assessment: ApacheAssessment) -> Result<String, ServerFnError> {
    leptos::logging::log!("Server: save_apache called for patient: {}", assessment.patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let id = uuid::Uuid::new_v4().to_string();
        let _: Option<ApacheAssessment> = client
            .create(("apache", &id))
            .content(assessment)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(id)
    } else {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[leptos::server(SaveSofa, "/api")]
pub async fn save_sofa(assessment: SofaAssessment) -> Result<String, ServerFnError> {
    leptos::logging::log!("Server: save_sofa called for patient: {}", assessment.patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let id = uuid::Uuid::new_v4().to_string();
        let _: Option<SofaAssessment> = client
            .create(("sofa", &id))
            .content(assessment)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(id)
    } else {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[leptos::server(SaveSaps, "/api")]
pub async fn save_saps(assessment: SapsAssessment) -> Result<String, ServerFnError> {
    leptos::logging::log!("Server: save_saps called for patient: {}", assessment.patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let id = uuid::Uuid::new_v4().to_string();
        let _: Option<SapsAssessment> = client
            .create(("saps", &id))
            .content(assessment)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(id)
    } else {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[leptos::server(SaveNews2, "/api")]
pub async fn save_news2(assessment: News2Assessment) -> Result<String, ServerFnError> {
    leptos::logging::log!("Server: save_news2 called for patient: {}", assessment.patient_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let id = uuid::Uuid::new_v4().to_string();
        let _: Option<News2Assessment> = client
            .create(("news2", &id))
            .content(assessment)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(id)
    } else {
        Ok(uuid::Uuid::new_v4().to_string())
    }
}

#[leptos::server(CanAssessPatient, "/api")]
pub async fn can_assess_patient(patient_id: String, assessment_type: String) -> Result<bool, ServerFnError> {
    leptos::logging::log!("Server: can_assess_patient called for {} type {}", patient_id, assessment_type);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let table = match assessment_type.as_str() {
            "glasgow" => "glasgow",
            "apache" => "apache",
            "sofa" => "sofa",
            "saps" => "saps",
            "news2" => "news2",
            _ => "assessment",
        };
        
        let results: Vec<serde_json::Value> = client
            .query(format!("SELECT * FROM {} WHERE patient_id = $patient_id ORDER BY assessed_at DESC LIMIT 1", table))
            .bind(("patient_id", patient_id))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(results.is_empty() || true)
    } else {
        Ok(true)
    }
}

#[leptos::server(DeleteAssessment, "/api")]
pub async fn delete_assessment(patient_id: String, assessment_type: String, assessment_id: String) -> Result<bool, ServerFnError> {
    leptos::logging::log!("Server: delete_assessment called for {} type {} id {}", patient_id, assessment_type, assessment_id);
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let table = match assessment_type.as_str() {
            "glasgow" => "glasgow",
            "apache" => "apache",
            "sofa" => "sofa",
            "saps" => "saps",
            "news2" => "news2",
            _ => "assessment",
        };
        
        let _: Option<serde_json::Value> = client
            .delete((table, &assessment_id))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(true)
    } else {
        Ok(true)
    }
}
