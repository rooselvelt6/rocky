// src/actors/hera/rules.rs
// Business Rules Engine

use serde_json::Value;
use std::collections::HashMap;
use crate::actors::hera::ValidationRule;

#[derive(Debug, Clone)]
pub struct RuleEngine {
    rules: HashMap<String, Vec<ValidationRule>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: HashMap::new(),
        };
        engine.load_default_rules();
        engine
    }
    
    fn load_default_rules(&mut self) {
        // Glasgow rules
        let mut glasgow_rules = Vec::new();
        glasgow_rules.push(ValidationRule {
            name: "glasgow_score_consistency".to_string(),
            rule_type: "consistency".to_string(),
            validation_pattern: "eye + verbal + motor = score".to_string(),
            error_message: "Glasgow score must equal sum of components".to_string(),
            is_required: true,
        });
        self.rules.insert("glasgow".to_string(), glasgow_rules);
        
        // Vital signs rules
        let mut vital_rules = Vec::new();
        vital_rules.push(ValidationRule {
            name: "pulse_pressure".to_string(),
            rule_type: "clinical".to_string(),
            validation_pattern: "systolic > diastolic".to_string(),
            error_message: "Systolic BP must be greater than diastolic BP".to_string(),
            is_required: true,
        });
        vital_rules.push(ValidationRule {
            name: "map_calculation".to_string(),
            rule_type: "clinical".to_string(),
            validation_pattern: "MAP = DBP + (SBP - DBP)/3".to_string(),
            error_message: "Mean arterial pressure calculation".to_string(),
            is_required: false,
        });
        self.rules.insert("vital_signs".to_string(), vital_rules);
        
        // Patient rules
        let mut patient_rules = Vec::new();
        patient_rules.push(ValidationRule {
            name: "age_reasonable".to_string(),
            rule_type: "range".to_string(),
            validation_pattern: "0 < age < 150".to_string(),
            error_message: "Age must be between 0 and 150".to_string(),
            is_required: true,
        });
        self.rules.insert("patient".to_string(), patient_rules);
    }
    
    pub fn validate_rules(&self, data: &Value, schema_name: &str) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        
        if let Some(rules) = self.rules.get(schema_name) {
            for rule in rules {
                match self.apply_rule(rule, data) {
                    RuleResult::Pass => {},
                    RuleResult::Warning(msg) => warnings.push(msg),
                    RuleResult::Error(msg) => {
                        if rule.is_required {
                            return Err(msg);
                        } else {
                            warnings.push(msg);
                        }
                    }
                }
            }
        }
        
        Ok(warnings)
    }
    
    fn apply_rule(&self, rule: &ValidationRule, data: &Value) -> RuleResult {
        match rule.name.as_str() {
            "glasgow_score_consistency" => self.validate_glasgow_consistency(data),
            "pulse_pressure" => self.validate_pulse_pressure(data),
            "age_reasonable" => self.validate_age(data),
            _ => RuleResult::Pass,
        }
    }
    
    fn validate_glasgow_consistency(&self, data: &Value) -> RuleResult {
        if let (Some(eye), Some(verbal), Some(motor), Some(score)) = (
            data.get("eye_response").and_then(|v| v.as_u64()),
            data.get("verbal_response").and_then(|v| v.as_u64()),
            data.get("motor_response").and_then(|v| v.as_u64()),
            data.get("score").and_then(|v| v.as_u64()),
        ) {
            let calculated = eye + verbal + motor;
            if calculated != score {
                return RuleResult::Error(format!(
                    "Glasgow score mismatch: components sum to {} but score is {}",
                    calculated, score
                ));
            }
        }
        RuleResult::Pass
    }
    
    fn validate_pulse_pressure(&self, data: &Value) -> RuleResult {
        if let (Some(sbp), Some(dbp)) = (
            data.get("systolic_bp").and_then(|v| v.as_u64()),
            data.get("diastolic_bp").and_then(|v| v.as_u64()),
        ) {
            if sbp <= dbp {
                return RuleResult::Error(
                    "Systolic BP must be greater than diastolic BP".to_string()
                );
            }
            
            let pulse_pressure = sbp - dbp;
            if pulse_pressure < 20 {
                return RuleResult::Warning(
                    "Narrow pulse pressure detected - consider cardiac evaluation".to_string()
                );
            }
            if pulse_pressure > 100 {
                return RuleResult::Warning(
                    "Wide pulse pressure detected - consider aortic regurgitation".to_string()
                );
            }
        }
        RuleResult::Pass
    }
    
    fn validate_age(&self, data: &Value) -> RuleResult {
        if let Some(age) = data.get("age").and_then(|v| v.as_u64()) {
            if age == 0 {
                return RuleResult::Warning("Age is 0 - verify if this is correct".to_string());
            }
            if age > 120 {
                return RuleResult::Warning("Age > 120 years - verify data entry".to_string());
            }
        }
        RuleResult::Pass
    }
    
    pub fn add_rule(&mut self, rule: ValidationRule) {
        let schema_name = rule.rule_type.clone();
        self.rules.entry(schema_name)
            .or_insert_with(Vec::new)
            .push(rule);
    }
}

#[derive(Debug, Clone)]
enum RuleResult {
    Pass,
    Warning(String),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_glasgow_consistency() {
        let engine = RuleEngine::new();
        
        // Valid Glasgow
        let valid = json!({
            "eye_response": 4,
            "verbal_response": 5,
            "motor_response": 6,
            "score": 15
        });
        assert!(engine.validate_rules(&valid, "glasgow").is_ok());
        
        // Invalid Glasgow (inconsistent score)
        let invalid = json!({
            "eye_response": 4,
            "verbal_response": 5,
            "motor_response": 6,
            "score": 14
        });
        assert!(engine.validate_rules(&invalid, "glasgow").is_err());
    }

    #[test]
    fn test_pulse_pressure() {
        let engine = RuleEngine::new();
        
        // Valid BP
        let valid = json!({
            "systolic_bp": 120,
            "diastolic_bp": 80
        });
        assert!(engine.validate_rules(&valid, "vital_signs").is_ok());
        
        // Invalid BP (systolic < diastolic)
        let invalid = json!({
            "systolic_bp": 80,
            "diastolic_bp": 120
        });
        assert!(engine.validate_rules(&invalid, "vital_signs").is_err());
    }
}
