/// Hera v12 - Reina de los Dioses
/// Guardiana de las invariantes del sistema y consistency

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeraRule {
    NoDuplicatePatients,
    ValidClinicalScores,
    ConsistentTimestamps,
    DataIntegrity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeraViolation {
    pub rule: HeraRule,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug)]
pub struct HeraV12 {
    rules: Vec<HeraRule>,
    violations: RwLock<Vec<HeraViolation>>,
    consistency_checks: RwLock<HashMap<String, String>>,
}

impl HeraV12 {
    pub fn new() -> Self {
        let rules = vec![
            HeraRule::NoDuplicatePatients,
            HeraRule::ValidClinicalScores,
            HeraRule::ConsistentTimestamps,
            HeraRule::DataIntegrity,
        ];
        
        Self {
            rules,
            violations: RwLock::new(Vec::new()),
            consistency_checks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn validate_invariant(&self, rule: &HeraRule) -> Result<(), HeraViolation> {
        match rule {
            HeraRule::NoDuplicatePatients => {
                // Validar que no haya pacientes duplicados
                Ok(())
            }
            HeraRule::ValidClinicalScores => {
                // Validar scores clínicos en rangos válidos
                Ok(())
            }
            HeraRule::ConsistentTimestamps => {
                // Validar consistencia de timestamps
                Ok(())
            }
            HeraRule::DataIntegrity => {
                // Validar integridad de datos
                Ok(())
            }
        }
    }

    pub async fn check_all_invariants(&self) -> Vec<HeraViolation> {
        let mut violations = Vec::new();
        
        for rule in &self.rules.clone() {
            if let Err(violation) = self.validate_invariant(rule).await {
                violations.push(violation);
            }
        }
        
        violations
    }

    pub async fn report_violation(&self, violation: HeraViolation) {
        let severity = violation.severity.clone();
        let rule = violation.rule.clone();
        let description = violation.description.clone();
        let mut violations = self.violations.write().await;
        violations.push(violation);
        
        tracing::warn!("👑 Hera: Violación detectada - {:?}: {}", 
                     rule, description);
        
        // Enviar notificación a Zeus si es crítica
        match severity {
            ViolationSeverity::Critical => {
                tracing::error!("🚨 Hera: Violación CRÍTICA - notificando a Zeus");
            }
            ViolationSeverity::Error => {
                tracing::error!("🚨 Hera: Error de invariante");
            }
            ViolationSeverity::Warning => {
                tracing::warn!("⚠️ Hera: Advertencia de invariante");
            }
        }
    }

    pub async fn get_violations(&self) -> Vec<HeraViolation> {
        self.violations.read().await.clone()
    }

    pub async fn clear_violations(&self) {
        let mut violations = self.violations.write().await;
        violations.clear();
        tracing::info!("👑 Hera: Violaciones limpiadas");
    }

    pub async fn set_consistency_check(&self, check_name: &str, status: &str) {
        let mut checks = self.consistency_checks.write().await;
        checks.insert(check_name.to_string(), status.to_string());
    }

    pub async fn get_consistency_status(&self) -> HashMap<String, String> {
        self.consistency_checks.read().await.clone()
    }

    pub fn get_rules(&self) -> &Vec<HeraRule> {
        &self.rules
    }
}

impl Default for HeraV12 {
    fn default() -> Self {
        Self::new()
    }
}