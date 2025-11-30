// src/main.rs
use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub mod uci;

#[tokio::main]
async fn main() {
    // Verificamos que la carpeta assets exista
    if !std::path::Path::new("assets").exists() {
        eprintln!("ERROR: No se encuentra la carpeta 'assets/'");
        eprintln!("   Crea la carpeta al mismo nivel que Cargo.toml");
        eprintln!("   Ejemplo: assets/index.html y assets/styles.css");
        std::process::exit(1);
    }

    let app = Router::new()
        // Esto sirve TODO lo que esté en assets, incluyendo index.html en la raíz
        .fallback_service(
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html")),
        );

    println!("¡Servidor Axum arrancando...");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("No se pudo bindear el puerto 3000 (¿ya está en uso?)");

    println!("¡LISTO! Servidor corriendo en http://localhost:3000");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
