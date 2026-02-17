// src/actors/hefesto/backup.rs
// OLYMPUS v15 - Sistema de Backups

use crate::actors::hefesto::config::ConfigEntry;
use crate::errors::ActorError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Manager de backups
#[derive(Debug, Clone)]
pub struct BackupManager {
    backups: Vec<Backup>,
}

impl BackupManager {
    pub fn new() -> Self {
        Self {
            backups: Vec::new(),
        }
    }

    pub fn create_backup(
        &mut self,
        configs: Vec<ConfigEntry>,
        backup_type: BackupType,
    ) -> Result<Backup, ActorError> {
        let backup = Backup {
            id: format!("backup_{}", Utc::now().timestamp_millis()),
            configurations: configs,
            created_at: Utc::now(),
            backup_type,
            size_bytes: 0, // Se calcularía en implementación real
        };

        self.backups.push(backup.clone());
        Ok(backup)
    }

    pub fn get_backup(&self, id: &str) -> Option<&Backup> {
        self.backups.iter().find(|b| b.id == id)
    }

    pub fn list_backups(&self) -> Vec<Backup> {
        self.backups.clone()
    }

    pub fn count(&self) -> usize {
        self.backups.len()
    }

    pub fn last_backup_date(&self) -> Option<DateTime<Utc>> {
        self.backups.last().map(|b| b.created_at)
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Backup de configuraciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub configurations: Vec<ConfigEntry>,
    pub created_at: DateTime<Utc>,
    pub backup_type: BackupType,
    pub size_bytes: u64,
}

/// Tipo de backup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    Manual,
    Scheduled,
    PreMigration,
}
