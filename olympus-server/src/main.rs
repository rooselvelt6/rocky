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

use crate::system::Genesis;

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
        serde_json::json!({
            "id": "1",
            "first_name": "Juan",
            "last_name": "Pérez",
            "diagnosis": "Neumonía severa",
            "severity": "High"
        }),
        serde_json::json!({
            "id": "2",
            "first_name": "María",
            "last_name": "García",
            "diagnosis": "Postquirúrgico - CABG",
            "severity": "Medium"
        }),
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
