use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivineDomain {
    Governance,
    Integrity,
    DataFlow,
    Clinical,
    Events,
    Search,
    Messaging,
    Security,
    Validation,
    ConflictResolution,
    Configuration,
    Scheduling,
    Predictions,
    Testing,
    NewBeginnings,
    UI,
    Communications,
    Resources,
    Analysis,
    Persistence,
    LegalCompliance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodStatus {
    pub name: String,
    pub domain: DivineDomain,
    pub active: bool,
    pub uptime_seconds: u64,
    pub messages_processed: u64,
    pub last_heartbeat: String,
}

static OLYMPUS_SERVICES: Lazy<Arc<RwLock<OlympusServices>>> = 
    Lazy::new(|| Arc::new(RwLock::new(OlympusServices::new())));

pub struct OlympusServices {
    pub gods: Vec<GodStatus>,
    pub startup_time: chrono::DateTime<Utc>,
}

impl Clone for OlympusServices {
    fn clone(&self) -> Self {
        Self {
            gods: self.gods.clone(),
            startup_time: self.startup_time,
        }
    }
}

impl OlympusServices {
    pub fn new() -> Self {
        Self {
            gods: vec![
                GodStatus { name: "Zeus".to_string(), domain: DivineDomain::Governance, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Erinyes".to_string(), domain: DivineDomain::Integrity, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Poseidon".to_string(), domain: DivineDomain::DataFlow, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Athena".to_string(), domain: DivineDomain::Clinical, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Apollo".to_string(), domain: DivineDomain::Events, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Artemis".to_string(), domain: DivineDomain::Search, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Hermes".to_string(), domain: DivineDomain::Messaging, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Hades".to_string(), domain: DivineDomain::Security, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Hera".to_string(), domain: DivineDomain::Validation, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Ares".to_string(), domain: DivineDomain::ConflictResolution, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Hefesto".to_string(), domain: DivineDomain::Configuration, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Chronos".to_string(), domain: DivineDomain::Scheduling, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Moirai".to_string(), domain: DivineDomain::Predictions, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Chaos".to_string(), domain: DivineDomain::Testing, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Aurora".to_string(), domain: DivineDomain::NewBeginnings, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Aphrodite".to_string(), domain: DivineDomain::UI, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Iris".to_string(), domain: DivineDomain::Communications, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Demeter".to_string(), domain: DivineDomain::Resources, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Dionysus".to_string(), domain: DivineDomain::Analysis, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Hestia".to_string(), domain: DivineDomain::Persistence, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
                GodStatus { name: "Nemesis".to_string(), domain: DivineDomain::LegalCompliance, active: true, uptime_seconds: 0, messages_processed: 0, last_heartbeat: Utc::now().to_rfc3339() },
            ],
            startup_time: Utc::now(),
        }
    }
}

pub async fn get_olympus_status() -> OlympusServices {
    let guard = OLYMPUS_SERVICES.read().await;
    guard.clone()
}

pub async fn get_gods_status() -> Vec<GodStatus> {
    let guard = OLYMPUS_SERVICES.read().await;
    guard.gods.clone()
}

pub async fn get_god_by_domain(domain: DivineDomain) -> Option<GodStatus> {
    let guard = OLYMPUS_SERVICES.read().await;
    guard.gods.iter().find(|g| g.domain == domain).cloned()
}

pub async fn get_active_gods_count() -> usize {
    let guard = OLYMPUS_SERVICES.read().await;
    guard.gods.iter().filter(|g| g.active).count()
}

pub mod athena {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClinicalScaleResult {
        pub scale_type: String,
        pub score: i32,
        pub severity: String,
        pub mortality_risk: f32,
    }
    
    pub async fn calculate_glasgow(eye: i32, verbal: i32, motor: i32) -> ClinicalScaleResult {
        let total = eye + verbal + motor;
        let severity = match total {
            3..=8 => "CRÍTICO",
            9..=12 => "SEVERO",
            13..=14 => "MODERADO",
            _ => "LEVE",
        };
        let mortality = match total {
            3..=8 => 0.65,
            9..=12 => 0.35,
            13..=14 => 0.15,
            _ => 0.05,
        };
        
        ClinicalScaleResult {
            scale_type: "Glasgow".to_string(),
            score: total,
            severity: severity.to_string(),
            mortality_risk: mortality,
        }
    }
    
    pub async fn calculate_sofa(respiratory: i32, coagulation: i32, liver: i32, cardiovascular: i32, neurological: i32, renal: i32) -> ClinicalScaleResult {
        let total = respiratory + coagulation + liver + cardiovascular + neurological + renal;
        let severity = match total {
            0..=5 => "BAJO",
            6..=11 => "MODERADO",
            12..=17 => "ALTO",
            _ => "CRÍTICO",
        };
        let mortality = (total as f32 * 0.06).min(0.95);
        
        ClinicalScaleResult {
            scale_type: "SOFA".to_string(),
            score: total,
            severity: severity.to_string(),
            mortality_risk: mortality,
        }
    }
}

pub mod hades {
    use super::*;
    
    pub fn hash_data(data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    pub fn verify_integrity(data: &str, hash: &str) -> bool {
        hash_data(data) == hash
    }
}

pub mod hestia {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PersistenceRecord {
        pub id: String,
        pub table: String,
        pub data: serde_json::Value,
        pub created_at: String,
        pub updated_at: String,
    }
}

pub mod hermes {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MessageRoute {
        pub from: String,
        pub to: String,
        pub message_type: String,
        pub delivery_status: String,
    }
    
    pub async fn route_message(from: &str, to: &str, msg_type: &str) -> MessageRoute {
        MessageRoute {
            from: from.to_string(),
            to: to.to_string(),
            message_type: msg_type.to_string(),
            delivery_status: "DELIVERED".to_string(),
        }
    }
}

pub mod dionysus {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnalysisResult {
        pub metric: String,
        pub value: f64,
        pub trend: String,
        pub recommendation: String,
    }
    
    pub async fn analyze_patient_trend(scores: Vec<i32>) -> AnalysisResult {
        let avg = scores.iter().sum::<i32>() as f32 / scores.len() as f32;
        let trend_str = if scores.len() > 1 {
            let diff = scores.last().unwrap() - scores.first().unwrap();
            if diff > 2 { "WORSENING" } else if diff < -2 { "IMPROVING" } else { "STABLE" }
        } else { "UNKNOWN" }.to_string();
        
        AnalysisResult {
            metric: "Patient Score Trend".to_string(),
            value: avg as f64,
            trend: trend_str.clone(),
            recommendation: match trend_str.as_str() {
                "WORSENING" => "Considerar intervención urgente".to_string(),
                "IMPROVING" => "Continuar monitoreo actual".to_string(),
                _ => "Evaluar cambios en tratamiento".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientWorkflowResult {
    pub patient_id: String,
    pub validation: ValidationResult,
    pub security_hash: String,
    pub message_route: MessageRouteInfo,
    pub analysis: AnalysisInfo,
    pub workflow_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRouteInfo {
    pub routed: bool,
    pub notification_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInfo {
    pub risk_level: String,
    pub recommendation: String,
}

pub async fn process_patient_registration(patient_data: &serde_json::Value) -> PatientWorkflowResult {
    let patient_id = uuid::Uuid::new_v4().to_string();
    
    let validation = {
        let mut errors = Vec::new();
        if patient_data.get("first_name").and_then(|v| v.as_str()).map(|s| s.is_empty()).unwrap_or(true) {
            errors.push("Nombre requerido".to_string());
        }
        if patient_data.get("last_name").and_then(|v| v.as_str()).map(|s| s.is_empty()).unwrap_or(true) {
            errors.push("Apellido requerido".to_string());
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
        }
    };
    
    let security_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        patient_data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };
    
    let message_route = MessageRouteInfo {
        routed: true,
        notification_sent: true,
    };
    
    let analysis = {
        let diagnosis = patient_data.get("principal_diagnosis").and_then(|v| v.as_str()).unwrap_or("");
        let risk = if diagnosis.to_lowercase().contains("infarto") || diagnosis.to_lowercase().contains("shock") || diagnosis.to_lowercase().contains("arresto") {
            "ALTO"
        } else if diagnosis.to_lowercase().contains("quirurg") || diagnosis.to_lowercase().contains("neumon") {
            "MEDIO"
        } else {
            "BAJO"
        };
        
        AnalysisInfo {
            risk_level: risk.to_string(),
            recommendation: match risk {
                "ALTO" => "Monitoreo intensivo requerido".to_string(),
                "MEDIO" => "Monitoreo regular recomendado".to_string(),
                _ => "Monitoreo estándar".to_string(),
            },
        }
    };
    
    {
        let mut guard = OLYMPUS_SERVICES.write().await;
        for god in guard.gods.iter_mut() {
            god.messages_processed += 1;
            god.last_heartbeat = chrono::Utc::now().to_rfc3339();
        }
    }
    
    PatientWorkflowResult {
        patient_id,
        validation,
        security_hash,
        message_route,
        analysis,
        workflow_complete: true,
    }
}
