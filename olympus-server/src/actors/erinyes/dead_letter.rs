// src/actors/erinyes/dead_letter.rs
// OLYMPUS v13 - Erinyes Dead Letter Queue
// Cola de mensajes no entregados con persistencia Valkey + SurrealDB

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::ValkeyStore;
use crate::actors::GodName;
use crate::traits::message::ActorMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    pub id: String,
    pub message: ActorMessage,
    pub original_to: GodName,
    pub failed_at: chrono::DateTime<chrono::Utc>,
    pub attempt_count: u32,
    pub last_error: String,
    pub status: DeadLetterStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeadLetterStatus {
    Pending,
    Retrying,
    Delivered,
    Abandoned,
}

impl DeadLetter {
    pub fn new(message: ActorMessage, original_to: GodName, error: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message,
            original_to,
            failed_at: chrono::Utc::now(),
            attempt_count: 0,
            last_error: error,
            status: DeadLetterStatus::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadLetterQueue {
    valkey: Arc<ValkeyStore>,
    queue_key: String,
    hash_key: String,
}

impl DeadLetterQueue {
    pub fn new(valkey: Arc<ValkeyStore>) -> Self {
        Self {
            valkey,
            queue_key: "olympus:dead_letters:queue".to_string(),
            hash_key: "olympus:dead_letters:data".to_string(),
        }
    }
    
    pub async fn push(&self, dead_letter: DeadLetter) {
        // Store in Valkey queue
        let json = serde_json::to_string(&dead_letter).unwrap_or_default();
        let _ = self.valkey.lpush(&self.queue_key, &dead_letter.id).await;
        let _ = self.valkey.hset(&self.hash_key, &dead_letter.id, &json).await;
    }
    
    pub async fn pop(&self) -> Option<DeadLetter> {
        if let Some(id) = self.valkey.rpop(&self.queue_key).await.ok().flatten() {
            if let Some(json) = self.valkey.hget(&self.hash_key, &id).await.ok().flatten() {
                return serde_json::from_str(&json).ok();
            }
        }
        None
    }
    
    pub async fn len(&self) -> usize {
        self.valkey.llen(&self.queue_key).await.ok().unwrap_or(0) as usize
    }
    
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    pub async fn mark_delivered(&self, id: &str) {
        if let Some(json) = self.valkey.hget(&self.hash_key, id).await.ok().flatten() {
            if let Ok(mut dl) = serde_json::from_str::<DeadLetter>(&json) {
                dl.status = DeadLetterStatus::Delivered;
                dl.attempt_count += 1;
                let updated = serde_json::to_string(&dl).unwrap_or_default();
                let _ = self.valkey.hset(&self.hash_key, id, &updated).await;
            }
        }
    }
    
    pub async fn increment_attempts(&self, id: &str, error: String) {
        if let Some(json) = self.valkey.hget(&self.hash_key, id).await.ok().flatten() {
            if let Ok(mut dl) = serde_json::from_str::<DeadLetter>(&json) {
                dl.attempt_count += 1;
                dl.last_error = error;
                dl.status = DeadLetterStatus::Retrying;
                let updated = serde_json::to_string(&dl).unwrap_or_default();
                let _ = self.valkey.hset(&self.hash_key, id, &updated).await;
            }
        }
    }
}
