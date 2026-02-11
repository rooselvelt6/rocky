// src/actors/nemesis/rules.rs
// OLYMPUS v15 - Rules: Sistema de Reglas Legales para Némesis

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use regex::Regex;

use crate::errors::ActorError;

/// Motor de reglas legales y políticas
#[derive(Debug, Clone)]
pub struct RuleEngine {
    /// Reglas activas
    active_rules: Arc<RwLock<Vec<LegalRule>>>,
    /// Política de cumplimiento
    compliance_policy: Arc<RwLock<CompliancePolicy>>,
    /// Cache de evaluaciones
    evaluation_cache: Arc<RwLock<HashMap<String, RuleEvaluation>>>,
    /// Métricas del motor de reglas
    metrics: Arc<RwLock<RuleMetrics>>,
}

/// Política de cumplimiento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    /// Versión de la política
    pub version: String,
    /// Estándares regulatorios aplicables
    pub applicable_standards: Vec<RegulatoryStandard>,
    /// Niveles de aplicación de reglas
    pub enforcement_levels: Vec<EnforcementLevel>,
    /// Excepciones y permisos especiales
    pub exceptions: Vec<PolicyException>,
    /// Revisión requerida
    pub review_required: bool,
}

/// Regla legal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRule {
    /// ID único de la regla
    pub rule_id: String,
    /// Nombre descriptivo
    pub name: String,
    /// Descripción de la regla
    pub description: String,
    /// Estándar regulatorio asociado
    pub standard: RegulatoryStandard,
    /// Tipo de regla
    pub rule_type: RuleType,
    /// Severidad de la violación
    pub severity: ViolationSeverity,
    /// Condición para activar la regla
    pub condition: RuleCondition,
    /// Acción a tomar cuando se viola
    pub enforcement_action: EnforcementAction,
    /// Evidencia requerida
    pub evidence_required: Vec<EvidenceType>,
    /// Excepciones
    pub exceptions: Vec<PolicyException>,
    /// Estado de la regla
    pub status: RuleStatus,
    /// Última modificación
    pub last_modified: DateTime<Utc>,
    /// Métricas de la regla
    pub metrics: RuleMetrics,
}

/// Tipos de reglas
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    /// Reglas de acceso y autenticación
    AccessControl,
    /// Reglas de encriptación y protección de datos
    DataProtection,
    /// Reglas de auditoría y logging
    AuditLogging,
    /// Reglas de retención de datos
    DataRetention,
    /// Reglas de consentimiento y privacidad
    ConsentPrivacy,
    /// Reglas de integridad y validación
    IntegrityValidation,
    /// Reglas de disponibilidad y resiliencia
    AvailabilityResilience,
    /// Reglas de documentación y trazabilidad
    DocumentationTraceability,
}

/// Condición de regla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Expresión de la condición
    pub expression: String,
    /// Variables disponibles
    pub available_variables: Vec<String>,
    /// Operador lógico
    pub logical_operator: LogicalOperator,
    /// Sub-condiciones
    pub sub_conditions: Vec<RuleCondition>,
}

/// Operadores lógicos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
    Xor,
}

/// Niveles de aplicación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    /// Solo logging
    LogOnly,
    /// Alerta
    Alert,
    /// Bloqueo de acceso
    BlockAccess,
    /// Escalado
    Escalate,
    /// Rechazo automático
    AutoReject,
    /// Cuarentena
    Quarantine,
}

/// Acciones de aplicación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    /// Generar alerta
    GenerateAlert,
    /// Bloquear operación
    BlockOperation,
    /// Escalar a supervisor
    EscalateToSupervisor,
    /// Rechazar solicitud
    RejectRequest,
    /// Poner en cuarentena
    QuarantineEntity,
    /// Solicitar aprobación
    RequireApproval,
    /// Registrar violación
    LogViolation,
}

/// Excepciones de política
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyException {
    /// ID de la excepción
    pub exception_id: String,
    /// Nombre
    pub name: String,
    /// Descripción
    pub description: String,
    /// Condición para aplicar la excepción
    pub condition: RuleCondition,
    /// Duración de la excepción (horas)
    pub duration_hours: u32,
    /// Quién aprobó la excepción
    pub approved_by: String,
    /// Justificación
    pub justification: String,
    /// Fecha de expiración
    pub expires_at: Option<DateTime<Utc>>,
}

/// Estado de la regla
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleStatus {
    Active,
    Inactive,
    Testing,
    Deprecated,
}

/// Métricas de regla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetrics {
    /// Veces evaluada
    pub evaluations_count: u64,
    /// Veces violada
    pub violations_count: u64,
    /// Tasa de cumplimiento
    pub compliance_rate: f64,
    /// Tiempo promedio de evaluación (ms)
    pub average_evaluation_time_ms: f64,
    /// Última evaluación
    pub last_evaluation: Option<DateTime<Utc>>,
}

/// Resultado de evaluación de regla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluation {
    /// ID de la regla evaluada
    pub rule_id: String,
    /// Objetivo evaluado
    pub target: String,
    /// Contexto de evaluación
    pub context: HashMap<String, serde_json::Value>,
    /// Resultado de la evaluación
    pub result: RuleResult,
    /// Confianza en el resultado
    pub confidence: f64,
    /// Tiempo de evaluación (ms)
    pub evaluation_time_ms: u64,
    /// Detalles del resultado
    pub details: RuleResultDetails,
    /// Evidencia recopilada
    pub evidence: Vec<String>,
}

/// Resultado de regla
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleResult {
    /// Cumple con la regla
    Compliant,
    /// Violación de la regla
    Violated,
    /// No aplicable
    NotApplicable,
    /// Indeterminado
    Indeterminate,
}

/// Detalles del resultado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResultDetails {
    /// Razón del resultado
    pub reason: String,
    /// Factores considerados
    pub factors: Vec<String>,
    /// Condiciones evaluadas
    pub evaluated_conditions: Vec<String>,
    /// Puntuación de cumplimiento
    pub compliance_score: f64,
    /// Umbral de decisión
    pub decision_threshold: f64,
}

/// Violación detectada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedViolation {
    /// ID de la violación
    pub violation_id: String,
    /// Regla violada
    pub violated_rule: String,
    /// Severidad
    pub severity: ViolationSeverity,
    /// Descripción
    pub description: String,
    /// Impacto
    pub impact: String,
    /// Recomendaciones
    pub recommendations: Vec<String>,
    /// Evidencia
    pub evidence: Vec<String>,
}

impl RuleEngine {
    /// Crea una nueva instancia del motor de reglas
    pub fn new() -> Self {
        Self {
            active_rules: Arc::new(RwLock::new(Vec::new())),
            compliance_policy: Arc::new(RwLock::new(CompliancePolicy::default())),
            evaluation_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(RuleMetrics::default())),
        }
    }
    
    /// Inicializa el motor de reglas
    pub async fn initialize(&self) -> Result<(), ActorError> {
        // Crear reglas por defecto para diferentes estándares
        let default_rules = self.create_default_rules().await;
        
        {
            let mut rules = self.active_rules.write().await;
            rules.extend(default_rules);
        }
        
        // Establecer política por defecto
        {
            let mut policy = self.compliance_policy.write().await;
            *policy = CompliancePolicy::default();
        }
        
        info!("⚖️ RuleEngine inicializado con {} reglas por defecto", 
            self.active_rules.read().await.len());
        
        Ok(())
    }
    
    /// Evalúa un contexto contra las reglas activas
    pub async fn evaluate_context(&self, context: &serde_json::Value) -> Result<Vec<LegalRule>, ActorError> {
        let mut violated_rules = Vec::new();
        let active_rules = self.active_rules.read().await;
        
        for rule in active_rules.iter() {
            if let Ok(evaluation) = self.evaluate_rule(rule, context).await {
                if evaluation.result == RuleResult::Violated {
                    violated_rules.push(rule.clone());
                }
            }
        }
        
        Ok(violated_rules)
    }
    
    /// Evalúa una regla específica contra un contexto
    pub async fn evaluate_rule(&self, rule: &LegalRule, context: &serde_json::Value) -> Result<RuleEvaluation, ActorError> {
        let start_time = std::time::Instant::now();
        
        // Convertir JSON a HashMap para procesamiento
        let context_map = self.json_to_hashmap(context);
        
        // Evaluar condición de la regla
        let (compliant, confidence, details) = self.evaluate_condition(&rule.condition, &context_map).await?;
        
        let result = if compliant {
            RuleResult::Compliant
        } else {
            RuleResult::Violated
        };
        
        // Generar detalles del resultado
        let result_details = RuleResultDetails {
            reason: self.generate_result_description(&result, &details).await,
            factors: details,
            evaluated_conditions: vec![rule.condition.expression.clone()],
            compliance_score: self.calculate_compliance_score(&rule, compliant, confidence),
            decision_threshold: 0.7,
        };
        
        // Recopilar evidencia
        let evidence = self.collect_evidence(rule, &context_map).await;
        
        let evaluation = RuleEvaluation {
            rule_id: rule.rule_id.clone(),
            target: "system".to_string(),
            context: context_map.clone(),
            result,
            confidence,
            evaluation_time_ms: start_time.elapsed().as_millis() as u64,
            details: result_details,
            evidence,
        };
        
        // Actualizar métricas de la regla
        self.update_rule_metrics(rule, &evaluation).await;
        
        Ok(evaluation)
    }
    
    /// Crea reglas por defecto para estándares comunes
    async fn create_default_rules(&self) -> Vec<LegalRule> {
        let mut rules = Vec::new();
        
        // Regla de autenticación fuerte (HIPAA)
        rules.push(LegalRule {
            rule_id: "auth_001".to_string(),
            name: "Strong Authentication Required".to_string(),
            description: "Autenticación multifactor requerida para acceso a datos sensibles".to_string(),
            standard: crate::actors::nemesis::compliance::RegulatoryStandard::HIPAA,
            rule_type: RuleType::AccessControl,
            severity: crate::actors::nemesis::compliance::ViolationSeverity::High,
            condition: RuleCondition {
                expression: "!has_mfa && is_sensitive_access".to_string(),
                available_variables: vec!["has_mfa".to_string(), "is_sensitive_access".to_string()],
                logical_operator: LogicalOperator::And,
                sub_conditions: Vec::new(),
            },
            enforcement_action: EnforcementAction::BlockAccess,
            evidence_required: vec![
                crate::actors::nemesis::compliance::EvidenceType::SystemMetric,
                crate::actors::nemesis::compliance::EvidenceType::LogRecord,
            ],
            exceptions: Vec::new(),
            status: RuleStatus::Active,
            last_modified: Utc::now(),
            metrics: RuleMetrics::default(),
        });
        
        // Regla de encriptación de datos (GDPR)
        rules.push(LegalRule {
            rule_id: "encryption_001".to_string(),
            name: "Data at Rest Encryption".to_string(),
            description: "Datos personales deben estar encriptados en reposo".to_string(),
            standard: crate::actors::nemesis::compliance::RegulatoryStandard::GDPR,
            rule_type: RuleType::DataProtection,
            severity: crate::actors::nemesis::compliance::ViolationSeverity::Critical,
            condition: RuleCondition {
                expression: "is_personal_data && !is_encrypted".to_string(),
                available_variables: vec!["is_personal_data".to_string(), "is_encrypted".to_string()],
                logical_operator: LogicalOperator::And,
                sub_conditions: Vec::new(),
            },
            enforcement_action: EnforcementAction::BlockOperation,
            evidence_required: vec![
                crate::actors::nemesis::compliance::EvidenceType::DatabaseRecord,
                crate::actors::nemesis::compliance::EvidenceType::ConfigurationFile,
            ],
            exceptions: Vec::new(),
            status: RuleStatus::Active,
            last_modified: Utc::now(),
            metrics: RuleMetrics::default(),
        });
        
        // Regla de retención de logs (varios estándares)
        rules.push(LegalRule {
            rule_id: "audit_001".to_string(),
            name: "Audit Log Retention".to_string(),
            description: "Logs de auditoría deben retenerse por el período requerido".to_string(),
            standard: crate::actors::nemesis::compliance::RegulatoryStandard::SOC2,
            rule_type: RuleType::AuditLogging,
            severity: crate::actors::nemesis::compliance::ViolationSeverity::Medium,
            condition: RuleCondition {
                expression: "is_audit_log && age_days > retention_period".to_string(),
                available_variables: vec!["is_audit_log".to_string(), "age_days".to_string(), "retention_period".to_string()],
                logical_operator: LogicalOperator::And,
                sub_conditions: Vec::new(),
            },
            enforcement_action: EnforcementAction::GenerateAlert,
            evidence_required: vec![
                crate::actors::nemesis::compliance::EvidenceType::LogFile,
                crate::actors::nemesis::compliance::EvidenceType::SystemMetric,
            ],
            exceptions: Vec::new(),
            status: RuleStatus::Active,
            last_modified: Utc::now(),
            metrics: RuleMetrics::default(),
        });
        
        // Regla de consentimiento (GDPR)
        rules.push(LegalRule {
            rule_id: "consent_001".to_string(),
            name: "Explicit Consent Required".to_string(),
            description: "Consentimiento explícito requerido para procesamiento de datos personales".to_string(),
            standard: crate::actors::nemesis::compliance::RegulatoryStandard::GDPR,
            rule_type: RuleType::ConsentPrivacy,
            severity: crate::actors::nemesis::compliance::ViolationSeverity::High,
            condition: RuleCondition {
                expression: "is_personal_processing && !has_consent".to_string(),
                available_variables: vec!["is_personal_processing".to_string(), "has_consent".to_string()],
                logical_operator: LogicalOperator::And,
                sub_conditions: Vec::new(),
            },
            enforcement_action: EnforcementAction::BlockOperation,
            evidence_required: vec![
                crate::actors::nemesis::compliance::EvidenceType::UserAction,
                crate::actors::nemesis::compliance::EvidenceType::SystemLog,
            ],
            exceptions: Vec::new(),
            status: RuleStatus::Active,
            last_modified: Utc::now(),
            metrics: RuleMetrics::default(),
        });
        
        rules
    }
    
    /// Evalúa una condición de regla
    async fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<(bool, f64, Vec<String>), ActorError> {
        // Evaluar sub-condiciones primero si existen
        if !condition.sub_conditions.is_empty() {
            let sub_results: Vec<_> = condition.sub_conditions.iter()
                .map(|sub| self.evaluate_condition(sub, context))
                .collect::<Result<_, _>>()?;
            
            let (compliant, confidence, details): Vec<_> = sub_results.iter()
                .map(|(c, conf, det)| (c, conf, det.clone()))
                .collect();
            
            let final_result = match condition.logical_operator {
                LogicalOperator::And => compliant.iter().all(|c| *c),
                LogicalOperator::Or => compliant.iter().any(|c| *c),
                LogicalOperator::Not => !compliant.first().unwrap_or(true),
                LogicalOperator::Xor => compliant.iter().filter(|c| *c).count() == 1,
            };
            
            let avg_confidence = confidence.iter().sum::<f64>() / confidence.len() as f64;
            let all_details: Vec<_> = details.into_iter().flatten().collect();
            
            return Ok((final_result, avg_confidence, all_details));
        }
        
        // Evaluar expresión principal
        let (result, confidence) = self.evaluate_expression(&condition.expression, context).await?;
        let details = vec![format!("Expression: {}", condition.expression)];
        
        Ok((result, confidence, details))
    }
    
    /// Evalúa una expresión booleana
    async fn evaluate_expression(
        &self,
        expression: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<(bool, f64), ActorError> {
        // Implementación simplificada de evaluación de expresiones
        // En una implementación real, esto usaría un motor de expresiones más robusto
        
        for (variable, value) in context {
            if let Ok(value_str) = serde_json::to_string(value) {
                if expression.contains(&format!("!{}", variable)) {
                    // Verificación booleana simple
                    if value_str == "true" {
                        return Ok((true, 0.9));
                    } else if value_str == "false" {
                        return Ok((false, 0.9));
                    }
                }
            }
        }
        
        // Implementación básica para demostración
        Ok((false, 0.5))
    }
    
    /// Convierte JSON a HashMap
    fn json_to_hashmap(&self, json: &serde_json::Value) -> HashMap<String, serde_json::Value> {
        match json {
            serde_json::Value::Object(map) => {
                map.into_iter().collect()
            },
            _ => HashMap::new(),
        }
    }
    
    /// Genera descripción del resultado
    async fn generate_result_description(&self, result: &RuleResult, details: &[String]) -> String {
        match result {
            RuleResult::Compliant => {
                format!("Rule evaluation: Compliant. Details: {}", details.join(", "))
            },
            RuleResult::Violated => {
                format!("Rule violation detected. Details: {}", details.join(", "))
            },
            RuleResult::NotApplicable => {
                "Rule not applicable to current context".to_string()
            },
            RuleResult::Indeterminate => {
                format!("Rule evaluation indeterminate. Details: {}", details.join(", "))
            },
        }
    }
    
    /// Calcula puntuación de cumplimiento
    fn calculate_compliance_score(&self, rule: &LegalRule, compliant: bool, confidence: f64) -> f64 {
        let base_score = if compliant { 100.0 } else { 0.0 };
        
        // Ajustar por severidad de la regla
        let severity_factor = match rule.severity {
            crate::actors::nemesis::compliance::ViolationSeverity::Info => 0.95,
            crate::actors::nemesis::compliance::ViolationSeverity::Low => 0.9,
            crate::actors::nemesis::compliance::ViolationSeverity::Medium => 0.8,
            crate::actors::nemesis::compliance::ViolationSeverity::High => 0.7,
            crate::actors::nemesis::compliance::ViolationSeverity::Critical => 0.6,
        };
        
        base_score * confidence * severity_factor
    }
    
    /// Recopila evidencia para la evaluación
    async fn collect_evidence(&self, rule: &LegalRule, context: &HashMap<String, serde_json::Value>) -> Vec<String> {
        let mut evidence = Vec::new();
        
        // Evidencia basada en las variables del contexto
        for (var_name, value) in context {
            if rule.evidence_required.iter().any(|e| match e {
                crate::actors::nemesis::compliance::EvidenceType::SystemMetric => var_name.contains("metric") || var_name.contains("performance"),
                crate::actors::nemesis::compliance::EvidenceType::LogRecord => var_name.contains("log") || var_name.contains("record"),
                crate::actors::nemesis::compliance::EvidenceType::UserAction => var_name.contains("user") || var_name.contains("action"),
                crate::actors::nemesis::compliance::EvidenceType::DatabaseRecord => var_name.contains("database") || var_name.contains("record"),
                crate::actors::nemesis::compliance::EvidenceType::ConfigurationFile => var_name.contains("config") || var_name.contains("setting"),
                _ => false,
            }) {
                if let Ok(value_str) = serde_json::to_string(value) {
                    evidence.push(format!("{} = {}", var_name, value_str));
                }
            }
        }
        
        // Agregar timestamp
        evidence.push(format!("evaluated_at = {}", Utc::now().to_rfc3339()));
        
        evidence
    }
    
    /// Actualiza las métricas de una regla
    async fn update_rule_metrics(&self, rule: &LegalRule, evaluation: &RuleEvaluation) {
        let mut metrics = self.metrics.write().await;
        
        // Esta función necesitaría acceso mutable a las métricas de la regla
        // Por ahora, actualizamos métricas globales del motor
        
        metrics.evaluations_count += 1;
        
        if evaluation.result == RuleResult::Compliant {
            metrics.compliance_rate = ((metrics.evaluations_count - 1) as f64 * metrics.compliance_rate + 1.0) / metrics.evaluations_count as f64;
        }
        
        metrics.last_evaluation = Some(Utc::now());
    }
    
    /// Obtiene métricas del motor de reglas
    pub async fn get_metrics(&self) -> RuleMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Agrega una nueva regla
    pub async fn add_rule(&self, rule: LegalRule) -> Result<(), ActorError> {
        let mut rules = self.active_rules.write().await;
        rules.push(rule);
        info!("⚖️ Regla agregada: {} ({})", rule.name, rule.rule_id);
        Ok(())
    }
    
    /// Elimina una regla por ID
    pub async fn remove_rule(&self, rule_id: &str) -> Result<(), ActorError> {
        let mut rules = self.active_rules.write().await;
        rules.retain(|r| r.rule_id != rule_id);
        info!("⚖️ Regla eliminada: {}", rule_id);
        Ok(())
    }
}

impl Default for CompliancePolicy {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            applicable_standards: vec![
                crate::actors::nemesis::compliance::RegulatoryStandard::HIPAA,
                crate::actors::nemesis::compliance::RegulatoryStandard::GDPR,
                crate::actors::nemesis::compliance::RegulatoryStandard::SOC2,
                crate::actors::nemesis::compliance::RegulatoryStandard::ISO27001,
            ],
            enforcement_levels: vec![
                EnforcementLevel::LogOnly,
                EnforcementLevel::Alert,
                EnforcementLevel::BlockAccess,
                EnforcementLevel::Escalate,
            ],
            exceptions: Vec::new(),
            review_required: true,
        }
    }
}

impl Default for RuleMetrics {
    fn default() -> Self {
        Self {
            evaluations_count: 0,
            violations_count: 0,
            compliance_rate: 1.0,
            average_evaluation_time_ms: 0.0,
            last_evaluation: None,
        }
    }
}