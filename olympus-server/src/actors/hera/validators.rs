// src/actors/hera/validators.rs
// Data Validators

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DataValidator;

impl DataValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate data types
    pub fn validate_types(&self, data: &Value) -> Result<(), String> {
        // Basic type validation
        if !data.is_object() && !data.is_array() {
            return Err("Data must be an object or array".to_string());
        }
        Ok(())
    }
    
    /// Validate clinical ranges
    pub fn validate_ranges(&self, data: &Value, schema_name: &str) -> Result<(), String> {
        match schema_name {
            "patient" => self.validate_patient_ranges(data),
            "glasgow" => self.validate_glasgow_ranges(data),
            "apache" => self.validate_apache_ranges(data),
            "sofa" => self.validate_sofa_ranges(data),
            "news2" => self.validate_news2_ranges(data),
            "vital_signs" => self.validate_vital_signs_ranges(data),
            _ => Ok(()),
        }
    }
    
    fn validate_patient_ranges(&self, data: &Value) -> Result<(), String> {
        if let Some(age) = data.get("age").and_then(|v| v.as_u64()) {
            if age > 150 {
                return Err("Age must be <= 150 years".to_string());
            }
        }
        
        if let Some(weight) = data.get("weight").and_then(|v| v.as_f64()) {
            if weight < 0.5 || weight > 500.0 {
                return Err("Weight must be between 0.5 and 500 kg".to_string());
            }
        }
        
        if let Some(height) = data.get("height").and_then(|v| v.as_f64()) {
            if height < 20.0 || height > 300.0 {
                return Err("Height must be between 20 and 300 cm".to_string());
            }
        }
        
        Ok(())
    }
    
    fn validate_glasgow_ranges(&self, data: &Value) -> Result<(), String> {
        if let Some(eye) = data.get("eye_response").and_then(|v| v.as_u64()) {
            if eye < 1 || eye > 4 {
                return Err("Eye response must be 1-4".to_string());
            }
        }
        
        if let Some(verbal) = data.get("verbal_response").and_then(|v| v.as_u64()) {
            if verbal < 1 || verbal > 5 {
                return Err("Verbal response must be 1-5".to_string());
            }
        }
        
        if let Some(motor) = data.get("motor_response").and_then(|v| v.as_u64()) {
            if motor < 1 || motor > 6 {
                return Err("Motor response must be 1-6".to_string());
            }
        }
        
        if let Some(score) = data.get("score").and_then(|v| v.as_u64()) {
            if score < 3 || score > 15 {
                return Err("Glasgow score must be 3-15".to_string());
            }
        }
        
        Ok(())
    }
    
    fn validate_apache_ranges(&self, data: &Value) -> Result<(), String> {
        if let Some(temp) = data.get("temperature").and_then(|v| v.as_f64()) {
            if temp < 25.0 || temp > 45.0 {
                return Err("Temperature must be between 25-45°C".to_string());
            }
        }
        
        if let Some(map) = data.get("mean_arterial_pressure").and_then(|v| v.as_i64()) {
            if map < 20 || map > 250 {
                return Err("MAP must be between 20-250 mmHg".to_string());
            }
        }
        
        if let Some(hr) = data.get("heart_rate").and_then(|v| v.as_i64()) {
            if hr < 20 || hr > 250 {
                return Err("Heart rate must be between 20-250 bpm".to_string());
            }
        }
        
        if let Some(rr) = data.get("respiratory_rate").and_then(|v| v.as_i64()) {
            if rr < 4 || rr > 60 {
                return Err("Respiratory rate must be between 4-60 breaths/min".to_string());
            }
        }
        
        if let Some(ph) = data.get("arterial_ph").and_then(|v| v.as_f64()) {
            if ph < 6.8 || ph > 7.8 {
                return Err("Arterial pH must be between 6.8-7.8".to_string());
            }
        }
        
        Ok(())
    }
    
    fn validate_sofa_ranges(&self, data: &Value) -> Result<(), String> {
        if let Some(pf) = data.get("pao2_fio2").and_then(|v| v.as_i64()) {
            if pf < 0 || pf > 600 {
                return Err("PaO2/FiO2 must be between 0-600".to_string());
            }
        }
        
        if let Some(plt) = data.get("platelets").and_then(|v| v.as_i64()) {
            if plt < 0 || plt > 1000 {
                return Err("Platelets must be between 0-1000 x10³/µL".to_string());
            }
        }
        
        if let Some(bil) = data.get("bilirubin").and_then(|v| v.as_f64()) {
            if bil < 0.0 || bil > 50.0 {
                return Err("Bilirubin must be between 0-50 mg/dL".to_string());
            }
        }
        
        Ok(())
    }
    
    fn validate_news2_ranges(&self, data: &Value) -> Result<(), String> {
        if let Some(rr) = data.get("respiration_rate").and_then(|v| v.as_u64()) {
            if rr < 4 || rr > 60 {
                return Err("Respiration rate must be between 4-60".to_string());
            }
        }
        
        if let Some(spo2) = data.get("spo2").and_then(|v| v.as_u64()) {
            if spo2 < 50 || spo2 > 100 {
                return Err("SpO2 must be between 50-100%".to_string());
            }
        }
        
        if let Some(sbp) = data.get("systolic_bp").and_then(|v| v.as_u64()) {
            if sbp < 40 || sbp > 300 {
                return Err("Systolic BP must be between 40-300 mmHg".to_string());
            }
        }
        
        if let Some(hr) = data.get("heart_rate").and_then(|v| v.as_u64()) {
            if hr < 20 || hr > 250 {
                return Err("Heart rate must be between 20-250 bpm".to_string());
            }
        }
        
        if let Some(temp) = data.get("temperature").and_then(|v| v.as_f64()) {
            if temp < 30.0 || temp > 45.0 {
                return Err("Temperature must be between 30-45°C".to_string());
            }
        }
        
        Ok(())
    }
    
    fn validate_vital_signs_ranges(&self, data: &Value) -> Result<(), String> {
        // Heart rate
        if let Some(hr) = data.get("heart_rate").and_then(|v| v.as_u64()) {
            if hr < 20 || hr > 250 {
                return Err("Heart rate must be between 20-250 bpm".to_string());
            }
        }
        
        // Blood pressure
        if let Some(sbp) = data.get("systolic_bp").and_then(|v| v.as_u64()) {
            if sbp < 40 || sbp > 300 {
                return Err("Systolic BP must be between 40-300 mmHg".to_string());
            }
        }
        
        if let Some(dbp) = data.get("diastolic_bp").and_then(|v| v.as_u64()) {
            if dbp < 20 || dbp > 200 {
                return Err("Diastolic BP must be between 20-200 mmHg".to_string());
            }
        }
        
        // Temperature
        if let Some(temp) = data.get("temperature").and_then(|v| v.as_f64()) {
            if temp < 30.0 || temp > 45.0 {
                return Err("Temperature must be between 30-45°C".to_string());
            }
        }
        
        // Respiratory rate
        if let Some(rr) = data.get("respiratory_rate").and_then(|v| v.as_u64()) {
            if rr < 4 || rr > 60 {
                return Err("Respiratory rate must be between 4-60 breaths/min".to_string());
            }
        }
        
        // SpO2
        if let Some(spo2) = data.get("spo2").and_then(|v| v.as_u64()) {
            if spo2 < 50 || spo2 > 100 {
                return Err("SpO2 must be between 50-100%".to_string());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_glasgow_validation() {
        let validator = DataValidator::new();
        
        // Valid Glasgow
        let valid = json!({
            "eye_response": 4,
            "verbal_response": 5,
            "motor_response": 6,
            "score": 15
        });
        assert!(validator.validate_ranges(&valid, "glasgow").is_ok());
        
        // Invalid Glasgow (eye out of range)
        let invalid = json!({
            "eye_response": 5,
            "verbal_response": 5,
            "motor_response": 6,
            "score": 16
        });
        assert!(validator.validate_ranges(&invalid, "glasgow").is_err());
    }

    #[test]
    fn test_vital_signs_validation() {
        let validator = DataValidator::new();
        
        // Valid vitals
        let valid = json!({
            "heart_rate": 80,
            "systolic_bp": 120,
            "diastolic_bp": 80,
            "temperature": 37.0,
            "respiratory_rate": 16,
            "spo2": 98
        });
        assert!(validator.validate_ranges(&valid, "vital_signs").is_ok());
        
        // Invalid vitals (HR too high)
        let invalid = json!({
            "heart_rate": 300,
            "systolic_bp": 120
        });
        assert!(validator.validate_ranges(&invalid, "vital_signs").is_err());
    }
}
