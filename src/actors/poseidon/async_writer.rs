// src/actors/poseidon/async_writer.rs
// OLYMPUS v13 - Poseidon Async Writer
// Escritura asíncrona a SurrealDB con retry infinito

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use tracing::warn;

use crate::infrastructure::SurrealStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTask {
    pub id: String,
    pub table: String,
    pub data: serde_json::Value,
    pub created_at: Instant,
    pub attempts: u32,
}

impl WriteTask {
    pub fn new(table: &str, data: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            table: table.to_string(),
            data,
            created_at: Instant::now(),
            attempts: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncWriter {
    surreal: Option<Arc<SurrealStore>>,
    tx: mpsc::Sender<WriteTask>,
    rx: mpsc::Receiver<WriteTask>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl AsyncWriter {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        
        Self {
            surreal: None,
            tx,
            rx,
            running,
        }
    }
    
    pub fn set_store(&mut self, store: Arc<SurrealStore>) {
        self.surreal = Some(store);
    }
    
    pub async fn queue_write(&self, table: &str, data: serde_json::Value) {
        let task = WriteTask::new(table, data);
        let _ = self.tx.send(task).await;
    }
    
    pub async fn start(&mut self) {
        let running = self.running.clone();
        let surreal = self.surreal.clone();
        let mut rx = self.rx.clone();
        
        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(task) = rx.recv().await {
                    if let Some(ref store) = surreal {
                        let mut attempts = 0;
                        loop {
                            attempts += 1;
                            
                            match store.create(&task.table, &task.data).await {
                                Ok(_) => break,
                                Err(e) => {
                                    if attempts >= 10 {
                                        warn!("Failed to write after {} attempts: {}", attempts, e);
                                        break;
                                    }
                                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
