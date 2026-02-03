// src/actors/erinyes/alerts.rs
// OLYMPUS v13 - Erinyes Alert System
// Sistema de alertas y notificaciones

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
    pub source: GodName,
    pub title: String,
    pub message: String,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ActorDown { actor: GodName },
    HighErrorRate { threshold: f64 },
    RecoveryFailed { actor: GodName },
    DeadLetterQueueHigh { threshold: usize },
    SystemDegraded,
}

#[derive(Debug, Clone)]
pub struct AlertSystem {
    alerts: Arc<RwLock<Vec<Alert>>>,
    rules: Arc<RwLock<Vec<AlertRule>>>,
    cooldown: Arc<RwLock<HashMap<String, Instant>>>,
}

impl AlertSystem {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            cooldown: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn create_alert(&self, severity: AlertSeverity, source: GodName, title: String, message: String) {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            severity,
            source,
            title,
            message,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };
        
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        
        // Keep only last 1000 alerts
        if alerts.len() > 1000 {
            alerts.remove(0);
        }
    }
    
    pub async fn acknowledge_alert(&self, alert_id: &str, by: &str) {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(by.to_string());
            alert.acknowledged_at = Some(chrono::Utc::now());
        }
    }
    
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter().filter(|a| !a.acknowledged).cloned().collect()
    }
    
    pub async fn get_alert_history(&self, limit: usize) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }
    
    pub async fn critical_count(&self) -> usize {
        let alerts = self.alerts.read().await;
        alerts.iter().filter(|a| !a.acknowledged && a.severity == AlertSeverity::Critical).count()
    }
}
