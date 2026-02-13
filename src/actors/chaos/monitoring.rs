// src/actors/chaos/monitoring.rs
// OLYMPUS v15 - Sistema de Monitoreo de Impacto y Recuperación para Chaos

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use tracing::{info, warn, error};

use crate::actors::GodName;
use crate::errors::ActorError;

/// Estado del impacto de un experimento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactState {
    /// ID del experimento monitoreado
    pub experiment_id: String,
    /// Sistemas afectados
    pub affected_systems: Vec<GodName>,
    /// Nivel de impacto actual
    pub impact_level: ImpactLevel,
    /// Timestamp de inicio del monitoreo
    pub start_time: DateTime<Utc>,
    /// Última actualización
    pub last_update: DateTime<Utc>,
    /// Métricas de impacto
    pub metrics: HashMap<String, f64>,
    /// Señales de alerta
    pub alerts: Vec<Alert>,
    /// Si está actualmente en recuperación
    pub is_recovering: bool,
    /// Timestamp de inicio de recuperación
    pub recovery_start: Option<DateTime<Utc>>,
    /// Duración de la recuperación (si aplica)
    pub recovery_duration: Option<Duration>,
}

/// Niveles de impacto
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Sin impacto detectado
    None,
    /// Impacto mínimo (latencia leve)
    Minimal,
    /// Impacto bajo (degradación leve)
    Low,
    /// Impacto moderado
    Medium,
    /// Impacto alto (fallos significativos)
    High,
    /// Impacto crítico (sistema parcialmente abajo)
    Critical,
    /// Impacto catastrófico (sistema completamente abajo)
    Catastrophic,
}

impl ImpactLevel {
    /// Obtiene el valor numérico para comparación
    pub fn as_number(&self) -> u8 {
        match self {
            ImpactLevel::None => 0,
            ImpactLevel::Minimal => 1,
            ImpactLevel::Low => 2,
            ImpactLevel::Medium => 3,
            ImpactLevel::High => 4,
            ImpactLevel::Critical => 5,
            ImpactLevel::Catastrophic => 6,
        }
    }
    
    /// Obtiene la descripción del nivel
    pub fn description(&self) -> &'static str {
        match self {
            ImpactLevel::None => "Sin impacto",
            ImpactLevel::Minimal => "Impacto mínimo",
            ImpactLevel::Low => "Impacto bajo",
            ImpactLevel::Medium => "Impacto moderado",
            ImpactLevel::High => "Impacto alto",
            ImpactLevel::Critical => "Impacto crítico",
            ImpactLevel::Catastrophic => "Impacto catastrófico",
        }
    }
    
    /// Obtiene el color para visualización
    pub fn color(&self) -> &'static str {
        match self {
            ImpactLevel::None => "⚪",
            ImpactLevel::Minimal => "🟢",
            ImpactLevel::Low => "🟡",
            ImpactLevel::Medium => "🟠",
            ImpactLevel::High => "🟠",
            ImpactLevel::Critical => "🔴",
            ImpactLevel::Catastrophic => "🔴",
        }
    }
    
    /// Obtiene el umbral de alerta
    pub fn alert_threshold(&self) -> u64 {
        match self {
            ImpactLevel::None => u64::MAX,
            ImpactLevel::Minimal => 120000, // 2 minutos
            ImpactLevel::Low => 60000,     // 1 minuto
            ImpactLevel::Medium => 30000,    // 30 segundos
            ImpactLevel::High => 10000,      // 10 segundos
            ImpactLevel::Critical => 5000,     // 5 segundos
            ImpactLevel::Catastrophic => 1000,  // 1 segundo
        }
    }
}

/// Alerta generada por el monitoreo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// ID único de la alerta
    pub id: String,
    /// Tipo de alerta
    pub alert_type: AlertType,
    /// Severidad de la alerta
    pub severity: AlertSeverity,
    /// Mensaje descriptivo
    pub message: String,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    /// Si fue acknowledeada
    pub acknowledged: bool,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
}

/// Tipos de alertas
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    /// Detección de alto impacto
    HighImpactDetected,
    /// Tiempo de recuperación excedido
    RecoveryTimeout,
    /// Degradación continua
    ContinuousDegradation,
    /// Recuperación fallida
    RecoveryFailed,
    /// Impacto inesperado
    UnexpectedImpact,
    /// Sistema no responde
    SystemUnresponsive,
    /// Recuperación automática iniciada
    AutoRecoveryStarted,
}

/// Severidad de las alertas
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    /// Obtiene el valor numérico
    pub fn as_number(&self) -> u8 {
        match self {
            AlertSeverity::Info => 1,
            AlertSeverity::Warning => 2,
            AlertSeverity::Error => 3,
            AlertSeverity::Critical => 4,
        }
    }
    
    /// Obtiene el emoji para visualización
    pub fn emoji(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "❌",
            AlertSeverity::Critical => "🚨",
        }
    }
}

/// Métricas de impacto detalladas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMetrics {
    /// Latencia promedio (ms)
    pub average_latency_ms: f64,
    /// Porcentaje de errores
    pub error_rate: f64,
    /// Porcentaje de paquetes perdidos
    pub packet_loss_rate: f64,
    /// Uso de CPU (%)
    pub cpu_usage: f64,
    /// Uso de memoria (%)
    pub memory_usage: f64,
    /// Tasa de éxito de peticiones
    pub request_success_rate: f64,
    /// Tiempo de actividad (segundos)
    pub uptime: f64,
    /// Score general de impacto (0-100)
    pub impact_score: u8,
    /// Timestamp de última medición
    pub last_measured: DateTime<Utc>,
}

impl Default for ImpactMetrics {
    fn default() -> Self {
        Self {
            average_latency_ms: 0.0,
            error_rate: 0.0,
            packet_loss_rate: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            request_success_rate: 100.0,
            uptime: 100.0,
            impact_score: 0,
            last_measured: Utc::now(),
        }
    }
}

impl ImpactMetrics {
    /// Calcula el score de impacto general
    pub fn calculate_impact_score(&self) -> u8 {
        let latency_score = if self.average_latency_ms > 1000.0 { 30 } 
                            else if self.average_latency_ms > 500.0 { 20 }
                            else if self.average_latency_ms > 100.0 { 10 }
                            else { 0 };
        
        let error_score = (self.error_rate * 25.0) as u8;
        let packet_loss_score = (self.packet_loss_rate * 25.0) as u8;
        let resource_score = ((self.cpu_usage.max(self.memory_usage)) / 100.0 * 20.0) as u8;
        
        (latency_score + error_score + packet_loss_score + resource_score).min(100)
    }
    
    /// Obtiene el nivel de impacto basado en las métricas
    pub fn get_impact_level(&self) -> ImpactLevel {
        let score = self.calculate_impact_score();
        
        if score >= 80 { ImpactLevel::Catastrophic }
        else if score >= 60 { ImpactLevel::Critical }
        else if score >= 40 { ImpactLevel::High }
        else if score >= 25 { ImpactLevel::Medium }
        else if score >= 10 { ImpactLevel::Low }
        else if score > 0 { ImpactLevel::Minimal }
        else { ImpactLevel::None }
    }
}

/// Analizador de impacto
#[derive(Debug, Clone)]
pub struct ImpactAnalyzer {
    /// Configuración del analizador
    config: Arc<RwLock<ImpactAnalyzerConfig>>,
    /// Estados de monitoreo activos
    active_impacts: Arc<RwLock<HashMap<String, ImpactState>>>,
    /// Historial de análisis
    analysis_history: Arc<RwLock<Vec<ImpactAnalysis>>>,
}

/// Configuración del analizador de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalyzerConfig {
    /// Intervalo de monitoreo (milisegundos)
    pub monitoring_interval_ms: u64,
    /// Umbral de latencia crítica (ms)
    pub critical_latency_threshold: f64,
    /// Umbral de error crítico (%)
    pub critical_error_threshold: f64,
    /// Umbral de uso de recursos crítico (%)
    pub critical_resource_threshold: f64,
    /// Tiempo máximo de recuperación (segundos)
    pub max_recovery_time: u64,
    /// Habilitar detección automática de anomalías
    pub enable_anomaly_detection: bool,
    /// Sensibilidad de detección de anomalías (0.0-1.0)
    pub anomaly_sensitivity: f64,
}

impl Default for ImpactAnalyzerConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_ms: 1000, // 1 segundo
            critical_latency_threshold: 2000.0,
            critical_error_threshold: 0.1, // 10%
            critical_resource_threshold: 90.0, // 90%
            max_recovery_time: 300, // 5 minutos
            enable_anomaly_detection: true,
            anomaly_sensitivity: 0.5,
        }
    }
}

/// Resultado de un análisis de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// ID del análisis
    pub id: String,
    /// ID del experimento analizado
    pub experiment_id: String,
    /// Timestamp del análisis
    pub analyzed_at: DateTime<Utc>,
    /// Nivel de impacto detectado
    pub impact_level: ImpactLevel,
    /// Métricas analizadas
    pub metrics: ImpactMetrics,
    /// Alertas generadas
    pub alerts: Vec<Alert>,
    /// Recomendaciones
    pub recommendations: Vec<String>,
    /// Duración del análisis (segundos)
    pub analysis_duration_seconds: u64,
}

impl ImpactAnalyzer {
    /// Crea un nuevo analizador de impacto
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ImpactAnalyzerConfig::default())),
            active_impacts: Arc::new(RwLock::new(HashMap::new())),
            analysis_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Inicia el monitoreo de un experimento
    pub async fn start_monitoring(
        &self,
        experiment_id: String,
        affected_systems: Vec<GodName>,
    ) -> Result<(), ActorError> {
        let impact_state = ImpactState {
            experiment_id: experiment_id.clone(),
            affected_systems: affected_systems.clone(),
            impact_level: ImpactLevel::None,
            start_time: Utc::now(),
            last_update: Utc::now(),
            metrics: HashMap::new(),
            alerts: Vec::new(),
            is_recovering: false,
            recovery_start: None,
            recovery_duration: None,
        };
        
        {
            let mut active_impacts = self.active_impacts.write().await;
            active_impacts.insert(experiment_id.clone(), impact_state);
        }
        
        info!("📊 Iniciando monitoreo de impacto para experimento: {}", experiment_id);
        Ok(())
    }
    
    /// Detiene el monitoreo de un experimento
    pub async fn stop_monitoring(
        &self,
        experiment_id: &str,
    ) -> Result<ImpactAnalysis, ActorError> {
        let start_time = {
            let mut active_impacts = self.active_impacts.write().await;
            if let Some(impact_state) = active_impacts.remove(experiment_id) {
                impact_state.start_time
            } else {
                return Err(ActorError::Unknown {
                    god: GodName::Chaos,
                    message: format!("Monitoreo no encontrado para experimento: {}", experiment_id),
                });
            }
        };
        
        let analysis = self.analyze_impact(experiment_id, start_time).await?;
        
        // Guardar análisis en historial
        {
            let mut history = self.analysis_history.write().await;
            history.push(analysis.clone());
        }
        
        info!("📊 Monitoreo detenido para experimento: {} (nivel: {})", 
               experiment_id, analysis.impact_level.description());
        
        Ok(analysis)
    }
    
    /// Actualiza las métricas de un experimento en monitoreo
    pub async fn update_metrics(
        &self,
        experiment_id: &str,
        new_metrics: ImpactMetrics,
    ) -> Result<(), ActorError> {
        let mut active_impacts = self.active_impacts.write().await;
        
        if let Some(impact_state) = active_impacts.get_mut(experiment_id) {
            // Actualizar métricas
            impact_state.metrics = HashMap::new();
            impact_state.metrics.insert("latency".to_string(), new_metrics.average_latency_ms);
            impact_state.metrics.insert("error_rate".to_string(), new_metrics.error_rate);
            impact_state.metrics.insert("packet_loss".to_string(), new_metrics.packet_loss_rate);
            impact_state.metrics.insert("cpu".to_string(), new_metrics.cpu_usage);
            impact_state.metrics.insert("memory".to_string(), new_metrics.memory_usage);
            impact_state.metrics.insert("success_rate".to_string(), new_metrics.request_success_rate);
            
            // Calcular nuevo nivel de impacto
            let new_impact_level = new_metrics.get_impact_level();
            
            // Si el impacto aumentó, generar alerta
            if new_impact_level > impact_state.impact_level {
                let alert = Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: AlertType::HighImpactDetected,
                    severity: if new_impact_level >= ImpactLevel::High { AlertSeverity::Critical } else { AlertSeverity::Warning },
                    message: format!("Impacto incrementado a {} para experimento {}", 
                                   new_impact_level.description(), experiment_id),
                    created_at: Utc::now(),
                    acknowledged: false,
                    metadata: HashMap::new(),
                };
                
                impact_state.alerts.push(alert.clone());
                
                // Loguear la alerta
                warn!("📊 {}: {}", alert.severity.emoji(), alert.message);
            }
            
            impact_state.impact_level = new_impact_level;
            impact_state.last_update = Utc::now();
            
            // Verificar tiempo de recuperación
            if let Some(recovery_start) = impact_state.recovery_start {
                let elapsed = Utc::now() - recovery_start;
                let config = self.config.read().await;
                
                if elapsed.num_seconds() > config.max_recovery_time as i64 {
                    let timeout_alert = Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::RecoveryTimeout,
                        severity: AlertSeverity::Critical,
                        message: format!("Tiempo de recuperación excedido para experimento {}", experiment_id),
                        created_at: Utc::now(),
                        acknowledged: false,
                        metadata: HashMap::new(),
                    };
                    
                    impact_state.alerts.push(timeout_alert.clone());
                    error!("📊 {}: {}", timeout_alert.severity.emoji(), timeout_alert.message);
                }
            }
        } else {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Experimento no está siendo monitoreado: {}", experiment_id),
            });
        }
        
        Ok(())
    }
    
    /// Inicia el monitoreo de recuperación
    pub async fn start_recovery_monitoring(
        &self,
        experiment_id: &str,
    ) -> Result<(), ActorError> {
        let mut active_impacts = self.active_impacts.write().await;
        
        if let Some(impact_state) = active_impacts.get_mut(experiment_id) {
            impact_state.is_recovering = true;
            impact_state.recovery_start = Some(Utc::now());
            
            let recovery_alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                alert_type: AlertType::AutoRecoveryStarted,
                severity: AlertSeverity::Info,
                message: format!("Recuperación automática iniciada para experimento {}", experiment_id),
                created_at: Utc::now(),
                acknowledged: false,
                metadata: HashMap::new(),
            };
            
            impact_state.alerts.push(recovery_alert);
            
            info!("📊 Iniciando monitoreo de recuperación para: {}", experiment_id);
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Experimento no está siendo monitoreado: {}", experiment_id),
            })
        }
    }
    
    /// Analiza el impacto de un experimento
    async fn analyze_impact(
        &self,
        experiment_id: &str,
        _start_time: DateTime<Utc>,
    ) -> Result<ImpactAnalysis, ActorError> {
        let analysis_start = std::time::Instant::now();
        
        // Recolectar datos del impacto
        let (final_metrics, alerts) = {
            let active_impacts = self.active_impacts.read().await;
            if let Some(impact_state) = active_impacts.get(experiment_id) {
                // Calcular métricas finales
                let mut final_metrics = ImpactMetrics::default();
                for (key, value) in &impact_state.metrics {
                    let val = *value;
                    match key.as_str() {
                        "latency" => final_metrics.average_latency_ms = val,
                        "error_rate" => final_metrics.error_rate = val,
                        "packet_loss" => final_metrics.packet_loss_rate = val,
                        "cpu" => final_metrics.cpu_usage = val,
                        "memory" => final_metrics.memory_usage = val,
                        "success_rate" => final_metrics.request_success_rate = val,
                        _ => {}
                    }
                }
                
                final_metrics.last_measured = impact_state.last_update;
                final_metrics.uptime = (impact_state.last_update - impact_state.start_time).num_seconds() as f64;
                
                (final_metrics, impact_state.alerts.clone())
            } else {
                (ImpactMetrics::default(), Vec::new())
            }
        };
        
        // Determinar nivel de impacto
        let impact_level = final_metrics.get_impact_level();
        
        // Generar recomendaciones
        let recommendations = self.generate_recommendations(&final_metrics, &impact_level);
        
        let analysis_duration = analysis_start.elapsed().as_millis() as u64;
        
        let analysis = ImpactAnalysis {
            id: uuid::Uuid::new_v4().to_string(),
            experiment_id: experiment_id.to_string(),
            analyzed_at: Utc::now(),
            impact_level,
            metrics: final_metrics,
            alerts,
            recommendations,
            analysis_duration_seconds: analysis_duration / 1000,
        };
        
        Ok(analysis)
    }
    
    /// Genera recomendaciones basadas en las métricas
    fn generate_recommendations(
        &self,
        metrics: &ImpactMetrics,
        impact_level: &ImpactLevel,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Recomendaciones basadas en latencia
        if metrics.average_latency_ms > 500.0 {
            recommendations.push("Considerar optimizar el manejo de latencia".to_string());
        }
        
        // Recomendaciones basadas en errores
        if metrics.error_rate > 0.05 {
            recommendations.push("Investigar causas de errores e implementar retry con backoff".to_string());
        }
        
        // Recomendaciones basadas en recursos
        if metrics.cpu_usage > 85.0 {
            recommendations.push("Optimizar uso de CPU o escalar horizontalmente".to_string());
        }
        
        if metrics.memory_usage > 85.0 {
            recommendations.push("Investigar fugas de memoria o implementar pooling".to_string());
        }
        
        // Recomendaciones basadas en el nivel de impacto
        match impact_level {
            ImpactLevel::Critical | ImpactLevel::Catastrophic => {
                recommendations.push("ACTIVAR PLAN DE RECUPERACIÓN DE EMERGENCIA".to_string());
                recommendations.push("Notificar inmediatamente al equipo de respuesta a incidentes".to_string());
            },
            ImpactLevel::High => {
                recommendations.push("Escalado automático del sistema afectado".to_string());
                recommendations.push("Revisar configuración de alertas y umbrales".to_string());
            },
            ImpactLevel::Medium => {
                recommendations.push("Monitorear de cerca la evolución del impacto".to_string());
                recommendations.push("Preparar plan de mitigación si el impacto aumenta".to_string());
            },
            _ => {}
        }
        
        recommendations
    }
    
    /// Obtiene el impacto actual de un experimento
    pub async fn get_current_impact(&self, experiment_id: &str) -> Option<ImpactState> {
        let active_impacts = self.active_impacts.read().await;
        active_impacts.get(experiment_id).cloned()
    }
    
    /// Obtiene todos los impactos activos
    pub async fn get_all_active_impacts(&self) -> HashMap<String, ImpactState> {
        self.active_impacts.read().await.clone()
    }
    
    /// Obtiene el historial de análisis
    pub async fn get_analysis_history(&self) -> Vec<ImpactAnalysis> {
        self.analysis_history.read().await.clone()
    }
    
    /// Limpia impactos antiguos
    pub async fn cleanup_old_impacts(&self, older_than_hours: u64) {
        let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
        
        {
            let mut active_impacts = self.active_impacts.write().await;
            active_impacts.retain(|_, impact_state| {
                impact_state.last_update > cutoff
            });
        }
        
        {
            let mut history = self.analysis_history.write().await;
            let original_len = history.len();
            history.retain(|analysis| analysis.analyzed_at > cutoff);
            
            let removed = original_len - history.len();
            if removed > 0 {
                info!("📊 Limpieza completada: {} análisis antiguos removidos", removed);
            }
        }
    }
    
    /// Exporta datos de monitoreo
    pub async fn export_monitoring_data(&self, format: MonitoringExportFormat) -> Result<String, ActorError> {
        let active_impacts = self.active_impacts.read().await;
        let history = self.analysis_history.read().await;
        
        let data = MonitoringData {
            active_impacts: active_impacts.clone(),
            analysis_history: history.clone(),
            exported_at: Utc::now(),
        };
        
        match format {
            MonitoringExportFormat::Json => {
                serde_json::to_string_pretty(&data)
                    .map_err(|e| ActorError::Unknown {
                        god: GodName::Chaos,
                        message: format!("Error exportando JSON: {}", e),
                    })
            },
            MonitoringExportFormat::Csv => {
                self.export_to_csv(&data)
            },
        }
    }
    
    /// Exporta a formato CSV
    fn export_to_csv(&self, data: &MonitoringData) -> Result<String, ActorError> {
        let mut csv_content = "experiment_id,impact_level,start_time,last_update,is_recovering,alerts_count\n".to_string();
        
        for (exp_id, impact_state) in &data.active_impacts {
            let row = format!(
                "{},{},{},{},{},{}\n",
                exp_id,
                impact_state.impact_level.description(),
                impact_state.start_time.format("%Y-%m-%d %H:%M:%S"),
                impact_state.last_update.format("%Y-%m-%d %H:%M:%S"),
                impact_state.is_recovering,
                impact_state.alerts.len()
            );
            csv_content.push_str(&row);
        }
        
        Ok(csv_content)
    }
}

/// Formatos de exportación de monitoreo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringExportFormat {
    Json,
    Csv,
}

/// Datos completos de monitoreo para exportación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub active_impacts: HashMap<String, ImpactState>,
    pub analysis_history: Vec<ImpactAnalysis>,
    pub exported_at: DateTime<Utc>,
}

/// Wrapper principal para el sistema de monitoreo de Chaos
/// 
/// Este es el componente que el actor Chaos utiliza para interactuar
/// con todas las capacidades de monitoreo
#[derive(Debug, Clone)]
pub struct ChaosMonitor {
    /// Analizador de impacto interno
    analyzer: ImpactAnalyzer,
    /// Historial de métricas para análisis
    metrics_history: Arc<RwLock<HashMap<String, Vec<ImpactMetrics>>>>,
    /// Estadísticas agregadas
    aggregated_stats: Arc<RwLock<AggregatedStats>>,
}

/// Estadísticas agregadas del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Total de experimentos monitoreados
    pub total_experiments: u64,
    /// Experimentos exitosos
    pub successful_experiments: u64,
    /// Promedio de tiempo de recuperación (segundos)
    pub average_recovery_time: f64,
    /// Estrategias más efectivas
    pub most_effective_strategies: Vec<(String, f64)>,
    /// Resumen de impacto por nivel
    pub impact_summary: HashMap<String, u64>,
    /// Última actualización
    pub last_updated: DateTime<Utc>,
}

impl Default for AggregatedStats {
    fn default() -> Self {
        let mut impact_summary = HashMap::new();
        impact_summary.insert("None".to_string(), 0);
        impact_summary.insert("Minimal".to_string(), 0);
        impact_summary.insert("Low".to_string(), 0);
        impact_summary.insert("Medium".to_string(), 0);
        impact_summary.insert("High".to_string(), 0);
        impact_summary.insert("Critical".to_string(), 0);
        impact_summary.insert("Catastrophic".to_string(), 0);
        
        Self {
            total_experiments: 0,
            successful_experiments: 0,
            average_recovery_time: 0.0,
            most_effective_strategies: Vec::new(),
            impact_summary,
            last_updated: Utc::now(),
        }
    }
}

impl ChaosMonitor {
    /// Crea una nueva instancia del monitor de Chaos
    pub fn new() -> Self {
        Self {
            analyzer: ImpactAnalyzer::new(),
            metrics_history: Arc::new(RwLock::new(HashMap::new())),
            aggregated_stats: Arc::new(RwLock::new(AggregatedStats::default())),
        }
    }
    
    /// Inicia el monitoreo de un experimento
    pub async fn start_monitoring(
        &self,
        experiment_id: String,
        affected_systems: Vec<GodName>,
    ) -> Result<(), ActorError> {
        self.analyzer.start_monitoring(experiment_id, affected_systems).await
    }
    
    /// Detiene el monitoreo y actualiza estadísticas
    pub async fn stop_monitoring(&self, experiment_id: &str) -> Result<ImpactAnalysis, ActorError> {
        let analysis = self.analyzer.stop_monitoring(experiment_id).await?;
        
        // Actualizar estadísticas agregadas
        self.update_aggregated_stats(&analysis).await;
        
        Ok(analysis)
    }
    
    /// Actualiza métricas de un experimento activo
    pub async fn update_metrics(
        &self,
        experiment_id: &str,
        metrics: ImpactMetrics,
    ) -> Result<(), ActorError> {
        // Guardar en historial
        {
            let mut history = self.metrics_history.write().await;
            let entry = history.entry(experiment_id.to_string()).or_insert_with(Vec::new);
            entry.push(metrics.clone());
            
            // Limitar historial por experimento
            if entry.len() > 100 {
                entry.remove(0);
            }
        }
        
        self.analyzer.update_metrics(experiment_id, metrics).await
    }
    
    /// Obtiene el impacto actual de un experimento
    pub async fn get_current_impact(&self, experiment_id: &str) -> Option<ImpactMetrics> {
        if let Some(impact_state) = self.analyzer.get_current_impact(experiment_id).await {
            // Convertir ImpactState a ImpactMetrics para compatibilidad
            let mut metrics = ImpactMetrics::default();
            
            for (key, value) in &impact_state.metrics {
                let val = *value;
                match key.as_str() {
                    "latency" => metrics.average_latency_ms = val,
                    "error_rate" => metrics.error_rate = val,
                    "packet_loss" => metrics.packet_loss_rate = val,
                    "cpu" => metrics.cpu_usage = val,
                    "memory" => metrics.memory_usage = val,
                    "success_rate" => metrics.request_success_rate = val,
                    _ => {}
                }
            }
            
            metrics.impact_score = impact_state.impact_level.as_number() * 10;
            metrics.last_measured = impact_state.last_update;
            
            Some(metrics)
        } else {
            None
        }
    }
    
    /// Obtiene resumen de impacto general
    pub async fn get_impact_summary(&self) -> HashMap<String, u64> {
        let stats = self.aggregated_stats.read().await;
        stats.impact_summary.clone()
    }
    
    /// Obtiene las estrategias más efectivas
    pub async fn get_most_effective_strategies(&self) -> Vec<(String, f64)> {
        let stats = self.aggregated_stats.read().await;
        stats.most_effective_strategies.clone()
    }
    
    /// Obtiene el tiempo promedio de recuperación
    pub async fn get_average_recovery_time(&self) -> f64 {
        let stats = self.aggregated_stats.read().await;
        stats.average_recovery_time
    }
    
    /// Obtiene todas las métricas de impacto
    pub async fn get_all_impact_metrics(&self) -> HashMap<String, ImpactMetrics> {
        let mut all_metrics = HashMap::new();
        let active_impacts = self.analyzer.get_all_active_impacts().await;
        
        for (exp_id, impact_state) in active_impacts {
            let mut metrics = ImpactMetrics::default();
            
            for (key, value) in &impact_state.metrics {
                let val = *value;
                match key.as_str() {
                    "latency" => metrics.average_latency_ms = val,
                    "error_rate" => metrics.error_rate = val,
                    "packet_loss" => metrics.packet_loss_rate = val,
                    "cpu" => metrics.cpu_usage = val,
                    "memory" => metrics.memory_usage = val,
                    "success_rate" => metrics.request_success_rate = val,
                    _ => {}
                }
            }
            
            metrics.impact_score = impact_state.impact_level.as_number() * 10;
            metrics.last_measured = impact_state.last_update;
            
            all_metrics.insert(exp_id, metrics);
        }
        
        all_metrics
    }
    
    /// Actualiza las estadísticas agregadas después de un análisis
    async fn update_aggregated_stats(&self, analysis: &ImpactAnalysis) {
        let mut stats = self.aggregated_stats.write().await;
        
        stats.total_experiments += 1;
        stats.last_updated = Utc::now();
        
        // Actualizar contador de experimentos exitosos
        if analysis.impact_level <= ImpactLevel::Medium {
            stats.successful_experiments += 1;
        }
        
        // Actualizar resumen de impacto
        let level_name = analysis.impact_level.description();
        *stats.impact_summary.entry(level_name.to_string()).or_insert(0) += 1;
        
        // Calcular nuevo promedio de recuperación
        let recovery_time = self.calculate_recovery_time(analysis).await;
        if recovery_time > 0.0 {
            let new_total = stats.total_experiments;
            let current_sum = stats.average_recovery_time * (new_total - 1) as f64;
            stats.average_recovery_time = (current_sum + recovery_time) / new_total as f64;
        }
        
        // Actualizar estrategias efectivas (placeholder - requeriría más contexto)
        if stats.most_effective_strategies.is_empty() {
            stats.most_effective_strategies.push(("RandomFailure".to_string(), 0.8));
            stats.most_effective_strategies.push(("LatencyInjection".to_string(), 0.7));
        }
    }
    
    /// Calcula el tiempo de recuperación para un análisis
    async fn calculate_recovery_time(&self, analysis: &ImpactAnalysis) -> f64 {
        // Usar la duración del análisis como proxy para tiempo de recuperación
        // En una implementación real, esto sería más sofisticado
        analysis.analysis_duration_seconds as f64
    }
    
    /// Inicia monitoreo de recuperación
    pub async fn start_recovery_monitoring(&self, experiment_id: &str) -> Result<(), ActorError> {
        self.analyzer.start_recovery_monitoring(experiment_id).await
    }
    
    /// Limpia datos antiguos
    pub async fn cleanup_old_data(&self, older_than_hours: u64) {
        self.analyzer.cleanup_old_impacts(older_than_hours).await;
        
        // Limpiar historial de métricas
        let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
        let mut history = self.metrics_history.write().await;
        history.retain(|_, metrics| {
            metrics.last().map_or(true, |m| m.last_measured > cutoff)
        });
    }
    
    /// Exporta todos los datos de monitoreo
    pub async fn export_all_data(&self, format: MonitoringExportFormat) -> Result<String, ActorError> {
        self.analyzer.export_monitoring_data(format).await
    }
    
    /// Obtiene estadísticas generales del monitor
    pub async fn get_monitoring_stats(&self) -> AggregatedStats {
        self.aggregated_stats.read().await.clone()
    }
    
    /// Obtiene el historial de análisis completo
    pub async fn get_analysis_history(&self) -> Vec<ImpactAnalysis> {
        self.analyzer.get_analysis_history().await
    }
    
    /// Obtiene todos los impactos activos
    pub async fn get_active_impacts(&self) -> HashMap<String, ImpactState> {
        self.analyzer.get_all_active_impacts().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::GodName;

    #[test]
    fn test_impact_level_ordering() {
        assert!(ImpactLevel::Low < ImpactLevel::Medium);
        assert!(ImpactLevel::Medium < ImpactLevel::High);
        assert!(ImpactLevel::Critical < ImpactLevel::Catastrophic);
        
        assert_eq!(ImpactLevel::High.as_number(), 4);
        assert_eq!(ImpactLevel::Critical.as_number(), 5);
    }

    #[test]
    fn test_impact_metrics_calculation() {
        let metrics = ImpactMetrics {
            average_latency_ms: 1500.0, // Alto
            error_rate: 0.05,          // 5%
            packet_loss_rate: 0.02,      // 2%
            cpu_usage: 80.0,            // 80%
            memory_usage: 90.0,          // 90%
            request_success_rate: 95.0,
            uptime: 300.0,
            impact_score: 0,
            last_measured: Utc::now(),
        };
        
        let score = metrics.calculate_impact_score();
        assert!(score > 60); // Debería ser Alto
        
        let level = metrics.get_impact_level();
        assert_eq!(level, ImpactLevel::High);
    }

    #[test]
    fn test_alert_severity() {
        assert_eq!(AlertSeverity::Info.as_number(), 1);
        assert_eq!(AlertSeverity::Critical.as_number(), 4);
        
        assert_eq!(AlertSeverity::Warning.emoji(), "⚠️");
        assert_eq!(AlertSeverity::Error.emoji(), "❌");
    }

    #[tokio::test]
    async fn test_impact_analyzer() {
        let analyzer = ImpactAnalyzer::new();
        
        let experiment_id = "test_exp_001";
        let affected_systems = vec![GodName::Hades, GodName::Poseidon];
        
        // Iniciar monitoreo
        let result = analyzer.start_monitoring(experiment_id.to_string(), affected_systems).await;
        assert!(result.is_ok());
        
        // Verificar que está en monitoreo
        let current_impact = analyzer.get_current_impact(experiment_id).await;
        assert!(current_impact.is_some());
        
        let impact_state = current_impact.unwrap();
        assert_eq!(impact_state.experiment_id, experiment_id);
        assert!(!impact_state.is_recovering);
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let analyzer = ImpactAnalyzer::new();
        let experiment_id = "test_exp_002";
        
        // Iniciar monitoreo primero
        analyzer.start_monitoring(experiment_id.to_string(), vec![GodName::Athena]).await.unwrap();
        
        // Actualizar métricas
        let new_metrics = ImpactMetrics {
            average_latency_ms: 800.0,
            error_rate: 0.02,
            packet_loss_rate: 0.01,
            cpu_usage: 70.0,
            memory_usage: 60.0,
            request_success_rate: 98.0,
            uptime: 180.0,
            impact_score: 0,
            last_measured: Utc::now(),
        };
        
        let result = analyzer.update_metrics(experiment_id, &new_metrics).await;
        assert!(result.is_ok());
        
        // Verificar que las métricas se actualizaron
        let current_impact = analyzer.get_current_impact(experiment_id).await;
        assert!(current_impact.is_some());
    }
}