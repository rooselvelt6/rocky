use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub action: String,
    pub target_table: String,
    pub target_id: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
}

pub struct Apollo;

impl Apollo {
    pub fn new() -> Self {
        Self
    }

    pub fn write_report(&self, category: &str, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let path = format!("reports/{}/{}", category, filename);
        fs::write(path, content)?;
        println!("☀️ Apollo: Informe escrito en el santuario: {}", filename);
        Ok(())
    }

    pub async fn log_action(
        &self,
        db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
        action: &str,
        target_table: &str,
        target_id: &str,
        details: Option<String>,
        user_id: Option<String>,
    ) {
        let log_entry = AuditLog {
            action: action.to_string(),
            target_table: target_table.to_string(),
            target_id: target_id.to_string(),
            details,
            timestamp: Utc::now(),
            user_id,
        };

        if let Err(e) = db.create("audit_logs").content(log_entry).await {
            tracing::error!("☀️ Apollo: Error al escribir rastro de auditoría: {}", e);
        }
    }
}

#[async_trait]
impl GodActor for Apollo {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("☀️ Apollo: Cronista histórico activo.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
