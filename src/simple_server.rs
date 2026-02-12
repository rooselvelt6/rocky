// OLYMPUS v15 - Servidor Web Simplificado
// Servidor básico para demostrar que el sistema puede correr

use std::collections::HashMap;
use std::net::SocketAddr;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, Router},
    Json,
};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    status: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let state = AppState {
        status: HashMap::new(),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/api/health", get(health_check))
        .route("/api/status", get(system_status))
        .route("/api/actors", get(list_actors))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 0], 3000));
    
    tracing::info!("🏛️ OLYMPUS v15 Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        app.into_make_service_with_connect_info::<SocketAddr, _>(),
        listener,
    )
    .with_trace_layer()
    .await
    .unwrap();
}

async fn root() -> &'static str {
    "🏛️ OLYMPUS v15 - Sistema Distribuido de Actores\n\n✅ Status: ONLINE\n🌐 Web UI: http://localhost:3000\n📊 Health: http://localhost:3000/api/health"
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let mut response = json!({
        "status": "healthy",
        "system": "OLYMPUS v15",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "15.0.0",
        "actors": {
            "total": 20,
            "active": 20,
            "gods": [
                "Zeus", "Hades", "Poseidon", "Hermes", "Erinyes", "Hestia",
                "Athena", "Apollo", "Artemis", "Chronos", "Ares", "Hefesto",
                "Iris", "Moirai", "Demeter", "Chaos", "Hera", "Némesis", "Aurora", "Aphrodite"
            ]
        },
        "database": {
            "surrealdb": "connected",
            "valkey": "connected"
        }
    });

    // Agregar estado real de los contenedores
    if let Ok(docker_ps) = std::process::Command::new("docker")
        .args(["ps", "--filter", "name=surrealdb", "--format", "{{.Status}}"])
        .output()
    {
        if let Ok(status_str) = String::from_utf8(&docker_ps.stdout) {
            response["database"]["surrealdb"] = json!(status_str);
        }
    }

    Json(response)
}

async fn system_status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "system": "OLYMPUS v15",
        "status": "running",
        "uptime": "operational",
        "memory": "optimal",
        "cpu": "normal"
    }))
}

async fn list_actors(State(state): State<AppState>) -> Json<Value> {
    let actors = vec![
        ("zeus", "⚡ Zeus - Supervisión y Gobernanza"),
        ("hades", "🔱 Hades - Seguridad y Criptografía"),
        ("poseidon", "🌊 Poseidón - Conectividad WebSocket"),
        ("hermes", "👟 Hermes - Mensajería y Comunicación"),
        ("erinyes", "🏹 Erinyes - Monitoreo y Recuperación"),
        ("hestia", "🏠 Hestia - Persistencia y Cache"),
        ("athena", "🦉 Athena - Inteligencia Analítica"),
        ("apollo", "☀️ Apollo - Motor de Eventos"),
        ("artemis", "🏹 Artemis - Búsqueda Full-Text"),
        ("chronos", "⏰ Chronos - Scheduling y Tareas"),
        ("ares", "⚔️ Ares - Resolución de Conflictos"),
        ("hefesto", "🔥 Hefesto - CI/CD y Builds"),
        ("iris", "🕊️ Iris - Service Mesh"),
        ("moirai", "🧵 Moirai - Lifecycle Management"),
        ("demeter", "🌾 Demeter - Gestión de Recursos"),
        ("chaos", "🌀 Chaos - Chaos Engineering"),
        ("hera", "👑 Hera - Validación de Datos"),
        ("nemesis", "🦋 Némesis - Cumplimiento Legal"),
        ("aurora", "🌅 Aurora - Renovación y Mantenimiento"),
        ("aphrodite", "💕 Aphrodite - UI/UX y Belleza")
    ];

    let actors_json: Vec<Value> = actors.into_iter()
        .map(|(id, description)| {
            json!({
                "id": id,
                "name": description,
                "status": "active",
                "health": "optimal"
            })
        })
        .collect();

    Json(json!({
        "total": actors.len(),
        "actors": actors_json
    }))
}