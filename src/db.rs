use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

/// Connect to SurrealDB and return a configured client
pub async fn connect() -> Result<Surreal<Client>, Error> {
    // Connect to local SurrealDB instance
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Sign in with root credentials
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    // Use the hospital namespace and uci database
    db.use_ns("hospital").use_db("uci").await?;

    tracing::info!("✅ Connected to SurrealDB (hospital/uci)");

    Ok(db)
}
