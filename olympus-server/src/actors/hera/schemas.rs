// src/actors/hera/schemas.rs
// Schema Validator

use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SchemaValidator {
    schemas: HashMap<String, Schema>,
}

impl SchemaValidator {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }
    
    pub fn load_default_schemas(&mut self) {
        // Patient schema
        self.schemas.insert("patient".to_string(), Schema {
            name: "patient".to_string(),
            required_fields: vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
            ],
            optional_fields: vec![
                "weight".to_string(),
                "height".to_string(),
                "medical_history".to_string(),
            ],
        });
        
        // Glasgow schema
        self.schemas.insert("glasgow".to_string(), Schema {
            name: "glasgow".to_string(),
            required_fields: vec![
                "eye_response".to_string(),
                "verbal_response".to_string(),
                "motor_response".to_string(),
                "score".to_string(),
            ],
            optional_fields: vec![
                "patient_id".to_string(),
                "assessed_by".to_string(),
                "assessed_at".to_string(),
            ],
        });
        
        // APACHE schema
        self.schemas.insert("apache".to_string(), Schema {
            name: "apache".to_string(),
            required_fields: vec![
                "temperature".to_string(),
                "mean_arterial_pressure".to_string(),
                "heart_rate".to_string(),
                "respiratory_rate".to_string(),
                "glasgow_coma_score".to_string(),
                "age".to_string(),
            ],
            optional_fields: vec![
                "patient_id".to_string(),
                "chronic_health".to_string(),
            ],
        });
        
        // SOFA schema
        self.schemas.insert("sofa".to_string(), Schema {
            name: "sofa".to_string(),
            required_fields: vec![
                "pao2_fio2".to_string(),
                "platelets".to_string(),
                "bilirubin".to_string(),
                "cardiovascular".to_string(),
                "glasgow".to_string(),
                "renal".to_string(),
            ],
            optional_fields: vec![
                "patient_id".to_string(),
            ],
        });
        
        // NEWS2 schema
        self.schemas.insert("news2".to_string(), Schema {
            name: "news2".to_string(),
            required_fields: vec![
                "respiration_rate".to_string(),
                "spo2".to_string(),
                "systolic_bp".to_string(),
                "heart_rate".to_string(),
                "temperature".to_string(),
                "consciousness".to_string(),
            ],
            optional_fields: vec![
                "patient_id".to_string(),
                "spo2_scale".to_string(),
                "air_or_oxygen".to_string(),
            ],
        });
        
        // Vital signs schema
        self.schemas.insert("vital_signs".to_string(), Schema {
            name: "vital_signs".to_string(),
            required_fields: vec![
                "heart_rate".to_string(),
                "systolic_bp".to_string(),
                "diastolic_bp".to_string(),
                "temperature".to_string(),
                "respiratory_rate".to_string(),
            ],
            optional_fields: vec![
                "spo2".to_string(),
                "pain_score".to_string(),
            ],
        });
    }
    
    pub fn validate(&self, data: &Value, schema_name: &str) -> Result<(), String> {
        let schema = self.schemas.get(schema_name)
            .ok_or_else(|| format!("Schema '{}' not found", schema_name))?;
        
        // Check required fields
        for field in &schema.required_fields {
            if data.get(field).is_none() {
                return Err(format!("Required field '{}' is missing", field));
            }
        }
        
        Ok(())
    }
    
    pub fn add_schema(&mut self, schema: Schema) {
        self.schemas.insert(schema.name.clone(), schema);
    }
    
    pub fn get_schema(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validation() {
        let mut validator = SchemaValidator::new();
        validator.load_default_schemas();
        
        // Valid patient
        let valid = json!({
            "id": "123",
            "name": "John Doe",
            "age": 45
        });
        assert!(validator.validate(&valid, "patient").is_ok());
        
        // Invalid patient (missing required field)
        let invalid = json!({
            "id": "123",
            "name": "John Doe"
        });
        assert!(validator.validate(&invalid, "patient").is_err());
    }
}
