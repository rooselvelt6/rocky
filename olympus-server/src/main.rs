use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::Query,
};
use tower_http::services::ServeDir;
use tracing::{info, error};
use std::net::SocketAddr;

mod actors;
mod traits;
mod system;
mod errors;
mod infrastructure;
mod uci;
mod q1;

use olympus_core::{Patient, User, SystemConfig};
use crate::system::Genesis;
use crate::q1::{
    search::{SearchFilters, PatientSearch},
    reports::{ReportGenerator, ReportType},
    analytics::{AnalyticsDashboard, DashboardMetrics},
    export::{DataExporter, ExportFormat, ExportDataType, ExportRequest},
    ward_view::{WardViewManager, PatientCard, CardSeverity},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("🏔️  OLYMPUS SYSTEM v16 - STARTING UP  🏔️");
    info!("⚡  Server Mode with 21 Gods (Actors)");
    info!("🏛️  Backend: Tokio + Axum + SurrealDB + Valkey");

    match Genesis::ignite().await {
        Ok(_) => info!("✨ Genesis completado. Los 21 dioses caminan entre nosotros."),
        Err(e) => {
            error!("💀 Genesis falló: {}", e);
        }
    }


    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health_check))
        .route("/api/status", get(system_status))
        .route("/api/login", get(api_login))
        .route("/api/patients", get(api_patients))
        .route("/api/patients/:id", get(api_patient))
        .route("/api/search", post(api_search))
        .route("/api/report/pdf", post(api_generate_pdf))
        .route("/api/dashboard/metrics", get(api_dashboard_metrics))
        .route("/api/export", post(api_export))
        .route("/api/ward/view", get(api_ward_view))
        .route("/api/ward/patient/:id", get(api_ward_patient))
        .nest_service("/static", ServeDir::new("../olympus-client/dist"))
        .fallback_service(ServeDir::new("../olympus-client/dist"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🌍 API Gateway escuchando en http://{}", addr);
    info!("🌐 Frontend disponible en http://{}/", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index() -> axum::response::Redirect {
    axum::response::Redirect::to("/static/index.html")
}

async fn health_check() -> &'static str {
    "Olympus v16 is RUNNING. The Gods are awake."
}

async fn system_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "active",
        "version": "v16.0.0",
        "mode": "server",
        "gods": 21,
        "trinity": ["Zeus", "Hades", "Poseidon"],
        "message": "System booted via Genesis"
    }))
}

async fn api_login() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Login endpoint ready"
    }))
}

async fn api_patients() -> Json<serde_json::Value> {
    let patients = vec![
        Patient {
            id: None,
            first_name: "Juan".to_string(),
            last_name: "Pérez".to_string(),
            date_of_birth: "1960-05-15".to_string(),
            gender: "Male".to_string(),
            hospital_admission_date: "2026-01-10T10:00:00Z".to_string(),
            uci_admission_date: "2026-01-10T14:00:00Z".to_string(),
            skin_color: olympus_core::patient::SkinColor::White,
            principal_diagnosis: "Neumonía severa".to_string(),
            mechanical_ventilation: true,
            uci_history: false,
            transfer_from_other_center: false,
            admission_type: olympus_core::patient::AdmissionType::Urgent,
            invasive_processes: true,
            created_at: "2026-01-10T10:00:00Z".to_string(),
            integrity_hash: "abc123".to_string(),
        },
        Patient {
            id: None,
            first_name: "María".to_string(),
            last_name: "García".to_string(),
            date_of_birth: "1975-08-22".to_string(),
            gender: "Female".to_string(),
            hospital_admission_date: "2026-02-01T08:00:00Z".to_string(),
            uci_admission_date: "2026-02-01T12:00:00Z".to_string(),
            skin_color: olympus_core::patient::SkinColor::Mixed,
            principal_diagnosis: "Postquirúrgico - CABG".to_string(),
            mechanical_ventilation: false,
            uci_history: true,
            transfer_from_other_center: false,
            admission_type: olympus_core::patient::AdmissionType::Programmed,
            invasive_processes: true,
            created_at: "2026-02-01T08:00:00Z".to_string(),
            integrity_hash: "def456".to_string(),
        },
    ];
    
    Json(serde_json::json!({
        "success": true,
        "data": patients
    }))
}

async fn api_patient(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id,
            "first_name": "Juan",
            "last_name": "Pérez"
        }
    }))
}

async fn api_search(
    Json(filters): Json<SearchFilters>,
) -> Json<serde_json::Value> {
    let search = PatientSearch::new();
    let results = search.search(filters);
    Json(serde_json::json!({
        "success": true,
        "data": results.results,
        "total": results.total_count,
        "query_time_ms": results.query_time_ms
    }))
}

async fn api_generate_pdf(
    Json(request): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let report_type = request.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("summary");
    
    let generator = ReportGenerator::default_config();
    let report_id = uuid::Uuid::new_v4().to_string();
    
    Json(serde_json::json!({
        "success": true,
        "report_id": report_id,
        "type": report_type,
        "message": "PDF report generation initiated"
    }))
}

async fn api_dashboard_metrics() -> Json<serde_json::Value> {
    let dashboard = AnalyticsDashboard::new();
    let metrics = DashboardMetrics::default();
    
    Json(serde_json::json!({
        "success": true,
        "data": metrics,
        "kpis": dashboard.get_kpis("uci")
    }))
}

async fn api_export(
    Json(request): Json<ExportRequest>,
) -> Json<serde_json::Value> {
    let exporter = DataExporter::new();
    
    Json(serde_json::json!({
        "success": true,
        "message": "Export initiated",
        "format": format!("{:?}", request.format),
        "data_type": format!("{:?}", request.data_type)
    }))
}

async fn api_ward_view() -> Json<serde_json::Value> {
    let ward = WardViewManager::new();
    let metrics = ward.get_metrics();
    let patients: Vec<_> = ward.get_all_patients().iter().map(|p| {
        serde_json::json!({
            "patient_id": p.patient_id,
            "bed_number": p.bed_number,
            "severity": format!("{:?}", p.severity),
            "severity_color": p.severity.color(),
            "alert_status": format!("{:?}", p.alert_status),
            "should_blink": p.alert_status.should_blink(),
            "sofa_score": p.sofa_score,
            "news2_score": p.news2_score
        })
    }).collect();
    
    Json(serde_json::json!({
        "success": true,
        "metrics": metrics,
        "patients": patients
    }))
}

async fn api_ward_patient(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let ward = WardViewManager::new();
    
    if let Some(patient) = ward.get_patient(&id) {
        Json(serde_json::json!({
            "success": true,
            "data": {
                "patient_id": patient.patient_id,
                "first_name": patient.first_name,
                "last_name": patient.last_name,
                "severity": format!("{:?}", patient.severity),
                "severity_color": patient.severity.color(),
                "alert_status": format!("{:?}", patient.alert_status),
                "is_critical": patient.is_critical,
                "sofa_score": patient.sofa_score,
                "saps_score": patient.saps_score,
                "news2_score": patient.news2_score,
                "mechanical_ventilation": patient.mechanical_ventilation,
                "protocols_active": patient.protocols_active,
                "last_update": patient.last_update
            }
        }))
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Patient not found"
        }))
    }
}
