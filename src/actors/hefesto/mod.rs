// src/actors/hefesto/mod.rs
// OLYMPUS v15 - Hefesto: Dios de la Forja y Configuración
// Responsabilidad: Gestión de configuraciones, validación, backups y migraciones

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use chrono::{DateTime, Utc};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

// Submódulos
pub mod config;
pub mod validation;
pub mod backup;
pub mod migration;

pub use config::{ConfigManager, ConfigEntry};
pub use validation::{SchemaValidator, ValidationResult};
pub use backup::{BackupManager, Backup, BackupType};
pub use migration::MigrationManager;

/// Hefesto - Dios de la Configuración
/// Gestiona configuraciones del sistema, validación, backups y migraciones
#[derive(Debug)]
pub struct Hefesto {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Manager de configuraciones
    config_manager: Arc<RwLock<ConfigManager>>,
    /// Validador de schemas
    validator: Arc<RwLock<SchemaValidator>>,
    /// Manager de backups
    backup_manager: Arc<RwLock<BackupManager>>,
    /// Manager de migraciones
    migration_manager: Arc<RwLock<MigrationManager>>,
}

impl Hefesto {
    pub async fn new() -> Self {
        info!("🔥 Hefesto: Inicializando sistema de configuración...");
        
        Self {
            name: GodName::Hefesto,
            state: ActorState::new(GodName::Hefesto),
            config: ActorConfig::default(),
            config_manager: Arc::new(RwLock::new(ConfigManager::new())),
            validator: Arc::new(RwLock::new(SchemaValidator::new())),
            backup_manager: Arc::new(RwLock::new(BackupManager::new())),
            migration_manager: Arc::new(RwLock::new(MigrationManager::new())),
        }
    }

    /// Obtiene una configuración
    pub async fn get_config(&self, key: &str) -> Option<ConfigEntry> {
        let manager = self.config_manager.read().await;
        manager.get(key)
    }

    /// Establece una configuración
    pub async fn set_config(&self, key: &str, value: serde_json::Value, description: Option<String>) -> Result<(), ActorError> {
        // Validar primero
        let validator = self.validator.read().await;
        let validation = validator.validate(key, &value)?;
        
        if !validation.is_valid {
            return Err(ActorError::InvalidCommand {
                god: GodName::Hefesto,
                reason: format!("Validación fallida: {:?}", validation.errors),
            });
        }
        
        drop(validator);
        
        // Establecer configuración
        let mut manager = self.config_manager.write().await;
        manager.set(key, value, description);
        
        info!("🔥 Hefesto: Configuración '{}' actualizada", key);
        Ok(())
    }

    /// Elimina una configuración
    pub async fn delete_config(&self, key: &str) -> Result<(), ActorError> {
        let mut manager = self.config_manager.write().await;
        manager.delete(key)
            .ok_or_else(|| ActorError::NotFound { god: GodName::Hefesto })?;
        
        info!("🔥 Hefesto: Configuración '{}' eliminada", key);
        Ok(())
    }

    /// Lista todas las configuraciones
    pub async fn list_configs(&self) -> Vec<ConfigEntry> {
        let manager = self.config_manager.read().await;
        manager.list_all()
    }

    /// Crea un backup de las configuraciones
    pub async fn create_backup(&self, backup_type: BackupType) -> Result<Backup, ActorError> {
        let configs = self.list_configs().await;
        let mut backup_manager = self.backup_manager.write().await;
        
        let backup = backup_manager.create_backup(configs, backup_type)?;
        
        info!("🔥 Hefesto: Backup '{}' creado", backup.id);
        Ok(backup)
    }

    /// Restaura un backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<(), ActorError> {
        let backup_manager = self.backup_manager.read().await;
        let backup = backup_manager.get_backup(backup_id)
            .ok_or_else(|| ActorError::NotFound { god: GodName::Hefesto })?;
        
        // Restaurar configuraciones
        let mut config_manager = self.config_manager.write().await;
        for entry in &backup.configurations {
            config_manager.set(&entry.key, entry.value.clone(), entry.description.clone());
        }
        
        info!("🔥 Hefesto: Backup '{}' restaurado", backup_id);
        Ok(())
    }

    /// Lista backups disponibles
    pub async fn list_backups(&self) -> Vec<Backup> {
        let manager = self.backup_manager.read().await;
        manager.list_backups()
    }

    /// Ejecuta una migración
    pub async fn run_migration(&self, migration_id: &str) -> Result<(), ActorError> {
        let mut manager = self.migration_manager.write().await;
        manager.run_migration(migration_id).await
    }

    /// Valida una configuración contra su schema
    pub async fn validate_config(&self, key: &str, value: &serde_json::Value) -> ValidationResult {
        let validator = self.validator.read().await;
        validator.validate(key, value).unwrap_or_else(|_| ValidationResult::invalid())
    }

    /// Obtiene estadísticas del sistema
    pub async fn get_statistics(&self) -> HefestoStatistics {
        let configs = self.config_manager.read().await;
        let backups = self.backup_manager.read().await;
        let migrations = self.migration_manager.read().await;
        
        HefestoStatistics {
            total_configs: configs.count(),
            total_backups: backups.count(),
            pending_migrations: migrations.pending_count(),
            last_backup: backups.last_backup_date(),
        }
    }
}

/// Estadísticas de Hefesto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HefestoStatistics {
    pub total_configs: usize,
    pub total_backups: usize,
    pub pending_migrations: usize,
    pub last_backup: Option<DateTime<Utc>>,
}

#[async_trait]
impl OlympianActor for Hefesto {
    fn name(&self) -> GodName { 
        GodName::Hefesto 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::Configuration 
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = Utc::now();

        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            _ => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }

    async fn persistent_state(&self) -> serde_json::Value {
        let stats = self.get_statistics().await;
        serde_json::json!({
            "name": "Hefesto",
            "messages": self.state.message_count,
            "configs": stats.total_configs,
            "backups": stats.total_backups,
            "status": self.state.status,
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }

    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name.clone(),
            status: self.state.status.clone(),
            last_seen: Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: 0,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🔥 Hefesto: Sistema de configuración v15 iniciado");
        info!("🔥 Hefesto: Gestión de configs, validación, backups y migraciones activos");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🔥 Hefesto: Sistema de configuración detenido");
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados
impl Hefesto {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("set_config") => {
                        let key = data.get("key")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Hefesto,
                                reason: "key requerido".to_string(),
                            })?;
                        
                        let value = data.get("value")
                            .cloned()
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Hefesto,
                                reason: "value requerido".to_string(),
                            })?;
                        
                        let description = data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        self.set_config(key, value, description).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Configuración '{}' establecida", key) 
                        })
                    }
                    Some("delete_config") => {
                        let key = data.get("key")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Hefesto,
                                reason: "key requerido".to_string(),
                            })?;
                        
                        self.delete_config(key).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Configuración '{}' eliminada", key) 
                        })
                    }
                    Some("create_backup") => {
                        let backup_type = data.get("type")
                            .and_then(|v| serde_json::from_value::<BackupType>(v.clone()).ok())
                            .unwrap_or(BackupType::Manual);
                        
                        let backup = self.create_backup(backup_type).await?;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "backup_id": backup.id,
                                "created_at": backup.created_at,
                            })
                        })
                    }
                    Some("restore_backup") => {
                        let backup_id = data.get("backup_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Hefesto,
                                reason: "backup_id requerido".to_string(),
                            })?;
                        
                        self.restore_backup(backup_id).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Backup '{}' restaurado", backup_id) 
                        })
                    }
                    Some("run_migration") => {
                        let migration_id = data.get("migration_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Hefesto,
                                reason: "migration_id requerido".to_string(),
                            })?;
                        
                        self.run_migration(migration_id).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Migración '{}' ejecutada", migration_id) 
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Hefesto, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Hefesto, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let stats = self.get_statistics().await;
                Ok(ResponsePayload::Stats { 
                    data: serde_json::to_value(&stats).unwrap_or_default()
                })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                match query_type {
                    "get_config" => {
                        let key = data.get("key")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Hefesto,
                                reason: "key requerido".to_string(),
                            })?;
                        
                        if let Some(config) = self.get_config(key).await {
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&config).unwrap_or_default()
                            })
                        } else {
                            Err(ActorError::NotFound { god: GodName::Hefesto })
                        }
                    }
                    "list_configs" => {
                        let configs = self.list_configs().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "configs": configs,
                                "count": configs.len(),
                            })
                        })
                    }
                    "list_backups" => {
                        let backups = self.list_backups().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "backups": backups,
                                "count": backups.len(),
                            })
                        })
                    }
                    "validate_config" => {
                        let key = data.get("key")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Hefesto,
                                reason: "key requerido".to_string(),
                            })?;
                        
                        let value = data.get("value")
                            .cloned()
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Hefesto,
                                reason: "value requerido".to_string(),
                            })?;
                        
                        let result = self.validate_config(key, &value).await;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&result).unwrap_or_default()
                        })
                    }
                    "statistics" => {
                        let stats = self.get_statistics().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&stats).unwrap_or_default()
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Hefesto, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Hefesto, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hefesto_initialization() -> Result<(), ActorError> {
        let mut hefesto = Hefesto::new().await;
        hefesto.initialize().await?;
        
        assert_eq!(hefesto.name(), GodName::Hefesto);
        assert_eq!(hefesto.domain(), DivineDomain::Configuration);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_config_management() -> Result<(), ActorError> {
        let hefesto = Hefesto::new().await;
        
        // Establecer configuración
        hefesto.set_config("test.key", serde_json::json!("value"), Some("Test config".to_string())).await?;
        
        // Obtener configuración
        let config = hefesto.get_config("test.key").await;
        assert!(config.is_some());
        
        // Listar configuraciones
        let configs = hefesto.list_configs().await;
        assert!(!configs.is_empty());
        
        // Eliminar configuración
        hefesto.delete_config("test.key").await?;
        let config = hefesto.get_config("test.key").await;
        assert!(config.is_none());
        
        Ok(())
    }
}
