// src/main.rs
use axum::{routing::post, Json, Router};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use uci::uci::scale::glasgow::{Glasgow, GlasgowRequest, GlasgowResponse};

#[tokio::main]
async fn main() {
    // Verificamos que la carpeta dist exista (generada por trunk build)
    if !std::path::Path::new("dist").exists() {
        eprintln!("ERROR: No se encuentra la carpeta 'dist/'");
        eprintln!("   Debes ejecutar 'trunk build' primero para compilar el frontend.");
        std::process::exit(1);
    }

    let app = Router::new()
        // API Endpoints
        .route("/api/glasgow", post(calculate_glasgow))
        // Servir archivos estáticos desde dist
        .fallback_service(
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        );

    println!("¡Servidor Axum arrancando...");
    println!("http://localhost:3000 → Aplicación UCI (Leptos + Axum)");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("No se pudo bindear el puerto 3000 (¿ya está en uso?)");

    println!("¡LISTO! Servidor corriendo en http://localhost:3000");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Handler para calcular la escala de Glasgow
async fn calculate_glasgow(Json(payload): Json<GlasgowRequest>) -> Json<GlasgowResponse> {
    // Intentamos crear la escala con los valores recibidos
    // Si hay error (valores fuera de rango), retornamos un score 0 y el error
    match Glasgow::from_u8(payload.eye, payload.verbal, payload.motor) {
        Ok(glasgow) => {
            let (diagnosis, recommendation) = glasgow.result();
            Json(GlasgowResponse {
                score: glasgow.score(),
                diagnosis,
                recommendation,
            })
        }
        Err(e) => Json(GlasgowResponse {
            score: 0,
            diagnosis: "Error".to_string(),
            recommendation: e,
        }),
    }
}
