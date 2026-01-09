use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Option<String>,
    pub action: String,
    pub target_table: String,
    pub target_id: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
}

impl AuditLog {
    pub fn new(
        action: &str,
        target_table: &str,
        target_id: &str,
        details: Option<String>,
        user_id: Option<String>,
    ) -> Self {
        Self {
            id: None,
            action: action.to_string(),
            target_table: target_table.to_string(),
            target_id: target_id.to_string(),
            details,
            timestamp: Utc::now(),
            user_id,
        }
    }
}

/// Logs an action to the 'audit_logs' table
pub async fn log_action(
    db: &Surreal<Client>,
    action: &str,
    target_table: &str,
    target_id: &str,
    details: Option<String>,
    user_id: Option<String>,
) {
    let log_entry = AuditLog::new(action, target_table, target_id, details, user_id);

    match db.create("audit_logs").content(log_entry).await {
        Ok(saved) => {
            // We cast to Option<AuditLog> to handle the 2.x return type safely
            let _: Option<AuditLog> = saved;
        }
        Err(e) => {
            // We just log the error to stderr/tracing so we don't break the main flow
            tracing::error!("FAILED TO WRITE AUDIT LOG: {}", e);
        }
    }
}
