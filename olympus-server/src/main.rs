use axum::{
    routing::get,
    Router,
    Json,
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

use olympus_core::{Patient, User, SystemConfig};
use crate::system::Genesis;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("🏔️  OLYMPUS SYSTEM v15 - STARTING UP  🏔️");
    info!("⚡  Server Mode with 20 Gods (Actors)");
    info!("🏛️  Backend: Tokio + Axum + SurrealDB + Valkey");

    match Genesis::ignite().await {
        Ok(_) => info!("✨ Genesis completado. Los 20 dioses caminan entre nosotros."),
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
    "Olympus v15 is RUNNING. The Gods are awake."
}

async fn system_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "active",
        "version": "v15.0.0",
        "mode": "server",
        "gods": 20,
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
