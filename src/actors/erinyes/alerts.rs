// src/actors/erinyes/alerts.rs
// OLYMPUS v15 - Erinyes Alert System
// Sistema avanzado de alertas con múltiples canales y smart grouping

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub source: GodName,
    pub title: String,
    pub message: String,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolution_note: Option<String>,
    pub correlation_id: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    pub fn priority(&self) -> u8 {
        match self {
            AlertSeverity::Info => 1,
            AlertSeverity::Warning => 2,
            AlertSeverity::Error => 3,
            AlertSeverity::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertCategory {
    Health,
    Performance,
    Security,
    Recovery,
    System,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub auto_resolve: bool,
    pub channels: Vec<AlertChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ActorDown { actor: GodName, timeout_ms: u64 },
    HighErrorRate { actor: Option<GodName>, threshold: f64, window_seconds: u64 },
    RecoveryFailed { actor: GodName, max_attempts: u32 },
    DeadLetterQueueHigh { threshold: usize },
    SystemDegraded { min_healthy_percent: f64 },
    CircuitBreakerOpen { actor: GodName },
    HighMemoryUsage { actor: GodName, threshold_mb: f64 },
    HighCpuUsage { actor: GodName, threshold_percent: f64 },
    Custom { check: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Log,
    Webhook { url: String, headers: HashMap<String, String> },
    Email { to: Vec<String> },
    Slack { webhook_url: String, channel: String },
    PagerDuty { service_key: String },
    Notification { destinations: Vec<GodName> },
}

#[derive(Debug, Clone)]
pub struct AlertGroup {
    pub correlation_id: String,
    pub alerts: Vec<Alert>,
    pub first_occurrence: chrono::DateTime<chrono::Utc>,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct AlertSystem {
    alerts: Arc<RwLock<Vec<Alert>>>,
    rules: Arc<RwLock<Vec<AlertRule>>>,
    cooldown: Arc<RwLock<HashMap<String, Instant>>>,
    groups: Arc<RwLock<HashMap<String, AlertGroup>>>,
    alert_tx: mpsc::Sender<Alert>,
    alert_rx: Arc<RwLock<mpsc::Receiver<Alert>>>,
}

impl AlertSystem {
    pub fn new() -> Self {
        let (alert_tx, alert_rx) = mpsc::channel(1000);
        
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            cooldown: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            alert_tx,
            alert_rx: Arc::new(RwLock::new(alert_rx)),
        }
    }
    
    pub async fn start_processor(&self) {
        let rx = self.alert_rx.clone();
        let alerts = self.alerts.clone();
        let groups = self.groups.clone();
        
        tokio::spawn(async move {
            let mut rx = rx.write().await;
            
            while let Some(alert) = rx.recv().await {
                // Store alert
                let mut alerts_guard = alerts.write().await;
                alerts_guard.push(alert.clone());
                
                // Keep only last 5000 alerts
                if alerts_guard.len() > 5000 {
                    alerts_guard.remove(0);
                }
                drop(alerts_guard);
                
                // Group similar alerts
                let mut groups_guard = groups.write().await;
                let correlation_id = alert.correlation_id.clone().unwrap_or_else(|| {
                    format!("{}:{:?}", alert.title, alert.source)
                });
                
                let group = groups_guard.entry(correlation_id.clone()).or_insert(AlertGroup {
                    correlation_id: correlation_id.clone(),
                    alerts: Vec::new(),
                    first_occurrence: alert.timestamp,
                    last_occurrence: alert.timestamp,
                    count: 0,
                });
                
                group.alerts.push(alert.clone());
                group.last_occurrence = alert.timestamp;
                group.count += 1;
                
                // Log based on severity
                match alert.severity {
                    AlertSeverity::Info => info!("🔔 Alert: {} - {}", alert.title, alert.message),
                    AlertSeverity::Warning => warn!("⚠️ Alert: {} - {}", alert.title, alert.message),
                    AlertSeverity::Error => error!("❌ Alert: {} - {}", alert.title, alert.message),
                    AlertSeverity::Critical => {
                        error!("🚨 CRITICAL Alert: {} - {}", alert.title, alert.message);
                    }
                }
            }
        });
    }
    
    pub async fn create_alert(
        &self,
        severity: AlertSeverity,
        source: GodName,
        title: String,
        message: String,
    ) -> String {
        self.create_alert_advanced(
            severity,
            AlertCategory::System,
            source,
            title,
            message,
            None,
            None,
        ).await
    }
    
    pub async fn create_alert_advanced(
        &self,
        severity: AlertSeverity,
        category: AlertCategory,
        source: GodName,
        title: String,
        message: String,
        correlation_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> String {
        let alert_id = uuid::Uuid::new_v4().to_string();
        
        let alert = Alert {
            id: alert_id.clone(),
            timestamp: chrono::Utc::now(),
            severity,
            category,
            source,
            title,
            message,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
            resolution_note: None,
            correlation_id,
            metadata: metadata.unwrap_or_default(),
        };
        
        // Send to processor
        let _ = self.alert_tx.send(alert).await;
        
        alert_id
    }
    
    pub async fn acknowledge_alert(&self, alert_id: &str, by: &str) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(by.to_string());
            alert.acknowledged_at = Some(chrono::Utc::now());
            
            info!("✅ Alert {} acknowledged by {}", alert_id, by);
            Ok(())
        } else {
            Err(format!("Alert {} not found", alert_id))
        }
    }
    
    pub async fn resolve_alert(&self, alert_id: &str, resolution_note: Option<String>) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(chrono::Utc::now());
            alert.resolution_note = resolution_note;
            
            info!("✓ Alert {} resolved", alert_id);
            Ok(())
        } else {
            Err(format!("Alert {} not found", alert_id))
        }
    }
    
    pub async fn get_active_alerts(&self, severity: Option<AlertSeverity>) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        
        alerts
            .iter()
            .filter(|a| !a.resolved && !a.acknowledged)
            .filter(|a| severity.as_ref().map(|s| &a.severity == s).unwrap_or(true))
            .cloned()
            .collect()
    }
    
    pub async fn get_alert_history(
        &self,
        limit: usize,
        severity: Option<AlertSeverity>,
        source: Option<GodName>,
    ) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        
        let filtered: Vec<_> = alerts
            .iter()
            .filter(|a| severity.as_ref().map(|s| &a.severity == s).unwrap_or(true))
            .filter(|a| source.as_ref().map(|s| &a.source == s).unwrap_or(true))
            .cloned()
            .collect();
        
        filtered.into_iter().rev().take(limit).collect()
    }
    
    pub async fn critical_count(&self) -> usize {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|a| !a.acknowledged && !a.resolved && a.severity == AlertSeverity::Critical)
            .count()
    }
    
    pub async fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }
    
    pub async fn get_rules(&self) -> Vec<AlertRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }
    
    pub async fn enable_rule(&self, rule_id: &str, enabled: bool) {
        let mut rules = self.rules.write().await;
        
        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
        }
    }
    
    pub async fn get_alert_stats(&self) -> AlertStats {
        let alerts = self.alerts.read().await;
        
        let total = alerts.len();
        let active = alerts.iter().filter(|a| !a.resolved).count();
        let acknowledged = alerts.iter().filter(|a| a.acknowledged).count();
        let resolved = alerts.iter().filter(|a| a.resolved).count();
        
        let critical_active = alerts
            .iter()
            .filter(|a| !a.resolved && a.severity == AlertSeverity::Critical)
            .count();
        
        let error_active = alerts
            .iter()
            .filter(|a| !a.resolved && a.severity == AlertSeverity::Error)
            .count();
        
        let warning_active = alerts
            .iter()
            .filter(|a| !a.resolved && a.severity == AlertSeverity::Warning)
            .count();
        
        AlertStats {
            total,
            active,
            acknowledged,
            resolved,
            critical_active,
            error_active,
            warning_active,
        }
    }
    
    pub async fn get_alert_groups(&self, correlation_id: Option<String>) -> Vec<AlertGroup> {
        let groups = self.groups.read().await;
        
        match correlation_id {
            Some(id) => groups
                .get(&id)
                .cloned()
                .map(|g| vec![g])
                .unwrap_or_default(),
            None => groups.values().cloned().collect(),
        }
    }
    
    pub async fn cleanup_old_alerts(&self, max_age: Duration) {
        let mut alerts = self.alerts.write().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(max_age).unwrap_or_default();
        
        alerts.retain(|alert| {
            alert.timestamp > cutoff || (!alert.resolved && !alert.acknowledged)
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total: usize,
    pub active: usize,
    pub acknowledged: usize,
    pub resolved: usize,
    pub critical_active: usize,
    pub error_active: usize,
    pub warning_active: usize,
}
