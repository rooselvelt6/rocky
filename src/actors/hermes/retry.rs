// src/actors/hermes/retry.rs
// OLYMPUS v15 - Hermes Retry System
// Sistema de reintentos con backoff exponencial

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, warn};

use super::{GodName, DeliveryTracker};
use crate::traits::message::ActorMessage;
use crate::errors::ActorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_lost".to_string(),
                "mailbox_full".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryableMessage {
    pub message: ActorMessage,
    pub to: GodName,
    pub attempts: u32,
    pub next_retry_at: Instant,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RetryQueue {
    messages: Arc<RwLock<Vec<RetryableMessage>>>,
    config: RetryConfig,
    delivery_tracker: Arc<DeliveryTracker>,
}

impl RetryQueue {
    pub fn new(config: RetryConfig, delivery_tracker: Arc<DeliveryTracker>) -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            config,
            delivery_tracker,
        }
    }

    pub async fn enqueue(&self, message: ActorMessage, to: GodName, error: String) {
        let retryable = RetryableMessage {
            message: message.clone(),
            to,
            attempts: 1,
            next_retry_at: Instant::now() + Duration::from_millis(self.config.initial_delay_ms),
            last_error: Some(error),
        };

        let mut messages = self.messages.write().await;
        messages.push(retryable);
        
        info!(
            message_id = %message.id,
            to = %to,
            "Message queued for retry"
        );
    }

    pub async fn process_retries<F, Fut>(&self, deliver_fn: F)
    where
        F: Fn(ActorMessage, GodName) -> Fut,
        Fut: std::future::Future<Output = Result<(), ActorError>>,
    {
        let now = Instant::now();
        let mut messages = self.messages.write().await;
        let mut to_remove = Vec::new();

        for (idx, retryable) in messages.iter_mut().enumerate() {
            if now >= retryable.next_retry_at {
                if retryable.attempts >= self.config.max_attempts {
                    // Max attempts reached, send to dead letter
                    warn!(
                        message_id = %retryable.message.id,
                        attempts = retryable.attempts,
                        "Max retry attempts reached, sending to dead letter"
                    );
                    
                    self.delivery_tracker
                        .record_dead_letter(&retryable.message.id)
                        .await;
                    
                    to_remove.push(idx);
                    continue;
                }

                // Attempt retry
                match deliver_fn(retryable.message.clone(), retryable.to).await {
                    Ok(()) => {
                        info!(
                            message_id = %retryable.message.id,
                            attempt = retryable.attempts,
                            "Message delivered after retry"
                        );
                        
                        self.delivery_tracker
                            .record_delivery(&retryable.message.id)
                            .await;
                        
                        to_remove.push(idx);
                    }
                    Err(e) => {
                        retryable.attempts += 1;
                        retryable.last_error = Some(e.to_string());
                        
                        // Calculate next retry time with exponential backoff
                        let delay_ms = (self.config.initial_delay_ms as f64
                            * self.config.backoff_multiplier.powi(retryable.attempts as i32 - 1))
                            .min(self.config.max_delay_ms as f64) as u64;
                        
                        retryable.next_retry_at = now + Duration::from_millis(delay_ms);
                        
                        warn!(
                            message_id = %retryable.message.id,
                            attempt = retryable.attempts,
                            next_retry_ms = delay_ms,
                            error = %e,
                            "Retry failed, scheduling next attempt"
                        );

                        self.delivery_tracker
                            .record_failure(&retryable.message.id, e.to_string())
                            .await;
                    }
                }
            }
        }

        // Remove processed messages
        for idx in to_remove.iter().rev() {
            messages.remove(*idx);
        }
    }

    pub async fn len(&self) -> usize {
        let messages = self.messages.read().await;
        messages.len()
    }

    pub async fn is_empty(&self) -> bool {
        let messages = self.messages.read().await;
        messages.is_empty()
    }

    pub async fn get_pending_count(&self) -> usize {
        let messages = self.messages.read().await;
        messages.len()
    }

    pub async fn clear(&self) {
        let mut messages = self.messages.write().await;
        messages.clear();
    }
}

#[derive(Debug)]
pub struct RetryWorker {
    queue: Arc<RetryQueue>,
    shutdown: Arc<RwLock<bool>>,
}

impl RetryWorker {
    pub fn new(queue: Arc<RetryQueue>) -> Self {
        Self {
            queue,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start<F, Fut>(&self, deliver_fn: F)
    where
        F: Fn(ActorMessage, GodName) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), ActorError>> + Send,
    {
        let queue = self.queue.clone();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            loop {
                // Check shutdown signal
                if *shutdown.read().await {
                    break;
                }

                // Process retries every 100ms
                queue.process_retries(&deliver_fn).await;
                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    pub async fn shutdown(&self) {
        let mut shutdown = self.shutdown.write().await;
        *shutdown = true;
    }
}
