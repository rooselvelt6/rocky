// src/actors/nemesis/audit.rs
// OLYMPUS v15 - Audit: Sistema de Auditoría y Trazabilidad para Némesis

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::ActorError;
use crate::actors::nemesis::{
    compliance::{ComplianceAudit, ComplianceViolation, EvidenceType, RegulatoryStandard, ComplianceLevel, ViolationSeverity},
    rules::LegalRule,
};
use tracing::info;

/// Estado de una sesión de auditoría
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSessionStatus {
    /// Activa
    Active,
    /// Completada exitosamente
    Completed,
    /// Cancelada
    Cancelled,
    /// Fallida
    Failed,
    /// En pausa
    Paused,
    /// Requiere revisión
    UnderReview,
}

/// Consulta de búsqueda en auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSearchQuery {
    /// Término de búsqueda
    pub search_term: Option<String>,
    /// Filtro por tipo de evento
    pub event_types: Vec<AuditEventType>,
    /// Filtro por severidad
    pub severities: Vec<AuditSeverity>,
    /// Rango de fechas
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filtro por actor
    pub actors: Vec<String>,
    /// Límite de resultados
    pub limit: Option<usize>,
    /// Offset para paginación
    pub offset: Option<usize>,
}

/// Errores de eventos de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventError {
    /// Error de serialización
    SerializationError(String),
    /// Error de escritura
    WriteError(String),
    /// Error de lectura
    ReadError(String),
    /// Error de validación
    ValidationError(String),
    /// Error de base de datos
    DatabaseError(String),
    /// Error de archivo
    FileError(String),
    /// Error de permisos
    PermissionError(String),
    /// Error desconocido
    Unknown(String),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::SerializationError(msg) => write!(f, "Serialización: {}", msg),
            EventError::WriteError(msg) => write!(f, "Escritura: {}", msg),
            EventError::ReadError(msg) => write!(f, "Lectura: {}", msg),
            EventError::ValidationError(msg) => write!(f, "Validación: {}", msg),
            EventError::DatabaseError(msg) => write!(f, "Base de datos: {}", msg),
            EventError::FileError(msg) => write!(f, "Archivo: {}", msg),
            EventError::PermissionError(msg) => write!(f, "Permisos: {}", msg),
            EventError::Unknown(msg) => write!(f, "Desconocido: {}", msg),
        }
    }
}

impl std::error::Error for EventError {}

/// Log de auditoría individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// ID único del log
    pub log_id: String,
    /// Timestamp del log
    pub timestamp: DateTime<Utc>,
    /// Nivel del log
    pub level: AuditLogLevel,
    /// Formato del log
    pub format: AuditLogFormat,
    /// Contenido del log
    pub content: String,
    /// Componente que generó el log
    pub component: String,
    /// Metadatos adicionales
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Métricas de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    /// Total de logs procesados
    pub total_logs: u64,
    /// Logs por nivel
    pub logs_by_level: HashMap<String, u64>,
    /// Logs por componente
    pub logs_by_component: HashMap<String, u64>,
    /// Espacio total utilizado (bytes)
    pub storage_used_bytes: u64,
    /// Timestamp de última actualización
    pub last_updated: DateTime<Utc>,
}

impl Default for AuditMetrics {
    fn default() -> Self {
        Self {
            total_logs: 0,
            logs_by_level: HashMap::new(),
            logs_by_component: HashMap::new(),
            storage_used_bytes: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Logger de auditoría y trazabilidad
#[derive(Debug, Clone)]
pub struct AuditLogger {
    /// Configuración del logger
    config: Arc<RwLock<AuditConfig>>,
    /// Trail de auditoría activa
    audit_trail: Arc<RwLock<AuditTrail>>,
    /// Logs de auditoría
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
    /// Archivos de evidencia
    evidence_files: Arc<RwLock<HashMap<String, EvidenceFile>>>,
    /// Métricas de auditoría
    metrics: Arc<RwLock<AuditMetrics>>,
}

/// Trail de auditoría - Registro secuencial de eventos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Eventos registrados
    pub events: Vec<AuditEvent>,
    /// Estados actuales de auditoría
    pub active_audits: HashMap<String, AuditSession>,
    /// Estadísticas globales
    pub global_stats: AuditStatistics,
    /// Configuración actual
    pub current_config: AuditConfig,
}

/// Evento de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// ID único del evento
    pub event_id: String,
    /// Timestamp del evento
    pub timestamp: DateTime<Utc>,
    /// Tipo de evento
    pub event_type: AuditEventType,
    /// Actor o sistema involucrado
    pub actor: Option<String>,
    /// Requisitos regulatorios afectados
    pub affected_requirements: Vec<String>,
    /// Severidad del evento
    pub severity: AuditSeverity,
    /// Mensaje descriptivo
    pub message: String,
    /// Contexto adicional
    pub context: HashMap<String, serde_json::Value>,
    /// Metadatos técnicos
    pub technical_metadata: AuditTechnicalMetadata,
}

/// Sesión de auditoría activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSession {
    /// ID de la sesión
    pub session_id: String,
    /// Estándar regulatorio
    pub standard: RegulatoryStandard,
    /// Objetivo de la auditoría
    pub target: String,
    /// Estado de la sesión
    pub status: AuditSessionStatus,
    /// Inicio de la sesión
    pub start_time: DateTime<Utc>,
    /// Fin de la sesión
    pub end_time: Option<DateTime<Utc>>,
    /// Evaluaciones realizadas
    pub evaluations: Vec<ComplianceEvaluation>,
    /// Resultado de la sesión
    pub session_result: Option<AuditSessionResult>,
    /// Investigador asignado
    pub investigator: Option<String>,
    /// Notas de la sesión
    pub notes: Vec<String>,
}

/// Evaluación de cumplimiento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvaluation {
    /// ID de la evaluación
    pub evaluation_id: String,
    /// Regla evaluada
    pub rule: LegalRule,
    /// Resultado de la evaluación
    pub result: ComplianceEvaluationResult,
    /// Confianza en el resultado
    pub confidence: f64,
    /// Timestamp de evaluación
    pub timestamp: DateTime<Utc>,
    /// Detalles adicionales
    pub details: String,
}

/// Resultado de evaluación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceEvaluationResult {
    /// Cumple con la regla
    Compliant,
    /// Violación detectada
    Violated,
    /// Regla no aplicable
    NotApplicable,
}

/// Estadísticas de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total de eventos registrados
    pub total_events: u64,
    /// Total de sesiones completadas
    pub completed_sessions: u64,
    /// Sesiones activas actualmente
    pub active_sessions: u64,
    /// Violaciones detectadas en total
    pub total_violations: u64,
    /// Violaciones críticas
    pub critical_violations: u64,
    /// Promedio de tiempo de sesión (minutos)
    pub average_session_time_minutes: f64,
    /// Siguiente sesión programada
    pub next_scheduled_session: Option<DateTime<Utc>>,
    /// Métricas por estándar
    pub metrics_by_standard: HashMap<String, StandardMetrics>,
}

/// Métricas por estándar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMetrics {
    /// Total de auditorías para este estándar
    pub total_audits: u64,
    /// Tasa de cumplimiento
    pub compliance_rate: f64,
    /// Violaciones por severidad
    violations_by_severity: HashMap<String, u32>,
    /// Última auditoría
    pub last_audit: Option<DateTime<Utc>>,
}

/// Archivo de evidencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFile {
    /// ID único
    pub file_id: String,
    /// Nombre del archivo
    pub file_name: String,
    /// Ruta del archivo
    pub file_path: String,
    /// Tipo de archivo
    pub file_type: EvidenceType,
    /// Hash del archivo para integridad
    pub file_hash: String,
    /// Tamaño del archivo (bytes)
    pub file_size: u64,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de última modificación
    pub last_modified: DateTime<Utc>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuración del sistema de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Nivel de logging
    pub log_level: AuditLogLevel,
    /// Formato de logs
    pub log_format: AuditLogFormat,
    /// Directorio de almacenamiento de logs
    pub log_directory: String,
    /// Retención de logs (días)
    pub log_retention_days: u64,
    /// Rotación de logs
    pub log_rotation: LogRotation,
    /// Compresión de logs antiguos
    pub compress_old_logs: bool,
    /// Cifrado de archivos de evidencia
    pub encrypt_evidence: bool,
    /// Almacenamiento en blockchain para trazabilidad inmutable
    pub blockchain_storage: bool,
    /// Alertas de auditoría
    pub alert_settings: AuditAlertSettings,
    /// Trazabilidad completa
    pub full_traceability: bool,
    /// Intervalo de análisis automático
    pub analysis_interval_hours: u64,
}

/// Niveles de logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Formatos de log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogFormat {
    JSON,
    Structured,
    Syslog,
    ELKStack,
}

/// Configuración de rotación de logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Tamaño máximo de archivo (MB)
    pub max_file_size_mb: u64,
    /// Número máximo de archivos
    pub max_files: u64,
    /// Frecuencia de rotación
    pub rotation_frequency: RotationFrequency,
}

/// Frecuencia de rotación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Configuración de alertas de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAlertSettings {
    /// Alertar violaciones críticas
    pub alert_critical_violations: bool,
    /// Alertar violaciones altas
    pub alert_high_violations: bool,
    /// Alertar violaciones medias
    pub alert_medium_violations: bool,
    /// Canales de alerta
    pub alert_channels: Vec<String>,
    /// Umbral para alertas masivas
    pub bulk_alert_threshold: u32,
}

/// Metadatos técnicos del evento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTechnicalMetadata {
    /// Dirección IP del cliente
    pub client_ip: Option<String>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// Referencia HTTP
    pub http_reference: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Componente del sistema
    pub component: String,
    /// Versión del software
    pub software_version: Option<String>,
    /// Plataforma
    pub platform: Option<String>,
}

/// Resultado de sesión de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSessionResult {
    /// ID de la sesión
    pub session_id: String,
    /// Resultado general
    pub overall_result: AuditSessionResultType,
    /// Puntuación de la sesión
    pub session_score: u8,
    /// Nivel de cumplimiento alcanzado
    pub compliance_level: ComplianceLevel,
    /// Resumen de violaciones
    pub violations_summary: ViolationsSummary,
    /// Recomendaciones
    pub recommendations: Vec<String>,
    /// Tiempo total de la sesión
    pub total_duration_minutes: u32,
}

/// Tipo de resultado de sesión
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSessionResultType {
    /// Auditoría exitosa
    Successful,
    /// Falló la auditoría
    Failed,
    /// Auditoría incompleta
    Incomplete,
    /// Auditoría cancelada
    Cancelled,
}

/// Resumen de violaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationsSummary {
    /// Total de violaciones
    pub total_count: u32,
    /// Violaciones críticas
    pub critical_count: u32,
    /// Violaciones altas
    pub high_count: u32,
    /// Violaciones medias
    pub medium_count: u32,
    /// Violaciones bajas
    pub low_count: u32,
    /// Violaciones por tipo
    violations_by_type: HashMap<String, u32>,
}

/// Severidad de auditoría
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Tipos de eventos de auditoría
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Inicio de sesión de auditoría
    SessionStarted,
    /// Fin de sesión de auditoría
    SessionEnded,
    /// Evaluación de regla
    RuleEvaluation,
    /// Violación detectada
    ViolationDetected,
    /// Acción correctiva
    CorrectiveAction,
    /// Evidencia agregada
    EvidenceAdded,
    /// Decisión tomada
    DecisionMade,
    /// Sistema modificado
    SystemModified,
    /// Cambio de configuración
    ConfigurationChanged,
    /// Inicio de exportación
    ExportStarted,
    /// Fin de exportación
    ExportCompleted,
    /// Sistema iniciado
    SystemStarted,
    /// Sistema detenido
    SystemStopped,
}

impl AuditLogger {
    /// Crea una nueva instancia del logger de auditoría
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AuditConfig::default())),
            audit_trail: Arc::new(RwLock::new(AuditTrail {
                events: Vec::new(),
                active_audits: HashMap::new(),
                global_stats: AuditStatistics::default(),
                current_config: AuditConfig::default(),
            })),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            evidence_files: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(AuditMetrics::default())),
        }
    }
    
    /// Inicializa el sistema de auditoría
    pub async fn initialize(&self) -> Result<(), ActorError> {
        // Crear directorios necesarios
        let config = self.config.read().await;
        tokio::fs::create_dir_all(&config.log_directory).await
            .map_err(|e| ActorError::Unknown {
                god: crate::actors::GodName::Nemesis,
                message: format!("Error creando directorio de logs: {}", e),
            })?;
        
        info!("📋 AuditLogger inicializado en {}", config.log_directory);
        Ok(())
    }
    
    /// Inicia una nueva sesión de auditoría
    pub async fn start_audit_session(
        &self,
        standard: RegulatoryStandard,
        target: String,
        investigator: Option<String>,
    ) -> Result<String, ActorError> {
        let session_id = Uuid::new_v4().to_string();
        
        let session = AuditSession {
            session_id: session_id.clone(),
            standard: standard.clone(),
            target: target.clone(),
            status: AuditSessionStatus::Active,
            start_time: Utc::now(),
            end_time: None,
            evaluations: Vec::new(),
            session_result: None,
            investigator,
            notes: Vec::new(),
        };
        
        {
            let mut trail = self.audit_trail.write().await;
            let mut active_audits = trail.active_audits;
            active_audits.insert(session_id.clone(), session);
            
            // Registrar evento de inicio de sesión
            trail.events.push(AuditEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::SessionStarted,
                actor: Some("Nemesis".to_string()),
                affected_requirements: vec![format!("{:?}", standard)],
                severity: AuditSeverity::Info,
                message: format!("Iniciando sesión de auditoría para {} contra {}", target, standard),
                context: std::collections::HashMap::from([
                    ("investigator".to_string(), serde_json::Value::Null),
                    ("session_id".to_string(), serde_json::Value::String(session_id.clone())),
                ]),
                technical_metadata: AuditTechnicalMetadata::default(),
            });
        }
        
        info!("📋 Sesión de auditoría iniciada: {} para {}", session_id, target);
        Ok(session_id)
    }
    
    /// Finaliza una sesión de auditoría
    pub async fn end_audit_session(
        &self,
        session_id: &str,
        result: AuditSessionResult,
        notes: Vec<String>,
    ) -> Result<(), ActorError> {
        let config = self.config.read().await;
        
        {
            let mut trail = self.audit_trail.write().await;
            
            // Actualizar estado de la sesión
            if let Some(session) = trail.active_audits.get_mut(session_id) {
                session.status = AuditSessionStatus::Completed;
                session.end_time = Some(Utc::now());
                session.session_result = Some(result);
                session.notes = notes;
                
                // Registrar evento de fin de sesión
                trail.events.push(AuditEvent {
                    event_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    event_type: AuditEventType::SessionEnded,
                    actor: Some("Nemesis".to_string()),
                    affected_requirements: vec![format!("{:?}", session.standard)],
                    severity: match result.overall_result {
                        AuditSessionResultType::Successful => AuditSeverity::Info,
                        AuditSessionResultType::Failed => AuditSeverity::Error,
                        _ => AuditSeverity::Warning,
                    },
                    message: format!("Sesión de auditoría finalizada: {}", result.session_id),
                    context: std::collections::HashMap::from([
                        ("result_type".to_string(), serde_json::to_value(&result.overall_result)),
                        ("session_score".to_string(), serde_json::Value::Number(result.session_score.into())),
                        ("compliance_level".to_string(), serde_json::to_value(&result.compliance_level)),
                    ]),
                    technical_metadata: AuditTechnicalMetadata::default(),
                });
                
                // Mover a historial de sesiones completadas
                trail.completed_sessions.push(session.clone());
                trail.active_audits.remove(session_id);
            }
            
            // Actualizar estadísticas globales
            trail.global_stats.completed_sessions += 1;
            trail.global_stats.total_violations += result.violations_summary.total_count;
            trail.global_stats.critical_violations += result.violations_summary.critical_count;
            
            info!("📋 Sesión de auditoría finalizada: {}", session_id);
        }
        
        // Generar reporte de auditoría
        self.generate_audit_report(&session_id).await?;
        
        Ok(())
    }
    
    /// Agrega un evento a la trail de auditoría
    pub async fn log_event(&self, event: AuditEvent) -> Result<(), ActorError> {
        {
            let mut trail = self.audit_trail.write().await;
            trail.events.push(event.clone());
            
            // Mantener tamaño razonable del trail
            if trail.events.len() > 10000 {
                trail.events.drain(0..5000); // Mantener últimos 5000 eventos
            }
        }
        
        Ok(())
    }
    
    /// Agrega una evaluación de regla
    pub async fn log_rule_evaluation(
        &self,
        session_id: &str,
        evaluation: &ComplianceEvaluation,
    ) -> Result<(), ActorError> {
        {
            let trail = self.audit_trail.write().await;
            
            // Agregar evaluación a la sesión
            if let Some(session) = trail.active_audits.get_mut(session_id) {
                session.evaluations.push(evaluation.clone());
            }
            
            // Registrar evento de evaluación
            self.log_event(AuditEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::RuleEvaluation,
                actor: Some("Nemesis".to_string()),
                affected_requirements: vec![format!("Regla: {}", evaluation.rule.name)],
                severity: match evaluation.result {
                    ComplianceEvaluationResult::Violated => AuditSeverity::Warning,
                    ComplianceEvaluationResult::Compliant => AuditSeverity::Info,
                    _ => AuditSeverity::Debug,
                },
                message: format!("Evaluación de regla {}: {}", evaluation.rule.name, evaluation.result),
                context: std::collections::HashMap::from([
                    ("rule_id".to_string(), serde_json::Value::String(evaluation.rule.rule_id.clone())),
                    ("result".to_string(), serde_json::to_value(&evaluation.result)),
                    ("confidence".to_string(), serde_json::Value::Number(evaluation.confidence)),
                    ("rule_name".to_string(), serde_json::Value::String(evaluation.rule.name.clone())),
                ]),
                technical_metadata: AuditTechnicalMetadata::default(),
            }).await?;
            
            Ok(())
        }
    }
    
    /// Registra una violación detectada
    pub async fn log_violation(
        &self,
        session_id: &str,
        violation: &ComplianceViolation,
    ) -> Result<(), ActorError> {
        {
            // Agregar violación a la sesión
            {
                let trail = self.audit_trail.write().await;
                if let Some(session) = trail.active_audits.get_mut(session_id) {
                    session.evaluations.push(ComplianceEvaluation {
                        evaluation_id: Uuid::new_v4().to_string(),
                        rule: LegalRule {
                            // Crear regla ficticia para la violación
                            rule_id: violation.violation_id.clone(),
                            name: format!("Violación de {}", violation.violation_type),
                            description: violation.description.clone(),
                            standard: RegulatoryStandard::HIPAA, // Por defecto
                            rule_type: crate::actors::nemesis::rules::RuleType::AccessControl,
                            severity: violation.severity,
                            condition: crate::actors::nemesis::rules::RuleCondition {
                                expression: "true".to_string(),
                                available_variables: vec![],
                                logical_operator: crate::actors::nemesis::rules::LogicalOperator::And,
                                sub_conditions: vec![],
                            },
                            enforcement_action: crate::actors::nemesis::rules::EnforcementAction::BlockAccess,
                            evidence_required: vec![],
                            exceptions: vec![],
                            status: crate::actors::nemesis::rules::RuleStatus::Active,
                            last_modified: Utc::now(),
                            metrics: crate::actors::nemesis::rules::RuleMetrics::default(),
                        },
                        result: ComplianceEvaluationResult::Violated,
                        confidence: 0.9,
                        timestamp: Utc::now(),
                        details: format!("Violación detectada: {}", violation.description),
                    });
                }
            }
            
            // Registrar evento de violación
            self.log_event(AuditEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::ViolationDetected,
                actor: Some("Nemesis".to_string()),
                affected_requirements: vec!["Security Requirement"],
                severity: violation.severity,
                message: format!("Violación detectada: {}", violation.description),
                context: std::collections::HashMap::from([
                    ("violation_id".to_string(), serde_json::Value::String(violation.violation_id.clone())),
                    ("violation_type".to_string(), serde_json::to_value(&violation.violation_type)),
                    ("severity".to_string(), serde_json::to_value(&violation.severity)),
                    ("corrective_actions".to_string(), serde_json::Value::Array(violation.corrective_actions.iter().map(|s| s.to_string()).collect())),
                ]),
                technical_metadata: AuditTechnicalMetadata::default(),
            }).await?;
            
            info!("📋 Violación registrada: {} (Severidad: {:?})", 
                   violation.violation_id, violation.severity);
            
            Ok(())
        }
    }
    
    /// Genera reporte de auditoría para una sesión
    pub async fn generate_audit_report(&self, session_id: &str) -> Result<(), ActorError> {
        let trail = self.audit_trail.read().await;
        
        if let Some(session) = trail.active_audits.get(session_id) {
            let config = self.config.read().await;
            
            // Generar reporte en formato JSON
            let report = serde_json::json!({
                "session": {
                    "session_id": session.session_id,
                    "standard": format!("{:?}", session.standard),
                    "target": session.target,
                    "investigator": session.investigator,
                    "status": session.status,
                    "start_time": session.start_time,
                    "end_time": session.end_time,
                    "duration_minutes": session.end_time.map_or(Utc::now())
                        .signed_duration_since(session.start_time).num_minutes() as u32,
                    "evaluations_count": session.evaluations.len(),
                    "result": session.session_result,
                    "notes": session.notes,
                },
                "events": trail.events.iter()
                    .filter(|e| {
                        e.actor.as_ref().map_or("Nemesis".to_string(), false) == "Nemesis" ||
                        (e.event_type == AuditEventType::SessionStarted || e.event_type == AuditEventType::SessionEnded)
                    })
                    .take_last(100) // Últimos 100 eventos
                    .collect(),
                "statistics": trail.global_stats,
                "generated_at": Utc::now(),
                "configuration": config,
            });
            
            // Guardar reporte en archivo
            let report_path = format!("{}/audit_report_{}.json", config.log_directory, session_id);
            tokio::fs::write(&report_path, serde_json::to_string_pretty(&report))
                .await
                .map_err(|e| ActorError::Unknown {
                    god: crate::actors::GodName::Nemesis,
                    message: format!("Error guardando reporte de auditoría: {}", e),
                })?;
            
            info!("📋 Reporte de auditoría generado: {}", report_path);
        }
        
        Ok(())
    }
    
    /// Busca eventos de auditoría
    pub async fn search_events(
        &self,
        query: &AuditSearchQuery,
    ) -> Result<Vec<AuditEvent>, ActorError> {
        let trail = self.audit_trail.read().await;
        
        let mut filtered_events = Vec::new();
        
        for event in &trail.events {
            let mut matches = true;
            
            // Filtrar por tipo de evento
            if let Some(event_types) = &query.event_types {
                if !event_types.contains(&event.event_type) {
                    matches = false;
                }
            }
            
            // Filtrar por severidad
            if let Some(min_severity) = &query.min_severity {
                if !self.severity_meets_requirement(&event.severity, min_severity) {
                    matches = false;
                }
            }
            
            // Filtrar por rango de tiempo
            if let (start_time, end_time) = query.time_range {
                if event.timestamp < *start_time || event.timestamp > *end_time {
                    matches = false;
                }
            }
            
            // Filtrar por actor
            if let Some(target_actor) = &query.target_actor {
                if event.actor.as_ref().map_or("", "") != *target_actor {
                    matches = false;
                }
            }
            
            // Filtrar por palabras clave
            if let Some(keywords) = &query.keywords {
                if !keywords.iter().any(|keyword| 
                    event.message.to_lowercase().contains(&keyword.to_lowercase()) ||
                    event.description.to_lowercase().contains(&keyword.to_lowercase())
                ) {
                    matches = false;
                }
            }
            
            if matches {
                filtered_events.push(event.clone());
            }
        }
        
        Ok(filtered_events)
    }
    
    /// Verifica si una severidad cumple con un mínimo requerido
    fn severity_meets_requirement(&self, severity: &AuditSeverity, minimum: &AuditSeverity) -> bool {
        match (severity, minimum) {
            (AuditSeverity::Info, AuditSeverity::Info) => true,
            (AuditSeverity::Warning, AuditSeverity::Info) => true,
            (AuditSeverity::Error, AuditSeverity::Info) => true,
            (AuditSeverity::Critical, AuditSeverity::Info) => false,
            (AuditSeverity::Info, AuditSeverity::Warning) => false,
            (AuditSeverity::Warning, AuditSeverity::Warning) => true,
            (AuditSeverity::Error, AuditSeverity::Error) => true,
            (AuditSeverity::Critical, AuditSeverity::Critical) => true,
        }
    }
    
    /// Obtiene estadísticas de auditoría
    pub async fn get_statistics(&self) -> AuditStatistics {
        let trail = self.audit_trail.read().await;
        trail.global_stats.clone()
    }
    
    /// Exporta el trail de auditoría
    pub async fn export_audit_trail(&self, format: AuditExportFormat) -> Result<String, ActorError> {
        let trail = self.audit_trail.read().await;
        
        match format {
            AuditExportFormat::JSON => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "audit_trail": trail,
                    "exported_at": Utc::now(),
                }))
            },
            AuditExportFormat::CSV => {
                self.export_to_csv(&trail).await
            },
            AuditExportFormat::Structured => {
                self.export_to_structured(&trail).await
            },
        }
    }
    
    /// Exporta a formato CSV
    async fn export_to_csv(&self, trail: &AuditTrail) -> Result<String, EventError> {
        let mut csv_content = String::new();
        
        // Encabezado CSV
        csv_content.push_str("event_id,timestamp,event_type,actor,severity,message,affected_requirements\n");
        
        // Datos de eventos
        for event in &trail.events {
            let requirements_str = event.affected_requirements.join(";");
            let timestamp_str = event.timestamp.to_rfc3339();
            
            csv_content.push_str(&format!(
                "{},{},\"{}\",\"{}\",\"{}\",{}\n",
                event.event_id,
                timestamp_str,
                format!("{:?}", event.event_type),
                event.actor.unwrap_or_default("Nemesis".to_string()),
                event.severity,
                csv_content.replace("\"", "\"\"").replace("\n", "\\n"),
            ));
        }
        
        Ok(csv_content)
    }
    
    /// Exporta a formato estructurado
    async fn export_to_structured(&self, trail: &AuditTrail) -> Result<String, ActorError> {
        serde_json::to_string_pretty(&trail)
    }
}

/// Formatos de exportación de auditoría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditExportFormat {
    JSON,
    CSV,
    Structured,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_level: AuditLogLevel::Info,
            log_format: AuditLogFormat::JSON,
            log_directory: "/var/log/audit".to_string(),
            log_retention_days: 365,
            log_rotation: LogRotation {
                max_file_size_mb: 100,
                max_files: 30,
                rotation_frequency: RotationFrequency::Daily,
            },
            compress_old_logs: true,
            encrypt_evidence: true,
            blockchain_storage: false,
            alert_settings: AuditAlertSettings::default(),
            full_traceability: true,
            analysis_interval_hours: 24,
        }
    }
}

impl Default for AuditStatistics {
    fn default() -> Self {
        Self {
            total_events: 0,
            completed_sessions: 0,
            active_sessions: 0,
            total_violations: 0,
            critical_violations: 0,
            average_session_time_minutes: 0.0,
            next_scheduled_session: None,
            metrics_by_standard: HashMap::new(),
        }
    }
}

impl Default for AuditAlertSettings {
    fn default() -> Self {
        Self {
            alert_critical_violations: true,
            alert_high_violations: true,
            alert_medium_violations: false,
            alert_low_violations: false,
            alert_channels: vec!["email".to_string(), "slack".to_string()],
            bulk_alert_threshold: 10,
        }
    }
}

impl Default for AuditTechnicalMetadata {
    fn default() -> Self {
        Self {
            client_ip: None,
            user_agent: None,
            http_reference: None,
            session_id: None,
            request_id: None,
            component: "Nemesis".to_string(),
            software_version: Some("v15.0.0".to_string()),
            platform: Some("Rust".to_string()),
        }
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            active_audits: HashMap::new(),
            global_stats: AuditStatistics::default(),
            current_config: AuditConfig::default(),
        }
    }
}

impl Default for AuditSessionResult {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            overall_result: AuditSessionResultType::Incomplete,
            session_score: 0,
            compliance_level: crate::actors::nemesis::compliance::ComplianceLevel::StandardCompliant,
            violations_summary: ViolationsSummary::default(),
            recommendations: Vec::new(),
            total_duration_minutes: 0,
        }
    }
}

impl Default for ViolationsSummary {
    fn default() -> Self {
        Self {
            total_count: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            violations_by_type: HashMap::new(),
        }
    }
}