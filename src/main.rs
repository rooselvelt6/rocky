/// Main entry point for Olympus v15
/// Bootloader: Genesis
/// Architecture: Actors (OTP-model)

use axum::{
    routing::{get, post},
    Router, Json,
};
use tracing::{info, error};
use std::net::SocketAddr;

mod actors;     // Panteón (V15)
mod traits;     // Interfaces
mod system;     // Genesis & Runners
mod errors;     // Errores
mod models;     // Modelos de datos
mod infrastructure; // DB & Cache

use crate::system::Genesis;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Logging Initialization
    tracing_subscriber::fmt::init();
    
    info!("🏔️  OLYMPUS SYSTEM v15 - STARTING UP  🏔️");
    info!("⚡  Target: The Trinity (Zeus, Hades, Poseidon) + Pantheon");
    info!("🏛️  Model:  Elixir/OTP Actors");

    // 2. Genesis Ignition (The Big Bang)
    // Instancia y arranca los 20 dioses en sus propios hilos (tasks)
    match Genesis::ignite().await {
        Ok(_) => info!("✨ Genesis completado exitosamente. Los dioses caminan entre nosotros."),
        Err(e) => {
            error!("💀 Genesis falló: {}", e);
            return Err(e);
        }
    }

    // 3. API Gateway (Axum)
    // En v15, la API es solo una interfaz para enviar mensajes a los actores.
    // Por ahora, levantamos un health check simple que confirma que el proceso vive.
    // TODO: Conectar Axum con los canales de los actores (Hermes Bridge)
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/status", get(system_status));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🌍 API Gateway escuchando en http://{}", addr);

    // Mantenemos el main loop vivo con el servidor Web
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn health_check() -> &'static str {
    "Olympus v15 is RUNNING. The Gods are awake."
}

async fn system_status() -> Json<serde_json::Value> {
    // En el futuro, consultaríamos a Zeus
    Json(serde_json::json!({
        "status": "active",
        "version": "v15.0.0",
        "trinity": ["Zeus", "Hades", "Poseidon"],
        "message": "System booted via Genesis"
    }))
}
