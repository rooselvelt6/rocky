// src/actors/chaos/impact.rs
// OLYMPUS v15 - Sistema Avanzado de Análisis de Impacto para Chaos

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use tracing::info;

use crate::actors::{GodName};
use crate::errors::ActorError;

/// Sistema avanzado de análisis de impacto
/// 
/// Proporciona análisis detallado del impacto de los experimentos chaos
/// incluyendo correlación, causalidad y predicción de efectos
#[derive(Debug, Clone)]
pub struct ImpactAnalysisSystem {
    /// Configuración del análisis
    config: Arc<RwLock<ImpactAnalysisConfig>>,
    /// Análisis activos
    active_analyses: Arc<RwLock<HashMap<String, ActiveImpactAnalysis>>>,
    /// Historial de impactos
    impact_history: Arc<RwLock<Vec<ImpactRecord>>>,
    /// Base de conocimiento de patrones de impacto
    impact_patterns: Arc<RwLock<HashMap<String, ImpactPattern>>>,
    /// Correlaciones detectadas
    correlations: Arc<RwLock<HashMap<String, CorrelationData>>>,
}

/// Configuración para análisis de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysisConfig {
    /// Niveles de impacto configurables
    pub impact_thresholds: ImpactThresholds,
    /// Tiempo de ventana para análisis (minutos)
    pub analysis_window_minutes: u64,
    /// Mínimo de muestras para correlación
    pub min_samples_for_correlation: usize,
    /// Umbral de significancia estadística
    pub significance_threshold: f64,
    /// Habilitar predicción de impacto
    pub enable_impact_prediction: bool,
    /// Horizonte de predicción (minutos)
    pub prediction_horizon_minutes: u64,
}

/// Umbrales de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactThresholds {
    /// Umbral para impacto bajo (%)
    pub low_threshold: f64,
    /// Umbral para impacto medio (%)
    pub medium_threshold: f64,
    /// Umbral para impacto alto (%)
    pub high_threshold: f64,
    /// Umbral para impacto crítico (%)
    pub critical_threshold: f64,
}

impl Default for ImpactAnalysisConfig {
    fn default() -> Self {
        Self {
            impact_thresholds: ImpactThresholds {
                low_threshold: 10.0,
                medium_threshold: 25.0,
                high_threshold: 50.0,
                critical_threshold: 75.0,
            },
            analysis_window_minutes: 30,
            min_samples_for_correlation: 5,
            significance_threshold: 0.8,
            enable_impact_prediction: true,
            prediction_horizon_minutes: 15,
        }
    }
}

/// Análisis de impacto activo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveImpactAnalysis {
    /// ID del análisis
    pub analysis_id: String,
    /// ID del experimento analizado
    pub experiment_id: String,
    /// Sistemas afectados primarios
    pub primary_affected: Vec<GodName>,
    /// Sistemas afectados secundarios (cascada)
    pub secondary_affected: Vec<GodName>,
    /// Métricas base (antes del experimento)
    pub baseline_metrics: HashMap<GodName, BaselineMetrics>,
    /// Métricas actuales
    pub current_metrics: HashMap<GodName, CurrentMetrics>,
    /// Impacto calculado por sistema
    pub system_impacts: HashMap<GodName, DetailedImpact>,
    /// Timeline de eventos
    pub event_timeline: Vec<ImpactEvent>,
    /// Estado del análisis
    pub status: AnalysisStatus,
    /// Momento de inicio
    pub start_time: DateTime<Utc>,
    /// Última actualización
    pub last_update: DateTime<Utc>,
}

/// Métricas base de un sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Latencia promedio (ms)
    pub avg_latency_ms: f64,
    /// Tasa de error (%)
    pub error_rate: f64,
    /// Throughput (req/s)
    pub throughput: f64,
    /// Uso de CPU (%)
    pub cpu_usage: f64,
    /// Uso de memoria (%)
    pub memory_usage: f64,
    /// Tiempo de actividad (%)
    pub uptime_percentage: f64,
    /// Timestamp de la medición
    pub measured_at: DateTime<Utc>,
}

/// Métricas actuales de un sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMetrics {
    /// Latencia promedio (ms)
    pub avg_latency_ms: f64,
    /// Tasa de error (%)
    pub error_rate: f64,
    /// Throughput (req/s)
    pub throughput: f64,
    /// Uso de CPU (%)
    pub cpu_usage: f64,
    /// Uso de memoria (%)
    pub memory_usage: f64,
    /// Tiempo de actividad (%)
    pub uptime_percentage: f64,
    /// Timestamp de la medición
    pub measured_at: DateTime<Utc>,
    /// Desviación respecto a baseline
    pub deviation_from_baseline: MetricDeviations,
}

/// Desviaciones de métricas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDeviations {
    /// Desviación de latencia (%)
    pub latency_deviation: f64,
    /// Desviación de error (%)
    pub error_deviation: f64,
    /// Desviación de throughput (%)
    pub throughput_deviation: f64,
    /// Desviación de CPU (%)
    pub cpu_deviation: f64,
    /// Desviación de memoria (%)
    pub memory_deviation: f64,
    /// Desviación de uptime (%)
    pub uptime_deviation: f64,
}

/// Impacto detallado en un sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedImpact {
    /// ID del sistema afectado
    pub system_id: GodName,
    /// Nivel de impacto general
    pub impact_level: ImpactLevel,
    /// Score de impacto (0-100)
    pub impact_score: u8,
    /// Tiempo de respuesta vs baseline
    pub response_time_factor: f64,
    /// Disponibilidad vs baseline
    pub availability_factor: f64,
    /// Rendimiento vs baseline
    pub performance_factor: f64,
    /// Categorías de impacto detectadas
    pub impact_categories: Vec<ImpactCategory>,
    /// Efectos secundarios observados
    pub side_effects: Vec<String>,
    /// Tiempo estimado de recuperación (segundos)
    pub estimated_recovery_time: u64,
    /// Confianza del análisis (0.0-1.0)
    pub confidence: f64,
}

/// Niveles de impacto
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpactLevel {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Critical,
    Catastrophic,
}

impl ImpactLevel {
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
    
    pub fn color(&self) -> &'static str {
        match self {
            ImpactLevel::None => "🟢",
            ImpactLevel::Minimal => "🟡",
            ImpactLevel::Low => "🟠",
            ImpactLevel::Medium => "🟠",
            ImpactLevel::High => "🔴",
            ImpactLevel::Critical => "🔴",
            ImpactLevel::Catastrophic => "🔴",
        }
    }
}

/// Categorías de impacto
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImpactCategory {
    Performance,
    Availability,
    Reliability,
    Scalability,
    DataIntegrity,
    UserExperience,
    Security,
    Cost,
    Compliance,
}

/// Evento en la timeline de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEvent {
    /// ID del evento
    pub event_id: String,
    /// Timestamp del evento
    pub timestamp: DateTime<Utc>,
    /// Tipo de evento
    pub event_type: ImpactEventType,
    /// Sistema afectado
    pub affected_system: Option<GodName>,
    /// Descripción del evento
    pub description: String,
    /// Severidad del evento
    pub severity: EventSeverity,
    /// Métricas asociadas
    pub metrics: HashMap<String, f64>,
    /// Contexto adicional
    pub context: HashMap<String, String>,
}

/// Tipos de eventos de impacto
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactEventType {
    /// Inicio de impacto
    ImpactStarted,
    /// Cambio en nivel de impacto
    ImpactLevelChanged,
    /// Nuevo sistema afectado
    SystemAffected,
    /// Recuperación parcial
    PartialRecovery,
    /// Recuperación completa
    FullRecovery,
    /// Degradación adicional
    FurtherDegradation,
    /// Correlación detectada
    CorrelationDetected,
    /// Anomalía detectada
    AnomalyDetected,
}

/// Severidad de eventos
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Estado del análisis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Initializing,
    CollectingBaselines,
    Analyzing,
    DetectingPatterns,
    GeneratingInsights,
    Completed,
    Failed,
}

/// Patrón de impacto detectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPattern {
    /// ID del patrón
    pub pattern_id: String,
    /// Nombre del patrón
    pub name: String,
    /// Descripción
    pub description: String,
    /// Secuencia típica de impactos
    pub impact_sequence: Vec<ImpactStep>,
    /// Frecuencia de ocurrencia
    pub frequency: f64,
    /// Confianza del patrón
    pub confidence: f64,
    /// Estrategias de mitigación recomendadas
    pub mitigation_strategies: Vec<String>,
    /// Timestamp de detección
    pub detected_at: DateTime<Utc>,
}

/// Paso en un patrón de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactStep {
    /// Sistema afectado
    pub system: GodName,
    /// Tipo de impacto
    pub impact_type: ImpactCategory,
    /// Nivel de impacto esperado
    pub expected_level: ImpactLevel,
    /// Tiempo relativo (segundos desde inicio)
    pub relative_time: u64,
    /// Duración esperada (segundos)
    pub expected_duration: u64,
}

/// Datos de correlación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationData {
    /// ID de la correlación
    pub correlation_id: String,
    /// Sistemas correlacionados
    pub correlated_systems: (GodName, GodName),
    /// Coeficiente de correlación
    pub correlation_coefficient: f64,
    /// Significancia estadística
    pub significance: f64,
    /// Tipo de correlación
    pub correlation_type: CorrelationType,
    /// Tiempo de desfasaje (segundos)
    pub lag_time: u64,
    /// Timestamp de detección
    pub detected_at: DateTime<Utc>,
}

/// Tipos de correlación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationType {
    Direct,
    Inverse,
    Lagged,
    Seasonal,
    Spurious,
}

/// Registro histórico de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactRecord {
    /// ID del registro
    pub record_id: String,
    /// ID del análisis
    pub analysis_id: String,
    /// ID del experimento
    pub experiment_id: String,
    /// Resumen de impacto
    pub impact_summary: ImpactSummary,
    /// Análisis detallado
    pub detailed_analysis: serde_json::Value,
    /// Lecciones aprendidas
    pub lessons_learned: Vec<String>,
    /// Recomendaciones
    pub recommendations: Vec<String>,
    /// Timestamp del registro
    pub recorded_at: DateTime<Utc>,
}

/// Resumen de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    /// Impacto máximo detectado
    pub max_impact_level: ImpactLevel,
    /// Sistemas afectados
    pub affected_systems_count: u32,
    /// Duración total del impacto (segundos)
    pub total_impact_duration: u64,
    /// Score de impacto general (0-100)
    pub overall_impact_score: u8,
    /// Tiempo de recuperación (segundos)
    pub recovery_time: u64,
    /// Categorías de impacto principales
    pub primary_impact_categories: Vec<ImpactCategory>,
}

impl ImpactAnalysisSystem {
    /// Crea una nueva instancia del sistema de análisis
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ImpactAnalysisConfig::default())),
            active_analyses: Arc::new(RwLock::new(HashMap::new())),
            impact_history: Arc::new(RwLock::new(Vec::new())),
            impact_patterns: Arc::new(RwLock::new(HashMap::new())),
            correlations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Inicia análisis de impacto para un experimento
    pub async fn start_impact_analysis(
        &self,
        experiment_id: String,
        target_systems: Vec<GodName>,
    ) -> Result<String, ActorError> {
        let analysis_id = Uuid::new_v4().to_string();
        
        // Recolectar métricas base
        let baseline_metrics = self.collect_baseline_metrics(&target_systems).await?;
        
        let analysis = ActiveImpactAnalysis {
            analysis_id: analysis_id.clone(),
            experiment_id: experiment_id.clone(),
            primary_affected: target_systems.clone(),
            secondary_affected: Vec::new(),
            baseline_metrics,
            current_metrics: HashMap::new(),
            system_impacts: HashMap::new(),
            event_timeline: vec![
                ImpactEvent {
                    event_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    event_type: ImpactEventType::ImpactStarted,
                    affected_system: None,
                    description: format!("Análisis iniciado para experimento {}", experiment_id),
                    severity: EventSeverity::Info,
                    metrics: HashMap::new(),
                    context: HashMap::new(),
                }
            ],
            status: AnalysisStatus::CollectingBaselines,
            start_time: Utc::now(),
            last_update: Utc::now(),
        };
        
        // Agregar a análisis activos
        {
            let mut active = self.active_analyses.write().await;
            active.insert(analysis_id.clone(), analysis);
        }
        
        // Iniciar análisis asíncrono
        self.execute_impact_analysis(analysis_id.clone()).await?;
        
        info!("📊 Análisis de impacto iniciado: {}", analysis_id);
        Ok(analysis_id)
    }
    
    /// Actualiza métricas durante un análisis activo
    pub async fn update_metrics(
        &self,
        analysis_id: &str,
        current_metrics: HashMap<GodName, CurrentMetrics>,
    ) -> Result<(), ActorError> {
        let mut active = self.active_analyses.write().await;
        
        if let Some(analysis) = active.get_mut(analysis_id) {
            analysis.current_metrics = current_metrics.clone();
            analysis.last_update = Utc::now();
            
            // Calcular impactos basados en nuevas métricas
            for (system, metrics) in current_metrics {
                if let Some(baseline) = analysis.baseline_metrics.get(&system) {
                    let impact = self.calculate_detailed_impact(baseline, &metrics).await;
                    analysis.system_impacts.insert(system, impact);
                }
            }
            
            // Agregar evento de actualización
            analysis.event_timeline.push(ImpactEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: ImpactEventType::ImpactLevelChanged,
                affected_system: None,
                description: "Métricas actualizadas".to_string(),
                severity: EventSeverity::Info,
                metrics: HashMap::new(),
                context: HashMap::new(),
            });
            
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Análisis no encontrado: {}", analysis_id),
            })
        }
    }
    
    /// Recolecta métricas base de los sistemas
    async fn collect_baseline_metrics(
        &self,
        systems: &[GodName],
    ) -> Result<HashMap<GodName, BaselineMetrics>, ActorError> {
        let mut baseline = HashMap::new();
        
        for system in systems {
            // Simular recolección de métricas base
            baseline.insert(system.clone(), BaselineMetrics {
                avg_latency_ms: 100.0 + (rand::random::<f64>() * 50.0),
                error_rate: rand::random::<f64>() * 0.01, // 0-1%
                throughput: 1000.0 + (rand::random::<f64>() * 500.0),
                cpu_usage: 30.0 + (rand::random::<f64>() * 40.0), // 30-70%
                memory_usage: 40.0 + (rand::random::<f64>() * 30.0), // 40-70%
                uptime_percentage: 99.5 + (rand::random::<f64>() * 0.5),
                measured_at: Utc::now(),
            });
        }
        
        Ok(baseline)
    }
    
    /// Calcula impacto detallado comparando baseline con métricas actuales
    async fn calculate_detailed_impact(
        &self,
        baseline: &BaselineMetrics,
        current: &CurrentMetrics,
    ) -> DetailedImpact {
        // Calcular desviaciones
        let latency_dev = ((current.avg_latency_ms - baseline.avg_latency_ms) / baseline.avg_latency_ms) * 100.0;
        let error_dev = ((current.error_rate - baseline.error_rate) / baseline.error_rate.max(0.001)) * 100.0;
        let throughput_dev = ((current.throughput - baseline.throughput) / baseline.throughput) * 100.0;
        let cpu_dev = current.cpu_usage - baseline.cpu_usage;
        let memory_dev = current.memory_usage - baseline.memory_usage;
        let uptime_dev = baseline.uptime_percentage - current.uptime_percentage;
        
        // Calcular score de impacto (0-100)
        let latency_score = if latency_dev > 100.0 { 25.0 } else if latency_dev > 50.0 { 15.0 } else if latency_dev > 20.0 { 10.0 } else { 0.0 };
        let error_score = if error_dev > 500.0 { 30.0 } else if error_dev > 100.0 { 20.0 } else if error_dev > 50.0 { 10.0 } else { 0.0 };
        let throughput_score = if throughput_dev < -50.0 { 20.0 } else if throughput_dev < -25.0 { 10.0 } else if throughput_dev < -10.0 { 5.0 } else { 0.0 };
        let resource_score = (cpu_dev.max(memory_dev) / 100.0 * 25.0).min(25.0);
        
        let impact_score = (latency_score + error_score + throughput_score + resource_score) as u8;
        
        // Determinar nivel de impacto
        let impact_level = if impact_score >= 80 { ImpactLevel::Catastrophic }
                          else if impact_score >= 60 { ImpactLevel::Critical }
                          else if impact_score >= 40 { ImpactLevel::High }
                          else if impact_score >= 25 { ImpactLevel::Medium }
                          else if impact_score >= 10 { ImpactLevel::Low }
                          else if impact_score > 0 { ImpactLevel::Minimal }
                          else { ImpactLevel::None };
        
        // Identificar categorías de impacto
        let mut categories = Vec::new();
        if latency_dev > 20.0 { categories.push(ImpactCategory::Performance); }
        if uptime_dev > 5.0 { categories.push(ImpactCategory::Availability); }
        if error_dev > 50.0 { categories.push(ImpactCategory::Reliability); }
        if cpu_dev > 30.0 || memory_dev > 30.0 { categories.push(ImpactCategory::Scalability); }
        
        // Calcular factores
        let response_time_factor = current.avg_latency_ms / baseline.avg_latency_ms;
        let availability_factor = current.uptime_percentage / baseline.uptime_percentage;
        let performance_factor = current.throughput / baseline.throughput;
        
        // Estimar tiempo de recuperación (segundos)
        let estimated_recovery = match impact_level {
            ImpactLevel::None => 0,
            ImpactLevel::Minimal => 30,
            ImpactLevel::Low => 120,
            ImpactLevel::Medium => 300,
            ImpactLevel::High => 600,
            ImpactLevel::Critical => 1800,
            ImpactLevel::Catastrophic => 3600,
        };
        
        DetailedImpact {
            system_id: GodName::Chaos, // Placeholder - should be passed as parameter
            impact_level,
            impact_score,
            response_time_factor,
            availability_factor,
            performance_factor,
            impact_categories: categories,
            side_effects: Vec::new(),
            estimated_recovery_time: estimated_recovery,
            confidence: 0.85, // Default confidence
        }
    }
    
    /// Ejecuta análisis de impacto de forma asíncrona
    async fn execute_impact_analysis(&self, analysis_id: String) -> Result<(), ActorError> {
        let active_analyses = self.active_analyses.clone();
        let correlations = self.correlations.clone();
        let impact_patterns = self.impact_patterns.clone();
        
        tokio::spawn(async move {
            // Fase 1: Recolección de datos
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            // Actualizar estado
            {
                let mut active = active_analyses.write().await;
                if let Some(analysis) = active.get_mut(&analysis_id) {
                    analysis.status = AnalysisStatus::Analyzing;
                }
            }
            
            // Fase 2: Detección de patrones
            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
            
            {
                let mut active = active_analyses.write().await;
                if let Some(analysis) = active.get_mut(&analysis_id) {
                    analysis.status = AnalysisStatus::DetectingPatterns;
                }
            }
            
            // Fase 3: Generación de insights
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            
            // Completar análisis
            {
                let mut active = active_analyses.write().await;
                if let Some(analysis) = active.get_mut(&analysis_id) {
                    analysis.status = AnalysisStatus::Completed;
                    
                    // Agregar evento de completado
                    analysis.event_timeline.push(ImpactEvent {
                        event_id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        event_type: ImpactEventType::FullRecovery,
                        affected_system: None,
                        description: "Análisis de impacto completado".to_string(),
                        severity: EventSeverity::Info,
                        metrics: HashMap::new(),
                        context: HashMap::new(),
                    });
                }
            }
        });
        
        Ok(())
    }
    
    /// Detecta correlaciones entre sistemas afectados
    pub async fn detect_correlations(&self, analysis_id: &str) -> Result<Vec<CorrelationData>, ActorError> {
        let active = self.active_analyses.read().await;
        
        if let Some(analysis) = active.get(analysis_id) {
            let mut correlations = Vec::new();
            
            // Simular detección de correlaciones
            let systems: Vec<_> = analysis.system_impacts.keys().cloned().collect();
            
            for i in 0..systems.len() {
                for j in (i + 1)..systems.len() {
                    // Simular correlación
                    let correlation_coeff = rand::random::<f64>() * 0.8 + 0.2; // 0.2-1.0
                    
                    if correlation_coeff > 0.7 {
                        let correlation = CorrelationData {
                            correlation_id: Uuid::new_v4().to_string(),
                            correlated_systems: (systems[i].clone(), systems[j].clone()),
                            correlation_coefficient: correlation_coeff,
                            significance: correlation_coeff * 0.9,
                            correlation_type: if correlation_coeff > 0.8 { CorrelationType::Direct } else { CorrelationType::Lagged },
                            lag_time: (rand::random::<f64>() * 30.0) as u64,
                            detected_at: Utc::now(),
                        };
                        correlations.push(correlation);
                    }
                }
            }
            
            // Guardar correlaciones detectadas
            {
                let mut stored_correlations = self.correlations.write().await;
                for correlation in &correlations {
                    stored_correlations.insert(correlation.correlation_id.clone(), correlation.clone());
                }
            }
            
            Ok(correlations)
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Análisis no encontrado: {}", analysis_id),
            })
        }
    }
    
    /// Predice impacto futuro basado en tendencias actuales
    pub async fn predict_impact(
        &self,
        analysis_id: &str,
        prediction_minutes: u64,
    ) -> Result<ImpactPrediction, ActorError> {
        let active = self.active_analyses.read().await;
        
        if let Some(analysis) = active.get(analysis_id) {
            // Simular predicción basada en tendencias actuales
            let max_current_impact = analysis.system_impacts.values()
                .map(|impact| impact.impact_score)
                .max()
                .unwrap_or(0);
            
            let predicted_impact = if max_current_impact > 60 {
                (max_current_impact + 10).min(100)
            } else if max_current_impact > 30 {
                max_current_impact + 5
            } else {
                max_current_impact + 2
            };
            
            let predicted_level = if predicted_impact >= 80 { ImpactLevel::Catastrophic }
                              else if predicted_impact >= 60 { ImpactLevel::Critical }
                              else if predicted_impact >= 40 { ImpactLevel::High }
                              else if predicted_impact >= 25 { ImpactLevel::Medium }
                              else if predicted_impact >= 10 { ImpactLevel::Low }
                              else { ImpactLevel::Minimal };
            
            Ok(ImpactPrediction {
                prediction_id: Uuid::new_v4().to_string(),
                analysis_id: analysis_id.to_string(),
                prediction_horizon_minutes: prediction_minutes,
                predicted_impact_level: predicted_level,
                predicted_impact_score: predicted_impact as u8,
                confidence: 0.75,
                predicted_affected_systems: analysis.primary_affected.clone(),
                risk_factors: vec![
                    "Degradación continuada detectada".to_string(),
                    "Recursos del sistema bajo presión".to_string(),
                ],
                recommended_actions: vec![
                    "Monitorear de cerca los sistemas afectados".to_string(),
                    "Preparar plan de recuperación".to_string(),
                ],
                created_at: Utc::now(),
            })
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Análisis no encontrado: {}", analysis_id),
            })
        }
    }
    
    /// Finaliza un análisis y guarda en historial
    pub async fn complete_analysis(&self, analysis_id: &str) -> Result<ImpactRecord, ActorError> {
        let analysis = {
            let mut active = self.active_analyses.write().await;
            active.remove(analysis_id).ok_or_else(|| ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Análisis no encontrado: {}", analysis_id),
            })?
        };
        
        // Calcular resumen de impacto
        let max_impact = analysis.system_impacts.values()
            .max_by_key(|impact| impact.impact_level.as_number())
            .cloned()
            .unwrap_or_else(|| DetailedImpact {
                system_id: GodName::Zeus,
                impact_level: ImpactLevel::None,
                impact_score: 0,
                response_time_factor: 1.0,
                availability_factor: 1.0,
                performance_factor: 1.0,
                impact_categories: Vec::new(),
                side_effects: Vec::new(),
                estimated_recovery_time: 0,
                confidence: 0.0,
            });
        
        let impact_summary = ImpactSummary {
            max_impact_level: max_impact.impact_level.clone(),
            affected_systems_count: analysis.system_impacts.len() as u32,
            total_impact_duration: (analysis.last_update - analysis.start_time).num_seconds() as u64,
            overall_impact_score: max_impact.impact_score,
            recovery_time: max_impact.estimated_recovery_time,
            primary_impact_categories: max_impact.impact_categories,
        };
        
        // Generar lecciones aprendidas
        let mut lessons = Vec::new();
        if max_impact.impact_level >= ImpactLevel::High {
            lessons.push("Se detectó impacto significativo en múltiples sistemas".to_string());
        }
        if analysis.secondary_affected.len() > 0 {
            lessons.push("Se observó efecto cascada en sistemas secundarios".to_string());
        }
        
        // Generar recomendaciones
        let mut recommendations = Vec::new();
        match max_impact.impact_level {
            ImpactLevel::Critical | ImpactLevel::Catastrophic => {
                recommendations.push("Implementar circuit breakers más agresivos".to_string());
                recommendations.push("Revisar configuración de límites de recursos".to_string());
            },
            ImpactLevel::High => {
                recommendations.push("Considerar auto-escalado basado en métricas".to_string());
                recommendations.push("Implementar monitoreo proactivo".to_string());
            },
            _ => {
                recommendations.push("Continuar monitoreo normal".to_string());
            }
        }
        
        let record = ImpactRecord {
            record_id: Uuid::new_v4().to_string(),
            analysis_id: analysis_id.to_string(),
            experiment_id: analysis.experiment_id,
            impact_summary,
            detailed_analysis: serde_json::to_value(&analysis.system_impacts).unwrap_or_default(),
            lessons_learned: lessons,
            recommendations,
            recorded_at: Utc::now(),
        };
        
        // Guardar en historial
        {
            let mut history = self.impact_history.write().await;
            history.push(record.clone());
        }
        
        info!("📊 Análisis completado y guardado: {}", analysis_id);
        Ok(record)
    }
    
    /// Obtiene el estado de un análisis activo
    pub async fn get_analysis_status(&self, analysis_id: &str) -> Option<ActiveImpactAnalysis> {
        let active = self.active_analyses.read().await;
        active.get(analysis_id).cloned()
    }
    
    /// Obtiene todos los análisis activos
    pub async fn get_active_analyses(&self) -> HashMap<String, ActiveImpactAnalysis> {
        self.active_analyses.read().await.clone()
    }
    
    /// Obtiene el historial de impactos
    pub async fn get_impact_history(&self) -> Vec<ImpactRecord> {
        self.impact_history.read().await.clone()
    }
    
    /// Limpia análisis antiguos
    pub async fn cleanup_old_analyses(&self, older_than_hours: u64) {
        let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
        
        {
            let mut active = self.active_analyses.write().await;
            let original_len = active.len();
            active.retain(|_, analysis| analysis.start_time > cutoff);
            
            let removed = original_len - active.len();
            if removed > 0 {
                info!("📊 {} análisis antiguos removidos", removed);
            }
        }
        
        {
            let mut history = self.impact_history.write().await;
            let original_len = history.len();
            history.retain(|record| record.recorded_at > cutoff);
            
            let removed = original_len - history.len();
            if removed > 0 {
                info!("📊 {} registros antiguos removidos", removed);
            }
        }
    }
    
    /// Obtiene estadísticas generales del sistema
    pub async fn get_system_stats(&self) -> serde_json::Value {
        let active = self.active_analyses.read().await;
        let history = self.impact_history.read().await;
        let patterns = self.impact_patterns.read().await;
        let correlations = self.correlations.read().await;
        
        serde_json::json!({
            "active_analyses": active.len(),
            "total_impact_records": history.len(),
            "detected_patterns": patterns.len(),
            "correlations_found": correlations.len(),
            "analysis_status_distribution:": {
                "initializing": active.values().filter(|a| matches!(a.status, AnalysisStatus::Initializing)).count(),
                "collecting": active.values().filter(|a| matches!(a.status, AnalysisStatus::CollectingBaselines)).count(),
                "analyzing": active.values().filter(|a| matches!(a.status, AnalysisStatus::Analyzing)).count(),
                "detecting": active.values().filter(|a| matches!(a.status, AnalysisStatus::DetectingPatterns)).count(),
                "generating": active.values().filter(|a| matches!(a.status, AnalysisStatus::GeneratingInsights)).count(),
                "completed": active.values().filter(|a| matches!(a.status, AnalysisStatus::Completed)).count(),
            }
        })
    }
}

/// Predicción de impacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPrediction {
    /// ID de la predicción
    pub prediction_id: String,
    /// ID del análisis base
    pub analysis_id: String,
    /// Horizonte de predicción (minutos)
    pub prediction_horizon_minutes: u64,
    /// Nivel de impacto predicho
    pub predicted_impact_level: ImpactLevel,
    /// Score de impacto predicho
    pub predicted_impact_score: u8,
    /// Confianza de la predicción (0.0-1.0)
    pub confidence: f64,
    /// Sistemas que se predicen afectados
    pub predicted_affected_systems: Vec<GodName>,
    /// Factores de riesgo identificados
    pub risk_factors: Vec<String>,
    /// Acciones recomendadas
    pub recommended_actions: Vec<String>,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
}