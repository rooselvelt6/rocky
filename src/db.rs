use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Conecta a SurrealDB y devuelve un cliente configurado (Modo Any para máxima versatilidad)
pub async fn connect() -> Result<Surreal<Any>, Error> {
    // Cargar archivo .env
    dotenvy::dotenv().ok();

    // Heurística ZEUS: Detectar modo de base de datos
    // Modos: "remote" (default/Docker) o "embedded" (ZEUS Native)
    let db_mode = std::env::var("DB_MODE").unwrap_or_else(|_| "remote".to_string());
    
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "hospital".to_string());
    let db_db = std::env::var("DB_DB").unwrap_or_else(|_| "uci".to_string());

    let db = if db_mode == "embedded" {
        tracing::info!("⚡ MODO ZEUS (Embebido): Iniciando base de datos RocksDB local...");
        // En modo embebido, la URL es el path al directorio de datos
        let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "rocksdb:uci_data".to_string());
        
        let db = any::connect(&db_path).await?;
        
        // En modo embebido nativo, a menudo no necesitamos signin si es local, 
        // pero por consistencia intentamos usar las credenciales configuradas
        let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
        
        // No hay reintentos en modo embebido de disco local (o funciona o no hay espacio/permisos)
        match db.signin(Root {
            username: &db_user,
            password: &db_pass,
        }).await {
            Ok(_) => tracing::info!("✅ Autenticación interna completada"),
            Err(e) => tracing::warn!("⚠️ Advertencia de autenticación interna (puede ser normal en primer arranque): {}", e),
        }
        
        db
    } else {
        // MODO REMOTO (WS) - Ideal para Docker y Escalamiento
        let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "8000".to_string());
        let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());

        let max_retries: u32 = std::env::var("DB_MAX_RETRIES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .unwrap_or(15);
        let retry_delay_secs: u64 = std::env::var("DB_RETRY_DELAY")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let endpoint = format!("ws://{}:{}", db_host, db_port);
        let retry_delay = std::time::Duration::from_secs(retry_delay_secs);
        let mut retry_count = 0;

        let db = loop {
            retry_count += 1;
            tracing::info!(
                "🔄 Intentando conectar a SurrealDB Remoto en {} (intento {}/{})...",
                endpoint,
                retry_count,
                max_retries
            );

            match any::connect(&endpoint).await {
                Ok(client) => break client,
                Err(e) => {
                    if retry_count >= max_retries {
                        tracing::error!(
                            "❌ No se pudo conectar tras {} intentos en {}: {}",
                            max_retries,
                            endpoint,
                            e
                        );
                        return Err(e);
                    }
                    tracing::warn!("⚠️ Reintentando conexión remota en {}s...", retry_delay_secs);
                    tokio::time::sleep(retry_delay).await;
                }
            }
        };

        db.signin(Root {
            username: &db_user,
            password: &db_pass,
        }).await?;
        
        tracing::info!("✅ Conexión WS establecida con éxito.");
        db
    };

    // Usar el namespace y base de datos
    db.use_ns(&db_ns).use_db(&db_db).await?;

    tracing::info!(
        "🚀 Sistema de Persistencia ZEUS Listo (Modo: {}, NS: {}, DB: {})",
        db_mode,
        db_ns,
        db_db
    );

    Ok(db)
}

