use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static DB: Lazy<Arc<RwLock<Option<Surreal<Any>>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8000".to_string(),
            namespace: "hospital".to_string(),
            database: "uci".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

pub async fn init_db(config: DbConfig) -> Result<(), String> {
    let db = surrealdb::engine::any::connect(&config.url)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .map_err(|e| format!("Namespace/DB failed: {}", e))?;

    let mut guard = DB.write().await;
    *guard = Some(db);
    
    println!("✅ Database connected: {}/{}", config.namespace, config.database);
    Ok(())
}

pub async fn get_db() -> Arc<RwLock<Option<Surreal<Any>>>> {
    DB.clone()
}

pub async fn health_check() -> bool {
    let guard = DB.read().await;
    if let Some(ref db) = *guard {
        db.health().await.is_ok()
    } else {
        false
    }
}
