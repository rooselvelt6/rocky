use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Connect to SurrealDB and return a configured client
pub async fn connect() -> Result<Surreal<Client>, Error> {
    // Load .env file
    dotenvy::dotenv().ok();

    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "hospital".to_string());
    let db_db = std::env::var("DB_DB").unwrap_or_else(|_| "uci".to_string());

    let mut retry_count = 0;
    let max_retries = 5;
    let retry_delay = std::time::Duration::from_secs(3);

    let db = loop {
        tracing::info!(
            "🔄 Intentando conectar a SurrealDB en {} (intento {}/{})...",
            db_host,
            retry_count + 1,
            max_retries
        );

        match Surreal::new::<Ws>(&db_host).await {
            Ok(client) => break client,
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    tracing::error!(
                        "❌ No se pudo conectar a SurrealDB tras {} intentos: {}",
                        max_retries,
                        e
                    );
                    return Err(e);
                }
                tokio::time::sleep(retry_delay).await;
            }
        }
    };

    // Sign in with credentials
    db.signin(Root {
        username: &db_user,
        password: &db_pass,
    })
    .await?;

    // Use the namespace and database
    db.use_ns(db_ns).use_db(db_db).await?;

    tracing::info!("✅ Conexión exitosa a SurrealDB en {}", db_host);

    Ok(db)
}
