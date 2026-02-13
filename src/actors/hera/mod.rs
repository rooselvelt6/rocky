// src/actors/hera/mod.rs
// OLYMPUS v15 - Hera: Reina de los Dioses y Validación de Datos

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, ResponsePayload, CommandPayload, QueryPayload, EventPayload};
use crate::errors::ActorError;

pub mod validators;
pub mod schemas;
pub mod rules;

use validators::*;
use schemas::*;
use rules::*;

#[derive(Debug, Clone)]
pub struct Hera {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Validation components
    schema_validator: Arc<RwLock<SchemaValidator>>,
    data_validator: Arc<RwLock<DataValidator>>,
    rule_engine: Arc<RwLock<RuleEngine>>,
    validation_history: Arc<RwLock<Vec<ValidationResult>>>,
}

impl Hera {
    pub async fn new() -> Self {
        Self {
            name: GodName::Hera,
            state: ActorState::new(GodName::Hera),
            config: ActorConfig::default(),
            
            schema_validator: Arc::new(RwLock::new(SchemaValidator::new())),
            data_validator: Arc::new(RwLock::new(DataValidator::new())),
            rule_engine: Arc::new(RwLock::new(RuleEngine::new())),
            validation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Validate data against schema
    pub async fn validate_data(&self, data: serde_json::Value, schema_name: &str) -> ValidationResult {
        let schema_validator = self.schema_validator.read().await;
        let data_validator = self.data_validator.read().await;
        let rule_engine = self.rule_engine.read().await;
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Schema validation
        if let Err(e) = schema_validator.validate(&data, schema_name) {
            errors.push(e);
        }
        
        // Data type validation
        if let Err(e) = data_validator.validate_types(&data) {
            errors.push(e);
        }
        
        // Range validation
        if let Err(e) = data_validator.validate_ranges(&data, schema_name) {
            errors.push(e);
        }
        
        // Business rules validation
        match rule_engine.validate_rules(&data, schema_name) {
            Ok(warns) => warnings.extend(warns),
            Err(e) => errors.push(e),
        }
        
        let is_valid = errors.is_empty();
        
        let result = ValidationResult {
            is_valid,
            errors,
            warnings,
            validated_at: chrono::Utc::now().to_rfc3339(),
            schema_name: schema_name.to_string(),
        };
        
        // Store in history
        let mut history = self.validation_history.write().await;
        history.push(result.clone());
        
        // Keep only last 1000 validations
        if history.len() > 1000 {
            let to_remove = history.len() - 1000;
            history.drain(0..to_remove);
        }
        
        result
    }
    
    /// Add custom validation rule
    pub async fn add_rule(&self, rule: ValidationRule) {
        let mut engine = self.rule_engine.write().await;
        engine.add_rule(rule);
    }
    
    /// Get validation statistics
    pub async fn get_stats(&self) -> ValidationStats {
        let history = self.validation_history.read().await;
        
        let total = history.len();
        let valid = history.iter().filter(|r| r.is_valid).count();
        let invalid = total - valid;
        
        let mut errors_by_type = HashMap::new();
        for result in history.iter() {
            for error in &result.errors {
                *errors_by_type.entry(error.clone()).or_insert(0) += 1;
            }
        }
        
        ValidationStats {
            total_validations: total,
            valid_count: valid,
            invalid_count: invalid,
            success_rate: if total > 0 { (valid as f64 / total as f64) * 100.0 } else { 0.0 },
            common_errors: errors_by_type,
        }
    }
}

#[async_trait]
impl OlympianActor for Hera {
    fn name(&self) -> GodName {
        GodName::Hera
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::Validation
    }
    
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();
        
        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Event(event) => self.handle_event(event).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }
    
    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Hera",
            "validation_count": self.state.message_count,
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Hera,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            god: GodName::Hera,
            status: ActorStatus::Healthy,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }
    
    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("👑 Hera: Initializing validation system...");
        
        // Initialize default schemas
        let mut schema_validator = self.schema_validator.write().await;
        schema_validator.load_default_schemas();
        
        info!("👑 Hera: Validation system ready");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("👑 Hera: Shutting down validation system");
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Hera {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            _ => Ok(ResponsePayload::Error { 
                error: "Unknown command".to_string(), 
                code: 400 
            }),
        }
    }
    
    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            _ => Ok(ResponsePayload::Data { data: serde_json::json!({}) }),
        }
    }
    
    async fn handle_event(&mut self, event: EventPayload) -> Result<ResponsePayload, ActorError> {
        match event {
            _ => Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub validated_at: String,
    pub schema_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub success_rate: f64,
    pub common_errors: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: String,
    pub validation_pattern: String,
    pub error_message: String,
    pub is_required: bool,
}
