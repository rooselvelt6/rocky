// src/actors/zeus/config.rs
// OLYMPUS v15 - Zeus Configuration
// Configuración centralizada del Olimpo con hot-reloading y validación

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, watch};
use std::sync::Arc;
use tracing::{info, warn};

/// Ambiente de ejecución
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Environment::Development),
            "staging" => Ok(Environment::Staging),
            "prod" | "production" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

/// Configuración completa de Zeus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeusConfig {
    // Identidad
    pub environment: Environment,
    pub olympus_name: String,
    pub olympus_version: String,
    
    // Heartbeat
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
    
    // Recovery
    pub max_restarts: u32,
    pub restart_window_seconds: u64,
    pub restart_delay_ms: u64,
    pub escalation_delay_seconds: u64,
    
    // Evaluación
    pub self_evaluation_interval_seconds: u64,
    pub health_check_interval_seconds: u64,
    
    // Shutdown
    pub emergency_shutdown_timeout_seconds: u64,
    pub graceful_shutdown_timeout_seconds: u64,
    
    // Métricas
    pub metrics_retention_hours: u64,
    pub metrics_export_interval_seconds: u64,
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    
    // Gobernanza
    pub governance_enabled: bool,
    pub auto_recovery_enabled: bool,
    pub circuit_breaker_enabled: bool,
    
    // Feature Flags
    pub feature_flags_refresh_interval_seconds: u64,
    
    // Rate Limiting
    pub global_rate_limit_rps: u64,
    pub global_rate_limit_burst: u64,
    
    // Supervisión
    pub supervision_enabled: bool,
    pub trinity_priority_monitoring: bool,
    
    // Alertas
    pub alert_on_trinity_failure: bool,
    pub alert_on_critical_health: bool,
    
    // Persistencia
    pub persistence_enabled: bool,
    pub state_backup_interval_minutes: u64,
    
    // Overrides específicos por ambiente
    #[serde(flatten)]
    pub overrides: ConfigOverrides,
}

/// Overrides por ambiente
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigOverrides {
    // Development overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_heartbeat_interval_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_max_restarts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_governance_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_auto_recovery_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_prometheus_enabled: Option<bool>,
    
    // Staging overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_heartbeat_interval_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_max_restarts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_governance_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_prometheus_enabled: Option<bool>,
    
    // Production overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_heartbeat_interval_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_max_restarts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_restart_window_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_governance_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_auto_recovery_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_prometheus_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_alert_on_trinity_failure: Option<bool>,
}

impl Default for ZeusConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            olympus_name: "Olympus".to_string(),
            olympus_version: "15.0.0".to_string(),
            
            heartbeat_interval_ms: 500,
            heartbeat_timeout_ms: 1000,
            
            max_restarts: 3,
            restart_window_seconds: 30,
            restart_delay_ms: 1000,
            escalation_delay_seconds: 10,
            
            self_evaluation_interval_seconds: 5,
            health_check_interval_seconds: 10,
            
            emergency_shutdown_timeout_seconds: 10,
            graceful_shutdown_timeout_seconds: 30,
            
            metrics_retention_hours: 24,
            metrics_export_interval_seconds: 60,
            prometheus_enabled: true,
            prometheus_port: 9090,
            
            governance_enabled: true,
            auto_recovery_enabled: true,
            circuit_breaker_enabled: true,
            
            feature_flags_refresh_interval_seconds: 30,
            
            global_rate_limit_rps: 10000,
            global_rate_limit_burst: 1000,
            
            supervision_enabled: true,
            trinity_priority_monitoring: true,
            
            alert_on_trinity_failure: true,
            alert_on_critical_health: true,
            
            persistence_enabled: true,
            state_backup_interval_minutes: 5,
            
            overrides: ConfigOverrides::default(),
        }
    }
}

#[allow(dead_code)]
impl ZeusConfig {
    /// Crea configuración desde variables de entorno
    pub fn from_env() -> Self {
        let environment = std::env::var("OLYMPUS_ENV")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Environment::Development);
        
        let mut config = Self::default();
        config.environment = environment;
        
        // Aplicar overrides por ambiente
        config.apply_environment_overrides();
        
        // Override con variables de entorno
        if let Ok(val) = std::env::var("ZEUS_HEARTBEAT_MS") {
            if let Ok(v) = val.parse() {
                config.heartbeat_interval_ms = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_MAX_RESTARTS") {
            if let Ok(v) = val.parse() {
                config.max_restarts = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_RESTART_WINDOW_S") {
            if let Ok(v) = val.parse() {
                config.restart_window_seconds = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_EVAL_INTERVAL_S") {
            if let Ok(v) = val.parse() {
                config.self_evaluation_interval_seconds = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_SHUTDOWN_TIMEOUT_S") {
            if let Ok(v) = val.parse() {
                config.emergency_shutdown_timeout_seconds = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_METRICS_RETENTION_H") {
            if let Ok(v) = val.parse() {
                config.metrics_retention_hours = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_GOVERNANCE_ENABLED") {
            if let Ok(v) = val.parse() {
                config.governance_enabled = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_AUTO_RECOVERY_ENABLED") {
            if let Ok(v) = val.parse() {
                config.auto_recovery_enabled = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_PROMETHEUS_ENABLED") {
            if let Ok(v) = val.parse() {
                config.prometheus_enabled = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_PROMETHEUS_PORT") {
            if let Ok(v) = val.parse() {
                config.prometheus_port = v;
            }
        }
        
        if let Ok(val) = std::env::var("ZEUS_GLOBAL_RATE_LIMIT_RPS") {
            if let Ok(v) = val.parse() {
                config.global_rate_limit_rps = v;
            }
        }
        
        info!("⚡ Zeus: Configuration loaded from environment ({:?})", environment);
        
        config
    }
    
    /// Carga configuración desde archivo
    pub async fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| ConfigError::FileError { 
                path: path.to_string(), 
                error: e.to_string() 
            })?;
        
        let config: ZeusConfig = serde_yaml::from_str(&content)
            .or_else(|_| serde_json::from_str(&content))
            .map_err(|e| ConfigError::ParseError { 
                path: path.to_string(), 
                error: e.to_string() 
            })?;
        
        config.validate()?;
        
        info!("⚡ Zeus: Configuration loaded from {}", path);
        Ok(config)
    }
    
    /// Guarda configuración a archivo
    pub async fn save_to_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        tokio::fs::write(path, content).await
            .map_err(|e| ConfigError::FileError { 
                path: path.to_string(), 
                error: e.to_string() 
            })?;
        
        Ok(())
    }
    
    /// Valida la configuración
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validar heartbeats
        if self.heartbeat_interval_ms < 100 {
            return Err(ConfigError::ValidationError {
                field: "heartbeat_interval_ms".to_string(),
                reason: "Must be at least 100ms".to_string(),
            });
        }
        
        if self.heartbeat_timeout_ms <= self.heartbeat_interval_ms {
            return Err(ConfigError::ValidationError {
                field: "heartbeat_timeout_ms".to_string(),
                reason: "Must be greater than heartbeat_interval_ms".to_string(),
            });
        }
        
        // Validar recovery
        if self.max_restarts == 0 {
            return Err(ConfigError::ValidationError {
                field: "max_restarts".to_string(),
                reason: "Must be at least 1".to_string(),
            });
        }
        
        if self.restart_window_seconds < 1 {
            return Err(ConfigError::ValidationError {
                field: "restart_window_seconds".to_string(),
                reason: "Must be at least 1 second".to_string(),
            });
        }
        
        // Validar timeouts
        if self.emergency_shutdown_timeout_seconds < 1 {
            return Err(ConfigError::ValidationError {
                field: "emergency_shutdown_timeout_seconds".to_string(),
                reason: "Must be at least 1 second".to_string(),
            });
        }
        
        // Validar métricas
        if self.metrics_retention_hours < 1 {
            return Err(ConfigError::ValidationError {
                field: "metrics_retention_hours".to_string(),
                reason: "Must be at least 1 hour".to_string(),
            });
        }
        
        if self.prometheus_port == 0 {
            return Err(ConfigError::ValidationError {
                field: "prometheus_port".to_string(),
                reason: "Port cannot be 0".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Aplica overrides específicos por ambiente
    fn apply_environment_overrides(&mut self) {
        match self.environment {
            Environment::Development => {
                if let Some(v) = self.overrides.dev_heartbeat_interval_ms {
                    self.heartbeat_interval_ms = v;
                }
                if let Some(v) = self.overrides.dev_max_restarts {
                    self.max_restarts = v;
                }
                if let Some(v) = self.overrides.dev_governance_enabled {
                    self.governance_enabled = v;
                }
                if let Some(v) = self.overrides.dev_auto_recovery_enabled {
                    self.auto_recovery_enabled = v;
                }
                if let Some(v) = self.overrides.dev_prometheus_enabled {
                    self.prometheus_enabled = v;
                }
            }
            Environment::Staging => {
                if let Some(v) = self.overrides.staging_heartbeat_interval_ms {
                    self.heartbeat_interval_ms = v;
                }
                if let Some(v) = self.overrides.staging_max_restarts {
                    self.max_restarts = v;
                }
                if let Some(v) = self.overrides.staging_governance_enabled {
                    self.governance_enabled = v;
                }
                if let Some(v) = self.overrides.staging_prometheus_enabled {
                    self.prometheus_enabled = v;
                }
            }
            Environment::Production => {
                if let Some(v) = self.overrides.prod_heartbeat_interval_ms {
                    self.heartbeat_interval_ms = v;
                }
                if let Some(v) = self.overrides.prod_max_restarts {
                    self.max_restarts = v;
                }
                if let Some(v) = self.overrides.prod_restart_window_seconds {
                    self.restart_window_seconds = v;
                }
                if let Some(v) = self.overrides.prod_governance_enabled {
                    self.governance_enabled = v;
                }
                if let Some(v) = self.overrides.prod_auto_recovery_enabled {
                    self.auto_recovery_enabled = v;
                }
                if let Some(v) = self.overrides.prod_prometheus_enabled {
                    self.prometheus_enabled = v;
                }
                if let Some(v) = self.overrides.prod_alert_on_trinity_failure {
                    self.alert_on_trinity_failure = v;
                }
            }
        }
    }
    
    /// Obtiene configuración específica para producción
    pub fn for_production() -> Self {
        let mut config = Self::default();
        config.environment = Environment::Production;
        config.heartbeat_interval_ms = 250;
        config.heartbeat_timeout_ms = 500;
        config.max_restarts = 5;
        config.restart_window_seconds = 60;
        config.self_evaluation_interval_seconds = 3;
        config.health_check_interval_seconds = 5;
        config.metrics_retention_hours = 72;
        config.prometheus_enabled = true;
        config.trinity_priority_monitoring = true;
        config.alert_on_trinity_failure = true;
        config.persistence_enabled = true;
        config.state_backup_interval_minutes = 1;
        config
    }
    
    /// Obtiene configuración específica para development
    pub fn for_development() -> Self {
        let mut config = Self::default();
        config.environment = Environment::Development;
        config.heartbeat_interval_ms = 1000;
        config.max_restarts = 3;
        config.self_evaluation_interval_seconds = 10;
        config.prometheus_enabled = false;
        config.state_backup_interval_minutes = 10;
        config
    }
    
    /// Obtiene configuración específica para staging
    pub fn for_staging() -> Self {
        let mut config = Self::default();
        config.environment = Environment::Staging;
        config.heartbeat_interval_ms = 500;
        config.max_restarts = 4;
        config.prometheus_enabled = true;
        config.state_backup_interval_minutes = 5;
        config
    }
}

/// Manager de configuración con hot-reloading
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<ZeusConfig>>,
    config_tx: watch::Sender<ZeusConfig>,
    config_rx: watch::Receiver<ZeusConfig>,
    config_path: Arc<RwLock<Option<String>>>,
    hot_reload_enabled: Arc<RwLock<bool>>,
}

impl ConfigManager {
    pub fn new(config: ZeusConfig) -> Self {
        let (config_tx, config_rx) = watch::channel(config.clone());
        
        Self {
            config: Arc::new(RwLock::new(config)),
            config_tx,
            config_rx,
            config_path: Arc::new(RwLock::new(None)),
            hot_reload_enabled: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Obtiene configuración actual
    pub async fn get_config(&self) -> ZeusConfig {
        self.config.read().await.clone()
    }
    
    /// Actualiza configuración
    pub async fn update_config(&self, new_config: ZeusConfig) -> Result<(), ConfigError> {
        // Validar nueva configuración
        new_config.validate()?;
        
        let mut config = self.config.write().await;
        *config = new_config.clone();
        drop(config);
        
        // Notificar cambio
        let _ = self.config_tx.send(new_config);
        
        info!("⚡ Zeus: Configuration updated");
        Ok(())
    }
    
    /// Suscribe a cambios de configuración
    pub fn subscribe(&self) -> watch::Receiver<ZeusConfig> {
        self.config_rx.clone()
    }
    
    /// Carga configuración desde archivo y habilita hot-reloading
    pub async fn load_from_file(&self, path: &str) -> Result<(), ConfigError> {
        let config = ZeusConfig::from_file(path).await?;
        self.update_config(config).await?;
        
        let mut path_guard = self.config_path.write().await;
        *path_guard = Some(path.to_string());
        drop(path_guard);
        
        info!("⚡ Zeus: Configuration loaded from {} (hot-reload enabled)", path);
        Ok(())
    }
    
    /// Inicia hot-reloading
    pub async fn start_hot_reload(&self, interval_seconds: u64) {
        let mut enabled = self.hot_reload_enabled.write().await;
        *enabled = true;
        drop(enabled);
        
        let config = self.config.clone();
        let config_tx = self.config_tx.clone();
        let config_path = self.config_path.clone();
        let hot_reload = self.hot_reload_enabled.clone();
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
            
            loop {
                ticker.tick().await;
                
                // Verificar si hot-reload está habilitado
                if !*hot_reload.read().await {
                    break;
                }
                
                // Verificar si hay path configurado
                let path_guard = config_path.read().await;
                if let Some(ref path) = *path_guard {
                    let path = path.clone();
                    drop(path_guard);
                    
                    // Intentar recargar
                    if let Ok(new_config) = ZeusConfig::from_file(&path).await {
                        let current_config = config.read().await.clone();
                        
                        // Solo actualizar si cambió
                        if serde_json::to_string(&new_config).unwrap() != 
                           serde_json::to_string(&current_config).unwrap() {
                            if let Err(e) = new_config.validate() {
                                warn!("⚡ Zeus: Hot-reload failed - invalid config: {:?}", e);
                                continue;
                            }
                            
                            let mut c = config.write().await;
                            *c = new_config.clone();
                            drop(c);
                            
                            let _ = config_tx.send(new_config);
                            info!("⚡ Zeus: Configuration hot-reloaded from {}", path);
                        }
                    }
                } else {
                    drop(path_guard);
                }
            }
        });
    }
    
    /// Detiene hot-reloading
    pub async fn stop_hot_reload(&self) {
        let mut enabled = self.hot_reload_enabled.write().await;
        *enabled = false;
        info!("⚡ Zeus: Hot-reload stopped");
    }
    
    /// Verifica si hot-reload está habilitado
    pub async fn is_hot_reload_enabled(&self) -> bool {
        *self.hot_reload_enabled.read().await
    }
    
    /// Obtiene path de configuración
    pub async fn get_config_path(&self) -> Option<String> {
        self.config_path.read().await.clone()
    }
}

/// Errores de configuración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    FileError { path: String, error: String },
    ParseError { path: String, error: String },
    SerializeError(String),
    ValidationError { field: String, reason: String },
    NotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileError { path, error } => {
                write!(f, "File error for {}: {}", path, error)
            }
            ConfigError::ParseError { path, error } => {
                write!(f, "Parse error for {}: {}", path, error)
            }
            ConfigError::SerializeError(e) => {
                write!(f, "Serialize error: {}", e)
            }
            ConfigError::ValidationError { field, reason } => {
                write!(f, "Validation error for {}: {}", field, reason)
            }
            ConfigError::NotFound(msg) => {
                write!(f, "Not found: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
