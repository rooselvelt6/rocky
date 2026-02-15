// src/actors/hades/audit.rs
// OLYMPUS v15 - Hades Audit Logger
// Sistema completo de auditoría HIPAA-compliant

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, debug};
use chrono::{Utc, Timelike};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub hipaa_relevant: bool,
    pub phi_accessed: Option<Vec<String>>, // Protected Health Information fields accessed
    pub sensitivity: DataSensitivity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Partial,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataSensitivity {
    Public,           // No restrictions
    Internal,         // Internal use only
    Confidential,     // Restricted access
    PHI,              // Protected Health Information (HIPAA)
    Critical,         // Highly sensitive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub actor: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub result: Option<AuditResult>,
    pub hipaa_only: bool,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct AuditLogger {
    logs: Arc<RwLock<Vec<AuditEntry>>>,
    index_actor: Arc<RwLock<HashMap<String, Vec<usize>>>>, // Index by actor
    index_action: Arc<RwLock<HashMap<String, Vec<usize>>>>, // Index by action
    index_resource: Arc<RwLock<HashMap<String, Vec<usize>>>>, // Index by resource
    hipaa_entries: Arc<RwLock<Vec<usize>>>, // Index of HIPAA-relevant entries
    max_entries: usize,
    export_enabled: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            index_actor: Arc::new(RwLock::new(HashMap::new())),
            index_action: Arc::new(RwLock::new(HashMap::new())),
            index_resource: Arc::new(RwLock::new(HashMap::new())),
            hipaa_entries: Arc::new(RwLock::new(Vec::new())),
            max_entries: 100000, // Keep last 100k entries in memory
            export_enabled: true,
        }
    }
    
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            index_actor: Arc::new(RwLock::new(HashMap::new())),
            index_action: Arc::new(RwLock::new(HashMap::new())),
            index_resource: Arc::new(RwLock::new(HashMap::new())),
            hipaa_entries: Arc::new(RwLock::new(Vec::new())),
            max_entries,
            export_enabled: true,
        }
    }
    
    /// Log an audit event
    pub async fn log(
        &self,
        action: &str,
        actor: &str,
        resource: &str,
        result: AuditResult,
        details: serde_json::Value,
    ) {
        self.log_advanced(
            action,
            actor,
            resource,
            result,
            details,
            None, // ip_address
            None, // user_agent
            None, // session_id
            false, // hipaa_relevant
            None, // phi_accessed
            DataSensitivity::Internal,
        ).await;
    }
    
    /// Log an audit event with all details
    pub async fn log_advanced(
        &self,
        action: &str,
        actor: &str,
        resource: &str,
        result: AuditResult,
        details: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        hipaa_relevant: bool,
        phi_accessed: Option<Vec<String>>,
        sensitivity: DataSensitivity,
    ) {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action: action.to_string(),
            actor: actor.to_string(),
            resource: resource.to_string(),
            result: result.clone(),
            details,
            ip_address,
            user_agent,
            session_id,
            hipaa_relevant,
            phi_accessed,
            sensitivity,
        };
        
        // Store entry
        let mut logs = self.logs.write().await;
        let index = logs.len();
        logs.push(entry);
        
        // Keep only max_entries
        let removed_count = if logs.len() > self.max_entries {
            let to_remove = logs.len() - self.max_entries;
            logs.drain(0..to_remove);
            to_remove
        } else {
            0
        };
        drop(logs);
        
        // Update indices (only if we didn't remove entries)
        if removed_count == 0 {
            let mut index_actor = self.index_actor.write().await;
            index_actor.entry(actor.to_string()).or_insert_with(Vec::new).push(index);
            drop(index_actor);
            
            let mut index_action = self.index_action.write().await;
            index_action.entry(action.to_string()).or_insert_with(Vec::new).push(index);
            drop(index_action);
            
            let mut index_resource = self.index_resource.write().await;
            index_resource.entry(resource.to_string()).or_insert_with(Vec::new).push(index);
            drop(index_resource);
            
            if hipaa_relevant {
                let mut hipaa_entries = self.hipaa_entries.write().await;
                hipaa_entries.push(index);
                drop(hipaa_entries);
            }
        }
        
        // Log based on sensitivity
        match result {
            AuditResult::Success => {
                if hipaa_relevant {
                    info!("📝 HIPAA Audit: {} by {} on {} - SUCCESS", action, actor, resource);
                } else {
                    debug!("📝 Audit: {} by {} on {} - SUCCESS", action, actor, resource);
                }
            }
            AuditResult::Failure => {
                warn!("📝 Audit: {} by {} on {} - FAILURE", action, actor, resource);
            }
            AuditResult::Denied => {
                warn!("📝 Audit: {} by {} on {} - DENIED", action, actor, resource);
            }
            AuditResult::Partial => {
                info!("📝 Audit: {} by {} on {} - PARTIAL", action, actor, resource);
            }
        }
    }
    
    /// Query audit logs
    pub async fn query(&self, query: AuditQuery) -> Vec<AuditEntry> {
        let logs = self.logs.read().await;
        
        let filtered: Vec<_> = logs
            .iter()
            .filter(|entry| {
                // Check time range
                if let Some(start) = query.start_time {
                    if entry.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if entry.timestamp > end {
                        return false;
                    }
                }
                
                // Check actor
                if let Some(ref actor) = query.actor {
                    if &entry.actor != actor {
                        return false;
                    }
                }
                
                // Check action
                if let Some(ref action) = query.action {
                    if &entry.action != action {
                        return false;
                    }
                }
                
                // Check resource
                if let Some(ref resource) = query.resource {
                    if &entry.resource != resource {
                        return false;
                    }
                }
                
                // Check result
                if let Some(ref result) = query.result {
                    if &entry.result != result {
                        return false;
                    }
                }
                
                // Check HIPAA only
                if query.hipaa_only && !entry.hipaa_relevant {
                    return false;
                }
                
                true
            })
            .cloned()
            .collect();
        
        // Return last N entries (most recent first)
        filtered.into_iter().rev().take(query.limit).collect()
    }
    
    /// Get logs by actor
    pub async fn get_logs_by_actor(&self, actor: &str, limit: usize) -> Vec<AuditEntry> {
        let index_actor = self.index_actor.read().await;
        
        if let Some(indices) = index_actor.get(actor) {
            let logs = self.logs.read().await;
            indices
                .iter()
                .rev()
                .take(limit)
                .filter_map(|&idx| logs.get(idx).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get logs by action type
    pub async fn get_logs_by_action(&self, action: &str, limit: usize) -> Vec<AuditEntry> {
        let index_action = self.index_action.read().await;
        
        if let Some(indices) = index_action.get(action) {
            let logs = self.logs.read().await;
            indices
                .iter()
                .rev()
                .take(limit)
                .filter_map(|&idx| logs.get(idx).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get HIPAA-relevant logs
    pub async fn get_hipaa_logs(&self, limit: usize) -> Vec<AuditEntry> {
        let hipaa_entries = self.hipaa_entries.read().await;
        let logs = self.logs.read().await;
        
        hipaa_entries
            .iter()
            .rev()
            .take(limit)
            .filter_map(|&idx| logs.get(idx).cloned())
            .collect()
    }
    
    /// Get audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        let logs = self.logs.read().await;
        let total = logs.len();
        
        let successful = logs.iter().filter(|e| e.result == AuditResult::Success).count();
        let failed = logs.iter().filter(|e| e.result == AuditResult::Failure).count();
        let denied = logs.iter().filter(|e| e.result == AuditResult::Denied).count();
        let hipaa_count = logs.iter().filter(|e| e.hipaa_relevant).count();
        
        // Count unique actors
        let unique_actors: std::collections::HashSet<_> = logs.iter().map(|e| &e.actor).collect();
        
        // Get last 24h stats
        let last_24h = Utc::now() - chrono::Duration::hours(24);
        let recent_count = logs.iter().filter(|e| e.timestamp > last_24h).count();
        
        AuditStats {
            total_entries: total,
            successful_actions: successful,
            failed_actions: failed,
            denied_actions: denied,
            hipaa_entries: hipaa_count,
            unique_actors: unique_actors.len(),
            entries_last_24h: recent_count,
        }
    }
    
    /// Export logs to various formats
    pub async fn export(&self, format: ExportFormat, query: Option<AuditQuery>) -> Result<String, AuditError> {
        let entries = match query {
            Some(q) => self.query(q).await,
            None => {
                let logs = self.logs.read().await;
                logs.clone()
            }
        };
        
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&entries)
                    .map_err(|e| AuditError::ExportError(e.to_string()))
            }
            ExportFormat::Csv => {
                let mut csv = String::from("id,timestamp,action,actor,resource,result,hipaa_relevant,sensitivity\n");
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{:?},{},{:?}\n",
                        entry.id,
                        entry.timestamp,
                        entry.action,
                        entry.actor,
                        entry.resource,
                        entry.result,
                        entry.hipaa_relevant,
                        entry.sensitivity
                    ));
                }
                Ok(csv)
            }
            ExportFormat::Syslog => {
                let mut syslog = String::new();
                for entry in entries {
                    syslog.push_str(&format!(
                        "{} {} {}: {} by {} on {} - {:?}\n",
                        entry.timestamp.format("%b %d %H:%M:%S"),
                        "olympus",
                        "hades-audit",
                        entry.action,
                        entry.actor,
                        entry.resource,
                        entry.result
                    ));
                }
                Ok(syslog)
            }
        }
    }
    
    /// Purge old logs
    pub async fn purge_old_logs(&self, max_age: Duration) -> usize {
        let cutoff = Utc::now() - chrono::Duration::from_std(max_age).unwrap_or_default();
        
        let mut logs = self.logs.write().await;
        let initial_count = logs.len();
        
        logs.retain(|entry| {
            entry.timestamp > cutoff || entry.hipaa_relevant // Never delete HIPAA logs
        });
        
        let removed = initial_count - logs.len();
        
        if removed > 0 {
            info!("🧹 Purged {} old audit logs", removed);
        }
        
        removed
    }
    
    /// Detect suspicious activity
    pub async fn detect_anomalies(&self) -> Vec<AnomalyReport> {
        let logs = self.logs.read().await;
        let mut reports = Vec::new();
        
        // Check for multiple failed logins
        let last_hour = Utc::now() - chrono::Duration::hours(1);
        let mut failed_logins: HashMap<String, usize> = HashMap::new();
        
        for entry in logs.iter().filter(|e| e.timestamp > last_hour) {
            if entry.action == "LOGIN_FAILED" {
                *failed_logins.entry(entry.actor.clone()).or_insert(0) += 1;
            }
        }
        
        for (actor, count) in failed_logins {
            if count >= 5 {
                reports.push(AnomalyReport {
                    severity: AnomalySeverity::High,
                    actor: actor.clone(),
                    description: format!("{} failed login attempts in last hour", count),
                    recommended_action: "Consider locking account or requiring CAPTCHA".to_string(),
                });
            }
        }
        
        // Check for after-hours PHI access
        let after_hours = logs.iter().filter(|e| {
            let hour = e.timestamp.hour();
            e.hipaa_relevant && (hour < 7 || hour > 22)
        });
        
        for entry in after_hours.take(10) {
            reports.push(AnomalyReport {
                severity: AnomalySeverity::Medium,
                actor: entry.actor.clone(),
                description: "PHI accessed outside business hours".to_string(),
                recommended_action: "Review if access was authorized".to_string(),
            });
        }
        
        reports
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub denied_actions: usize,
    pub hipaa_entries: usize,
    pub unique_actors: usize,
    pub entries_last_24h: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Syslog,
}

#[derive(Debug, Clone)]
pub struct AnomalyReport {
    pub severity: AnomalySeverity,
    pub actor: String,
    pub description: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Export error: {0}")]
    ExportError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}
