// src/actors/hestia/async_buffer.rs
// OLYMPUS v13 - Hestia Async Buffer
// Buffer de escritura asíncrona a SurrealDB

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

use super::ValkeyStore;
use super::SurrealStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedWrite {
    pub id: String,
    pub table: String,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: Instant,
    pub attempts: u32,
}

impl BufferedWrite {
    pub fn new(table: &str, key: String, value: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            table: table.to_string(),
            key,
            value,
            created_at: Instant::now(),
            attempts: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncBuffer {
    valkey: Arc<ValkeyStore>,
    surreal: Arc<SurrealStore>,
    tx: mpsc::Sender<BufferedWrite>,
    queue_key: String,
}

impl AsyncBuffer {
    pub fn new(valkey: Arc<ValkeyStore>, surreal: Arc<SurrealStore>) -> Self {
        let (tx, mut rx) = mpsc::channel(1000);
        let queue_key = "olympus:hestia:async_buffer".to_string();
        
        let valkey_clone = valkey.clone();
        let surreal_clone = surreal.clone();
        
        tokio::spawn(async move {
            while let Some(write) = rx.recv().await {
                let json = serde_json::to_string(&write).unwrap_or_default();
                let _ = valkey_clone.lpush(&queue_key, &json).await;
                
                // Attempt to write to SurrealDB
                let mut attempts = 0;
                loop {
                    match surreal_clone.create(&write.table, &write.value).await {
                        Ok(_) => {
                            let _ = valkey_clone.lpush(&format!("{}:processed", queue_key), &write.id).await;
                            break;
                        }
                        Err(_) => {
                            attempts += 1;
                            if attempts >= 5 {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(100 * attempts)).await;
                        }
                    }
                }
            }
        });
        
        Self {
            valkey,
            surreal,
            tx,
            queue_key,
        }
    }
    
    pub async fn push(&self, table: &str, key: String, value: serde_json::Value) {
        let write = BufferedWrite::new(table, key, value);
        let _ = self.tx.send(write).await;
    }
    
    pub async fn len(&self) -> usize {
        self.valkey.llen(&self.queue_key).await.ok().unwrap_or(0) as usize
    }
    
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    pub async fn flush(&self) {
        // In real implementation, would process all pending writes
    }
}
