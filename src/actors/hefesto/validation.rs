// src/actors/hefesto/validation.rs
// OLYMPUS v15 - Validación de Schemas

use crate::actors::GodName;
use crate::errors::ActorError;
use serde::{Deserialize, Serialize};

/// Validador de schemas
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    schemas: std::collections::HashMap<String, ConfigSchema>,
}

impl SchemaValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            schemas: std::collections::HashMap::new(),
        };
        validator.register_default_schemas();
        validator
    }

    fn register_default_schemas(&mut self) {
        // Schema para configuraciones de base de datos
        self.register_schema(
            "database",
            ConfigSchema {
                field_type: FieldType::Object,
                required_fields: vec!["host".to_string(), "port".to_string()],
                allowed_values: None,
            },
        );

        // Schema para configuraciones de logging
        self.register_schema(
            "logging",
            ConfigSchema {
                field_type: FieldType::String,
                required_fields: vec![],
                allowed_values: Some(vec![
                    "debug".to_string(),
                    "info".to_string(),
                    "warn".to_string(),
                    "error".to_string(),
                ]),
            },
        );
    }

    pub fn register_schema(&mut self, key: &str, schema: ConfigSchema) {
        self.schemas.insert(key.to_string(), schema);
    }

    pub fn validate(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<ValidationResult, ActorError> {
        // Si no hay schema registrado, permitir cualquier valor
        let Some(schema) = self.schemas.get(key) else {
            return Ok(ValidationResult::valid());
        };

        let mut errors = Vec::new();

        // Validar tipo
        match schema.field_type {
            FieldType::String => {
                if !value.is_string() {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: "Debe ser un string".to_string(),
                    });
                }
            }
            FieldType::Number => {
                if !value.is_number() {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: "Debe ser un número".to_string(),
                    });
                }
            }
            FieldType::Boolean => {
                if !value.is_boolean() {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: "Debe ser un booleano".to_string(),
                    });
                }
            }
            FieldType::Object => {
                if !value.is_object() {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: "Debe ser un objeto".to_string(),
                    });
                }
            }
            FieldType::Array => {
                if !value.is_array() {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: "Debe ser un array".to_string(),
                    });
                }
            }
        }

        // Validar valores permitidos
        if let Some(allowed) = &schema.allowed_values {
            if let Some(val_str) = value.as_str() {
                if !allowed.contains(&val_str.to_string()) {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: format!("Valor no permitido: {}", val_str),
                    });
                }
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        })
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema de configuración
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    pub field_type: FieldType,
    pub required_fields: Vec<String>,
    pub allowed_values: Option<Vec<String>>,
}

/// Tipos de campo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// Resultado de validación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid() -> Self {
        Self {
            is_valid: false,
            errors: Vec::new(),
        }
    }
}

/// Error de validación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}
