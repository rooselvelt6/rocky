use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Connect to SurrealDB and return a configured client
pub async fn connect() -> Result<Surreal<Client>, Error> {
    // Load .env file
    dotenvy::dotenv().ok();

    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "8000".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "hospital".to_string());
    let db_db = std::env::var("DB_DB").unwrap_or_else(|_| "uci".to_string());

    let max_retries: u32 = std::env::var("DB_MAX_RETRIES")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
    let retry_delay_secs: u64 = std::env::var("DB_RETRY_DELAY")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    let addr = format!("{}:{}", db_host, db_port);
    let retry_delay = std::time::Duration::from_secs(retry_delay_secs);
    let mut retry_count = 0;

    let db = loop {
        retry_count += 1;
        tracing::info!(
            "🔄 Intentando conectar a SurrealDB en {} (intento {}/{})...",
            addr,
            retry_count,
            max_retries
        );

        match Surreal::new::<Ws>(&addr).await {
            Ok(client) => break client,
            Err(e) => {
                if retry_count >= max_retries {
                    tracing::error!(
                        "❌ No se pudo conectar a SurrealDB tras {} intentos en {}: {}",
                        max_retries,
                        addr,
                        e
                    );
                    return Err(e);
                }
                tracing::warn!(
                    "⚠️ Error de conexión (reintentando en {}s): {}",
                    retry_delay_secs,
                    e
                );
                tokio::time::sleep(retry_delay).await;
            }
        }
    };

    // Sign in with credentials
    if let Err(e) = db
        .signin(Root {
            username: &db_user,
            password: &db_pass,
        })
        .await
    {
        tracing::error!("❌ Error de autenticación en SurrealDB: {}", e);
        return Err(e);
    }

    // Use the namespace and database
    if let Err(e) = db.use_ns(&db_ns).use_db(&db_db).await {
        tracing::error!("❌ Error al seleccionar NS/DB ({}/{}): {}", db_ns, db_db, e);
        return Err(e);
    }

    tracing::info!(
        "✅ Conexión exitosa a SurrealDB en {} (NS: {}, DB: {})",
        addr,
        db_ns,
        db_db
    );

    Ok(db)
}
