/// Hefesto v12 - Diosa del Hogar y el Fuego
/// Gestión de la configuración centralizada

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationItem {
    pub key: String,
    pub value: serde_json::Value,
    pub category: ConfigCategory,
    pub data_type: ConfigDataType,
    pub is_sensitive: bool,
    pub last_modified: DateTime<Utc>,
    pub modified_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigCategory {
    System,
    Database,
    Security,
    Clinical,
    UI,
    Performance,
    Networking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigDataType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Encrypted,
}

#[derive(Debug, Clone)]
pub struct HefestoV12 {
    configuration: HashMap<String, ConfigurationItem>,
    audit_trail: Vec<ConfigurationChange>,
    backup_enabled: bool,
    auto_save_enabled: bool,
    validation_rules: HashMap<String, ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChange {
    pub config_key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub changed_by: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub validation_pattern: String,
    pub error_message: String,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Email,
    PhoneNumber,
    URL,
    NumericRange,
    StringLength,
    JSONSchema,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String,
    pub uptime: chrono::Duration,
    pub active_processes: u32,
    pub resource_usage: ResourceUsage,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub disk_usage: u64,
    pub network_usage: NetworkStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u32,
    pub connections_total: u64,
}

impl HefestoV12 {
    pub fn new() -> Self {
        Self {
            configuration: Self::load_default_configuration(),
            audit_trail: Vec::new(),
            backup_enabled: true,
            auto_save_enabled: true,
            validation_rules: Self::create_default_validation_rules(),
        }
    }

    fn load_default_configuration() -> HashMap<String, ConfigurationItem> {
        let mut config = HashMap::new();
        
        // Configuración del sistema
        config.insert("system.name".to_string(), ConfigurationItem {
            key: "system.name".to_string(),
            value: serde_json::Value::String("Olympus v12 Clinical Intelligence System".to_string()),
            category: ConfigCategory::System,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("system.version".to_string(), ConfigurationItem {
            key: "system.version".to_string(),
            value: serde_json::Value::String("12.0.0".to_string()),
            category: ConfigCategory::System,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("system.environment".to_string(), ConfigurationItem {
            key: "system.environment".to_string(),
            value: serde_json::Value::String("production".to_string()),
            category: ConfigCategory::System,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        // Configuración de la base de datos
        config.insert("database.url".to_string(), ConfigurationItem {
            key: "database.url".to_string(),
            value: serde_json::Value::String("memory://olympus_v12".to_string()),
            category: ConfigCategory::Database,
            data_type: ConfigDataType::Encrypted,
            is_sensitive: true,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("database.namespace".to_string(), ConfigurationItem {
            key: "database.namespace".to_string(),
            value: serde_json::Value::String("uci".to_string()),
            category: ConfigCategory::Database,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        // Configuración de seguridad
        config.insert("security.jwt_secret".to_string(), ConfigurationItem {
            key: "security.jwt_secret".to_string(),
            value: serde_json::Value::String("change_in_production_default".to_string()),
            category: ConfigCategory::Security,
            data_type: ConfigDataType::Encrypted,
            is_sensitive: true,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("security.session_timeout".to_string(), ConfigurationItem {
            key: "security.session_timeout".to_string(),
            value: serde_json::Value::Number(24.0),
            category: ConfigCategory::Security,
            data_type: ConfigDataType::Number,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("security.max_login_attempts".to_string(), ConfigurationItem {
            key: "security.max_login_attempts".to_string(),
            value: serde_json::Value::Number(5.0),
            category: ConfigCategory::Security,
            data_type: ConfigDataType::Number,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        // Configuración clínica
        config.insert("clinical.default_language".to_string(), ConfigurationItem {
            key: "clinical.default_language".to_string(),
            value: serde_json::Value::String("es".to_string()),
            category: ConfigCategory::Clinical,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("clinical.assessment_intervals".to_string(), ConfigurationItem {
            key: "clinical.assessment_intervals".to_string(),
            value: serde_json::json!({
                "glasgow": 4,
                "apache": 24,
                "sofa": 12,
                "saps": 24,
                "news2": 1
            }),
            category: ConfigCategory::Clinical,
            data_type: ConfigDataType::Object,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("clinical.risk_thresholds".to_string(), ConfigurationItem {
            key: "clinical.risk_thresholds".to_string(),
            value: serde_json::json!({
                "low": 0.3,
                "moderate": 0.6,
                "high": 0.8,
                "critical": 0.9
            }),
            category: ConfigCategory::Clinical,
            data_type: ConfigDataType::Object,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        // Configuración de UI
        config.insert("ui.theme".to_string(), ConfigurationItem {
            key: "ui.theme".to_string(),
            value: serde_json::Value::String("light".to_string()),
            category: ConfigCategory::UI,
            data_type: ConfigDataType::String,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("ui.animations".to_string(), ConfigurationItem {
            key: "ui.animations".to_string(),
            value: serde_json::Value::Boolean(true),
            category: ConfigCategory::UI,
            data_type: ConfigDataType::Boolean,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("ui.auto_refresh_interval".to_string(), ConfigurationItem {
            key: "ui.auto_refresh_interval".to_string(),
            value: serde_json::Value::Number(30.0),
            category: ConfigCategory::UI,
            data_type: ConfigDataType::Number,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        // Configuración de rendimiento
        config.insert("performance.max_concurrent_users".to_string(), ConfigurationItem {
            key: "performance.max_concurrent_users".to_string(),
            value: serde_json::Value::Number(100.0),
            category: ConfigCategory::Performance,
            data_type: ConfigDataType::Number,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config.insert("performance.query_timeout".to_string(), ConfigurationItem {
            key: "performance.query_timeout".to_string(),
            value: serde_json::Value::Number(30.0),
            category: ConfigCategory::Performance,
            data_type: ConfigDataType::Number,
            is_sensitive: false,
            last_modified: Utc::now(),
            modified_by: None,
        });
        
        config
    }

    fn create_default_validation_rules() -> HashMap<String, ValidationRule> {
        let mut rules = HashMap::new();
        
        // Regla de validación de email
        rules.insert("email".to_string(), ValidationRule {
            rule_type: ValidationType::Email,
            validation_pattern: r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string(),
            error_message: "Formato de email inválido".to_string(),
            is_required: false,
        });
        
        // Regla de validación de teléfono
        rules.insert("phone".to_string(), ValidationRule {
            rule_type: ValidationType::PhoneNumber,
            validation_pattern: r"^\+?(\d{1,3})?[-.(\s]?)?\(?\d{3})\)?[-.(\s]?)?\(?\d{4,5})(?:[-.(\s]?)?\d{6,8})?)$".to_string(),
            error_message: "Formato de teléfono inválido".to_string(),
            is_required: false,
        });
        
        // Regla de validación de URL
        rules.insert("url".to_string(), ValidationRule {
            rule_type: ValidationType::URL,
            validation_pattern: r"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()]{1,6}\b)*([/?#][\w-])*@?$".to_string(),
            error_message: "URL inválida".to_string(),
            is_required: false,
        });
        
        rules
    }

    pub fn get_config(&self, key: &str) -> Option<&ConfigurationItem> {
        self.configuration.get(key)
    }

    pub async fn update_config(&mut self, key: &str, value: serde_json::Value, modified_by: &str, reason: &str) -> Result<(), String> {
        if let Some(old_item) = self.configuration.get(key) {
            // Registrar cambio en audit trail
            let change = ConfigurationChange {
                config_key: key.to_string(),
                old_value: old_item.value.clone(),
                new_value: value.clone(),
                changed_by: modified_by.to_string(),
                timestamp: Utc::now(),
                reason: reason.to_string(),
            };
            
            self.audit_trail.push(change);
            
            // Limitar audit trail
            if self.audit_trail.len() > 10000 {
                self.audit_trail.drain(0..5000);
            }
        }

        let category = self.configuration.get(key)
            .map(|item| item.category.clone())
            .unwrap_or(ConfigCategory::System);
        
        let new_item = ConfigurationItem {
            key: key.to_string(),
            value: value.clone(),
            category,
            data_type: Self::determine_data_type(&value),
            is_sensitive: self.is_sensitive_key(key),
            last_modified: Utc::now(),
            modified_by: Some(modified_by.to_string()),
        };

        self.configuration.insert(key.to_string(), new_item);
        
        // Auto-save si está habilitado
        if self.auto_save_enabled {
            tracing::info!("🏛️ Hefesto: Auto-guardando configuración para {}", key);
            // En v13 esto persistiría en base de datos
        }

        tracing::info!("🏛️ Hefesto: Configuración actualizada - {} por {} - {}", key, modified_by, reason);
        Ok(())
    }

    pub async fn update_multiple_configs(&mut self, updates: Vec<(String, serde_json::Value, String, String)>, modified_by: &str) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();
        let mut successful_updates = Vec::new();
        
        for (key, value, reason) in updates {
            if let Err(e) = self.update_config(key, value, modified_by, &reason).await {
                errors.push(format!("Error actualizando {}: {}", key, e));
            } else {
                successful_updates.push(key.to_string());
            }
        }

        if successful_updates.len() > 0 {
            tracing::info!("🏛️ Hefesto: {} configuraciones actualizadas exitosamente", successful_updates.len());
        }

        if errors.len() > 0 {
            tracing::warn!("🏛️ Hefesto: {} errores en actualización", errors.len());
        }

        Ok(successful_updates)
    }

    pub fn delete_config(&mut self, key: &str, modified_by: &str, reason: &str) -> Result<(), String> {
        if let Some(old_item) = self.configuration.remove(key) {
            // Registrar eliminación en audit trail
            let change = ConfigurationChange {
                config_key: key.to_string(),
                old_value: old_item.value,
                new_value: serde_json::Value::Null,
                changed_by: modified_by.to_string(),
                timestamp: Utc::now(),
                reason: reason.to_string(),
            };
            
            self.audit_trail.push(change);
            
            tracing::info!("🏛️ Hefesto: Configuración {} eliminada por {}", key, modified_by);
            Ok(())
        } else {
            Err(format!("Configuración {} no encontrada", key))
        }
    }

    pub fn validate_value(&self, key: &str, value: &serde_json::Value) -> Result<(), String> {
        if let Some(rule) = self.validation_rules.get(key) {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            
            // Verificar si es requerido y está presente
            if rule.is_required && value_str.trim().is_empty() {
                return Err(format!("Valor requerido para {}", key));
            }
            
            // Aplicar validación con regex si existe
            if !rule.validation_pattern.is_empty() {
                let re = regex::Regex::new(&rule.validation_pattern)
                    .map_err(|e| format!("Error en patrón de validación: {}", e))?;
                
                if !re.is_match(&value_str) {
                    return Err(rule.error_message);
                }
            }
        }
        
        Ok(())
    }

    pub fn get_config_by_category(&self, category: ConfigCategory) -> Vec<&ConfigurationItem> {
        self.configuration
            .values()
            .filter(|item| item.category == category)
            .collect()
    }

    pub fn search_config(&self, query: &str) -> Vec<&ConfigurationItem> {
        self.configuration
            .values()
            .filter(|item| {
                item.key.contains(query) || 
                serde_json::to_string(&item.value).unwrap_or_default().contains(query) ||
                serde_json::to_string(&item.key).unwrap_or_default().contains(query)
            })
            .collect()
    }

    pub fn backup_configuration(&self) -> String {
        if !self.backup_enabled {
            return "Backup no habilitado".to_string();
        }

        let backup_data = serde_json::json!({
            "timestamp": Utc::now(),
            "configuration": self.configuration,
            "audit_trail_sample": self.audit_trail.iter().take(100).collect::<Vec<_>>(),
            "version": "12.0.0",
        });

        let backup_json = serde_json::to_string_pretty(&backup_data).unwrap_or_default();
        
        tracing::info!("🏛️ Hefesto: Backup de configuración generado");
        backup_json
    }

    pub fn restore_configuration(&mut self, backup_data: &str, modified_by: &str) -> Result<(), String> {
        let backup_config: serde_json::Value = serde_json::from_str(backup_data)
            .map_err(|e| format!("Error parseando backup: {}", e))?;
        
        if let Some(config_obj) = backup_config.as_object() {
            if let Some(new_config) = config_obj.get("configuration").and_then(|v| v.as_object()) {
                let new_map: HashMap<String, serde_json::Value> = new_config
                    .into_iter()
                    .filter_map(|(k, v)| {
                        Some((k.clone(), v.clone()))
                    })
                    .collect();
                
                let old_configuration = self.configuration.clone();
                
                // Migrar cada item con validación
                for (key, value) in new_map {
                    if let Err(e) = self.validate_value(key, &value) {
                        tracing::warn!("🏛️ Hefesto: Validación fallida para {}: {}", key, e);
                        continue;
                    }
                    
                    if let Some(old_item) = old_configuration.get(key) {
                        let change = ConfigurationChange {
                            config_key: key.clone(),
                            old_value: old_item.value.clone(),
                            new_value: value.clone(),
                            changed_by: modified_by.to_string(),
                            timestamp: Utc::now(),
                            reason: "Restauración desde backup".to_string(),
                        };
                        
                        self.audit_trail.push(change);
                    }
                    
                    let category = old_item.category.clone();
                    let new_item = ConfigurationItem {
                        key: key.clone(),
                        value,
                        category,
                        data_type: Self::determine_data_type(&value),
                        is_sensitive: self.is_sensitive_key(key),
                        last_modified: Utc::now(),
                        modified_by: Some(modified_by.to_string()),
                    };
                    
                    self.configuration.insert(key, new_item);
                }
                
                tracing::info!("🏛️ Hefesto: Configuración restaurada exitosamente con {} items", new_map.len());
                return Ok(());
            }
        }
        
        Err("Formato de backup inválido".to_string())
    }

    pub fn get_system_status(&self) -> SystemStatus {
        // En una implementación real, esto obtendría métricas del sistema
        SystemStatus {
            status: "operational".to_string(),
            uptime: chrono::Duration::hours(24), // Simulado
            active_processes: 20,
            resource_usage: ResourceUsage {
                cpu_usage: 45.2,
                memory_usage: 512 * 1024 * 1024, // 512MB
                disk_usage: 2 * 1024 * 1024 * 1024, // 2GB
                network_usage: NetworkStats {
                    bytes_sent: 1024 * 1024,
                    bytes_received: 2 * 1024 * 1024,
                    connections_active: 15,
                    connections_total: 1000,
                },
            },
            last_update: Utc::now(),
        }
    }

    pub fn get_audit_trail(&self, limit: Option<usize>) -> Vec<&ConfigurationChange> {
        let mut trail = self.audit_trail.clone();
        
        // Ordenar por timestamp (más reciente primero)
        trail.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            trail.truncate(limit);
        }
        
        trail.iter().collect()
    }

    pub fn clear_audit_trail(&mut self, older_than_hours: u64) {
        let cutoff_time = Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        let initial_count = self.audit_trail.len();
        
        self.audit_trail.retain(|change| change.timestamp > cutoff_time);
        
        let removed_count = initial_count - self.audit_trail.len();
        if removed_count > 0 {
            tracing::info!("🏛️ Hefesto: {} entradas de audit limpiadas", removed_count);
        }
    }

    fn determine_data_type(value: &serde_json::Value) -> ConfigDataType {
        match value {
            serde_json::Value::String(_) => ConfigDataType::String,
            serde_json::Value::Number(_) => ConfigDataType::Number,
            serde_json::Value::Bool(_) => ConfigDataType::Boolean,
            serde_json::Value::Array(_) => ConfigDataType::Array,
            serde_json::Value::Object(_) => ConfigDataType::Object,
            _ => ConfigDataType::String,
        }
    }

    fn is_sensitive_key(&self, key: &str) -> bool {
        key.contains("password") || 
        key.contains("secret") || 
        key.contains("key") || 
        key.contains("token") ||
        key.contains("jwt")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HefestoStatus {
    pub total_configurations: usize,
    pub audit_trail_size: usize,
    pub backup_enabled: bool,
    pub auto_save_enabled: bool,
    pub last_update: DateTime<Utc>,
}

impl Default for HefestoV12 {
    fn default() -> Self {
        Self::new()
    }
}