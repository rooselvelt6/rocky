// src/actors/hermes/delivery.rs
// OLYMPUS v13 - Hermes Delivery Tracker
// Seguimiento de entrega de mensajes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryTracking {
    pub message_id: String,
    pub to: GodName,
    pub status: DeliveryStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attempts: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    InTransit,
    Delivered,
    Failed,
    DeadLettered,
}

#[derive(Debug, Clone)]
pub struct DeliveryTracker {
    trackings: Arc<RwLock<HashMap<String, DeliveryTracking>>>,
}

impl DeliveryTracker {
    pub fn new() -> Self {
        Self {
            trackings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_tracking(&self, message_id: &str, to: GodName) -> DeliveryTrackingHandle {
        let tracking = DeliveryTracking {
            message_id: message_id.to_string(),
            to,
            status: DeliveryStatus::InTransit,
            started_at: chrono::Utc::now(),
            delivered_at: None,
            attempts: 0,
            last_error: None,
        };

        let mut trackings = self.trackings.write().await;
        trackings.insert(message_id.to_string(), tracking.clone());

        DeliveryTrackingHandle {
            message_id: message_id.to_string(),
            trackings: self.trackings.clone(),
        }
    }

    pub async fn record_delivery(&self, message_id: &str) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(message_id) {
            tracking.status = DeliveryStatus::Delivered;
            tracking.delivered_at = Some(chrono::Utc::now());
        }
    }

    pub async fn record_failure(&self, message_id: &str, error: String) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(message_id) {
            tracking.attempts += 1;
            tracking.last_error = Some(error);
            tracking.status = DeliveryStatus::Failed;
        }
    }

    pub async fn record_dead_letter(&self, message_id: &str) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(message_id) {
            tracking.status = DeliveryStatus::DeadLettered;
        }
    }

    pub async fn get_tracking(&self, message_id: &str) -> Option<DeliveryTracking> {
        let trackings = self.trackings.read().await;
        trackings.get(message_id).cloned()
    }

    pub async fn delivered_count(&self) -> u64 {
        let trackings = self.trackings.read().await;
        trackings
            .values()
            .filter(|t| t.status == DeliveryStatus::Delivered)
            .count() as u64
    }

    pub async fn failed_count(&self) -> u64 {
        let trackings = self.trackings.read().await;
        trackings
            .values()
            .filter(|t| t.status == DeliveryStatus::Failed)
            .count() as u64
    }

    pub async fn pending_count(&self) -> u64 {
        let trackings = self.trackings.read().await;
        trackings
            .values()
            .filter(|t| t.status == DeliveryStatus::Pending || t.status == DeliveryStatus::InTransit)
            .count() as u64
    }

    pub async fn get_failed_messages(&self) -> Vec<DeliveryTracking> {
        let trackings = self.trackings.read().await;
        trackings
            .values()
            .filter(|t| t.status == DeliveryStatus::Failed)
            .cloned()
            .collect()
    }

    pub async fn cleanup_old_trackings(&self, max_age: chrono::Duration) {
        let mut trackings = self.trackings.write().await;
        let now = chrono::Utc::now();
        trackings.retain(|_, tracking| {
            now.signed_duration_since(tracking.started_at) < max_age
        });
    }
}

#[derive(Debug, Clone)]
pub struct DeliveryTrackingHandle {
    message_id: String,
    trackings: Arc<RwLock<HashMap<String, DeliveryTracking>>>,
}

impl DeliveryTrackingHandle {
    pub async fn record_delivery(&self) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(&self.message_id) {
            tracking.status = DeliveryStatus::Delivered;
            tracking.delivered_at = Some(chrono::Utc::now());
        }
    }

    pub async fn record_failure(&self, error: String) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(&self.message_id) {
            tracking.attempts += 1;
            tracking.last_error = Some(error);
            tracking.status = DeliveryStatus::Failed;
        }
    }

    pub async fn increment_attempt(&self) {
        let mut trackings = self.trackings.write().await;
        if let Some(tracking) = trackings.get_mut(&self.message_id) {
            tracking.attempts += 1;
        }
    }

    pub async fn get_attempts(&self) -> u32 {
        let trackings = self.trackings.read().await;
        trackings
            .get(&self.message_id)
            .map(|t| t.attempts)
            .unwrap_or(0)
    }
}
