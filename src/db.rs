use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Connect to SurrealDB and return a configured client
pub async fn connect() -> Result<Surreal<Client>, Error> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    let db_host = "127.0.0.1:8000"; // Ws connector adds scheme generally? Or use string?
                                    // Surreal::new::<Ws>() expects host:port usually.

    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DB_NS").unwrap_or_else(|_| "hospital".to_string());
    let db_db = std::env::var("DB_DB").unwrap_or_else(|_| "uci".to_string());

    // Connect to SurrealDB instance using explicit Ws client
    // In 2.x, Surreal::init() returns Surreal<Any>.
    // Surreal::new::<Ws>(address) returns Surreal<Client> (where Client=WsClient).
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
