// src/infrastructure/surreal.rs
// OLYMPUS v13 - SurrealDB Client
// Persistencia a largo plazo

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SurrealError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrealConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for SurrealConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8000".to_string(),
            namespace: "olympus".to_string(),
            database: "v13".to_string(),
            username: None,
            password: None,
        }
    }
}

#[derive(Debug)]
pub struct SurrealStore {
    config: SurrealConfig,
    client: Arc<tokio::sync::RwLock<Option<surrealdb::Surreal<surrealdb::engine::any::Any>>>>,
}

impl SurrealStore {
    pub fn new(config: SurrealConfig) -> Self {
        Self {
            config,
            client: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    pub fn default() -> Self {
        Self::new(SurrealConfig::default())
    }

    pub async fn connect(&self) -> Result<(), SurrealError> {
        let config = self.config.clone();
        let connection = surrealdb::engine::any::connect(config.url).await
            .map_err(|e| SurrealError::ConnectionFailed(e.to_string()))?;
        
        let mut client = self.client.write().await;
        *client = Some(connection);

        if let Some(ref mut c) = *client {
            c.use_ns(&config.namespace).await
                .map_err(|e| SurrealError::ConnectionFailed(e.to_string()))?;
            c.use_db(&config.database).await
                .map_err(|e| SurrealError::ConnectionFailed(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn create<T: Serialize>(&self, table: &str, data: &T) -> Result<serde_json::Value, SurrealError> {
        let client = self.client.read().await;
        let data = serde_json::to_value(data)
            .map_err(|e| SurrealError::SerializationError(e.to_string()))?;

        let mut response = client.as_ref()
            .ok_or_else(|| SurrealError::ConnectionFailed("Not connected".to_string()))?
            .query(format!("CREATE {} CONTENT {}", table, data))
            .await
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        let mut result: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        Ok(result.pop().unwrap_or(serde_json::Value::Null))
    }

    pub async fn select<T: for<'de> serde::Deserialize<'de>>(&self, table: &str, id: &str) -> Result<Option<T>, SurrealError> {
        let client = self.client.read().await;
        
        let mut response = client.as_ref()
            .ok_or_else(|| SurrealError::ConnectionFailed("Not connected".to_string()))?
            .query(format!("SELECT * FROM {} WHERE id = $id", table))
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        let result: Option<T> = response.take(0)
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        Ok(result)
    }

    pub async fn update<T: Serialize>(&self, _table: &str, id: &str, data: &T) -> Result<serde_json::Value, SurrealError> {
        let client = self.client.read().await;
        let data = serde_json::to_value(data)
            .map_err(|e| SurrealError::SerializationError(e.to_string()))?;

        let mut response = client.as_ref()
            .ok_or_else(|| SurrealError::ConnectionFailed("Not connected".to_string()))?
            .query(format!("UPDATE {} CONTENT {}", id, data))
            .await
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        let mut result: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        Ok(result.pop().unwrap_or(serde_json::Value::Null))
    }

    pub async fn delete(&self, _table: &str, id: &str) -> Result<(), SurrealError> {
        let client = self.client.read().await;
        
        client.as_ref()
            .ok_or_else(|| SurrealError::ConnectionFailed("Not connected".to_string()))?
            .query(format!("DELETE {}", id))
            .await
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn query<T: for<'de> serde::Deserialize<'de>>(&self, query: &str) -> Result<Vec<T>, SurrealError> {
        let client = self.client.read().await;
        
        let mut response = client.as_ref()
            .ok_or_else(|| SurrealError::ConnectionFailed("Not connected".to_string()))?
            .query(query)
            .await
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        let result: Vec<T> = response.take(0)
            .map_err(|e| SurrealError::QueryFailed(e.to_string()))?;

        Ok(result)
    }

    pub async fn health_check(&self) -> Result<bool, SurrealError> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            Ok(c.health().await.is_ok())
        } else {
            Err(SurrealError::ConnectionFailed("Not connected".to_string()))
        }
    }
}

pub type SharedSurrealStore = Arc<SurrealStore>;
