pub mod olympus_services;

use axum::{
    routing::get,
    Router,
    Json,
};
use tower_http::services::ServeDir;
use tracing::{info, error};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

static DB: Lazy<Arc<RwLock<Option<Surreal<Any>>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8000".to_string(),
            namespace: "hospital".to_string(),
            database: "uci".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

async fn init_db(config: DbConfig) -> Result<(), String> {
    let db = surrealdb::engine::any::connect(&config.url)
        .await
        .map_err(|e| format!("SurrealDB connection failed: {}", e))?;
    
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .map_err(|e| format!("SurrealDB namespace/DB failed: {}", e))?;

    let mut guard = DB.write().await;
    *guard = Some(db);
    
    println!("✅ SurrealDB connected: {}/{}", config.namespace, config.database);
    Ok(())
}

async fn get_db() -> Arc<RwLock<Option<Surreal<Any>>>> {
    DB.clone()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("🏔️  OLYMPUS SYSTEM v15 - STARTING UP  🏔️");
    info!("⚡  With 20 Divine Gods - Leptos 0.8");

    let db_config = DbConfig::default();
    
    match init_db(db_config).await {
        Ok(_) => info!("✅ Base de datos conectada"),
        Err(e) => error!("⚠️  SurrealDB no disponible: {}", e),
    }

    // Inicializar servicios de los dioses
    let gods_status = olympus_services::get_gods_status().await;
    info!("⚡ {} dioses activos inicializados", gods_status.len());

    let app = Router::new()
        .route("/", get(index))
        .route("/api/status", get(api_status))
        .route("/api/olympus/gods", get(api_olympus_gods))
        .route("/api/olympus/god/:domain", get(api_olympus_god))
        .route("/api/scales/glasgow", get(api_glasgow))
        .route("/api/scales/sofa", get(api_sofa))
        .route("/api/patients", get(api_patients))
        .route("/api/patient/:id", get(api_patient))
        .route("/api/login", get(api_login))
        .route("/api/logout", get(api_logout))
        .route("/api/admin/stats", get(api_stats))
        .nest_service("/static", ServeDir::new("dist"))
        .fallback_service(ServeDir::new("dist"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🌍 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index() -> axum::response::Html<&'static str> {
    axum::response::Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>UCI - Olympus v15</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div id="app" class="min-h-screen bg-gradient-to-br from-indigo-900 to-purple-900 text-white">
        <div class="container mx-auto p-8">
            <h1 class="text-4xl font-bold mb-4">🏔️ OLYMPUS v15</h1>
            <p class="text-xl">Sistema UCI con 20 Dioses OTP</p>
            <div class="mt-8 grid grid-cols-4 gap-4">
                <div class="bg-white/10 p-4 rounded-lg text-center">
                    <p class="text-3xl font-bold">20</p>
                    <p class="text-sm">Dioses</p>
                </div>
                <div class="bg-white/10 p-4 rounded-lg text-center">
                    <p class="text-3xl font-bold">100%</p>
                    <p class="text-sm">Rust</p>
                </div>
                <div class="bg-white/10 p-4 rounded-lg text-center">
                    <p class="text-3xl font-bold">OTP</p>
                    <p class="text-sm">Pattern</p>
                </div>
                <div class="bg-white/10 p-4 rounded-lg text-center">
                    <p class="text-3xl font-bold">v15</p>
                    <p class="text-sm">Versión</p>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#)
}

async fn api_status() -> Json<serde_json::Value> {
    let db_healthy = {
        let guard = DB.read().await;
        guard.is_some()
    };

    let active_gods = olympus_services::get_active_gods_count().await;

    Json(serde_json::json!({
        "status": "active",
        "version": "v15.0.0",
        "mode": "Olympus + SurrealDB",
        "surreal_db": db_healthy,
        "active_gods": active_gods,
        "message": "System ready with 20 divine gods"
    }))
}

async fn api_olympus_gods() -> Json<serde_json::Value> {
    let gods = olympus_services::get_gods_status().await;
    Json(serde_json::json!({
        "gods": gods,
        "total": gods.len()
    }))
}

async fn api_olympus_god(axum::extract::Path(domain): axum::extract::Path<String>) -> Json<serde_json::Value> {
    let domain_lower = domain.to_lowercase();
    let domain_enum = match domain_lower.as_str() {
        "governance" => olympus_services::DivineDomain::Governance,
        "clinical" => olympus_services::DivineDomain::Clinical,
        "security" => olympus_services::DivineDomain::Security,
        "persistence" => olympus_services::DivineDomain::Persistence,
        "analysis" => olympus_services::DivineDomain::Analysis,
        _ => olympus_services::DivineDomain::Governance,
    };
    
    let god = olympus_services::get_god_by_domain(domain_enum).await;
    Json(serde_json::json!({
        "god": god
    }))
}

async fn api_glasgow() -> Json<serde_json::Value> {
    let result = olympus_services::athena::calculate_glasgow(3, 4, 5).await;
    Json(serde_json::json!({
        "scale": "Glasgow",
        "result": result
    }))
}

async fn api_sofa() -> Json<serde_json::Value> {
    let result = olympus_services::athena::calculate_sofa(2, 1, 1, 2, 1, 1).await;
    Json(serde_json::json!({
        "scale": "SOFA",
        "result": result
    }))
}

async fn api_patients() -> Json<serde_json::Value> {
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        match client.query("SELECT * FROM patient").await {
            Ok(mut response) => {
                if let Ok(patients) = response.take::<Vec<serde_json::Value>>(0) {
                    return Json(serde_json::json!({ "patients": patients }));
                }
            }
            Err(e) => {
                info!("DB query error: {}", e);
            }
        }
    }
    
    Json(serde_json::json!({
        "patients": [
            {"first_name": "Juan", "last_name": "Pérez", "principal_diagnosis": "Neumonía", "gender": "Male", "date_of_birth": "1960-05-15", "id": "1"},
            {"first_name": "María", "last_name": "García", "principal_diagnosis": "Postquirúrgico", "gender": "Female", "date_of_birth": "1975-08-22", "id": "2"}
        ]
    }))
}

async fn api_patient(axum::extract::Path(id): axum::extract::Path<String>) -> Json<serde_json::Value> {
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        match client.query("SELECT * FROM patient WHERE id = $id").bind(("id", format!("patient:{}", id))).await {
            Ok(mut response) => {
                if let Ok(patients) = response.take::<Vec<serde_json::Value>>(0) {
                    if let Some(patient) = patients.into_iter().next() {
                        return Json(serde_json::json!({ "patient": patient }));
                    }
                }
            }
            Err(e) => {
                info!("DB query error: {}", e);
            }
        }
    }
    
    Json(serde_json::json!({
        "patient": {"first_name": "Demo", "last_name": "Patient"}
    }))
}

async fn api_login() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "token": "demo_token_12345",
        "user_id": "user_001",
        "role": "Admin",
        "message": "Login successful - Zeus approves"
    }))
}

async fn api_logout() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Logged out - Hades secures your session"
    }))
}

async fn api_stats() -> Json<serde_json::Value> {
    let active_gods = olympus_services::get_active_gods_count().await;
    
    Json(serde_json::json!({
        "total_patients": 24,
        "active_patients": 12,
        "total_assessments": 156,
        "critical_patients": 3,
        "stable_patients": 8,
        "warning_patients": 1,
        "olympus_gods": active_gods
    }))
}
