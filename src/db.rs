use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Connect to SurrealDB and return a configured client
pub async fn connect() -> Result<Surreal<Client>, Error> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "hospital".to_string());
    let db_db = std::env::var("DB_DB").unwrap_or_else(|_| "uci".to_string());

    // Connect to SurrealDB instance
    let db = Surreal::new::<Ws>(db_host).await?;

    // Sign in with credentials
    db.signin(Root {
        username: &db_user,
        password: &db_pass,
    })
    .await?;

    // Use the namespace and database
    db.use_ns(db_ns).use_db(db_db).await?;

    tracing::info!("✅ Connected to SurrealDB");

    Ok(db)
}
