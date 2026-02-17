// src/actors/aurora/opportunities.rs
// OLYMPUS v15 - Aurora: Detección y Gestión de Oportunidades

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::actors::GodName;
use crate::errors::ActorError;
use tracing::info;

/// Tipos de oportunidades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityType {
    /// Oportunidad técnica
    Technical,
    /// Oportunidad de negocio
    Business,
    /// Oportunidad personal
    Personal,
    /// Oportunidad de aprendizaje
    Learning,
    /// Oportunidad de colaboración
    Collaboration,
    /// Oportunidad de innovación
    Innovation,
    /// Oportunidad de optimización
    Optimization,
    /// Oportunidad de crecimiento
    Growth,
    /// Oportunidad de servicio
    Service,
}

/// Prioridades de oportunidades
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum OpportunityPriority {
    /// Crítica - actuar inmediatamente
    Critical { urgency_score: u8 },
    /// Alta - actuar pronto
    High { deadline_hours: u32 },
    /// Media - planificar
    Medium { effort_estimate: u8 },
    /// Baja - considerar
    Low { benefit_ratio: f64 },
    /// Informativa - solo conocimiento
    Informational,
}

impl OpportunityPriority {
    pub fn to_numeric(&self) -> u8 {
        match self {
            OpportunityPriority::Critical { urgency_score } => 80 + urgency_score / 4,
            OpportunityPriority::High { deadline_hours } => 60 + (deadline_hours / 8).min(15) as u8,
            OpportunityPriority::Medium { effort_estimate } => 40 + effort_estimate / 6,
            OpportunityPriority::Low { benefit_ratio } => 20 + (*benefit_ratio * 10.0).min(19.0) as u8,
            OpportunityPriority::Informational => 10,
        }
    }
    
    pub fn get_deadline_hours(&self) -> Option<u32> {
        match self {
            OpportunityPriority::High { deadline_hours } => Some(*deadline_hours),
            OpportunityPriority::Critical { urgency_score } => Some(1 + (255 - *urgency_score) as u32 / 50),
            _ => None,
        }
    }
}

/// Estados de oportunidades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityStatus {
    /// Detectada recientemente
    Detected,
    /// En evaluación
    Evaluating,
    /// Aprobada para ejecución
    Approved,
    /// En progreso
    InProgress,
    /// Completada exitosamente
    Completed,
    /// Fallida
    Failed,
    /// Cancelada
    Cancelled,
    /// Pospuesta
    Postponed,
    /// Expirada
    Expired,
}

/// Oportunidad detectada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    /// ID único de la oportunidad
    pub opportunity_id: String,
    /// Timestamp de detección
    pub detected_at: DateTime<Utc>,
    /// Tipo de oportunidad
    pub opportunity_type: OpportunityType,
    /// Prioridad
    pub priority: OpportunityPriority,
    /// Estado actual
    pub status: OpportunityStatus,
    /// Título descriptivo
    pub title: String,
    /// Descripción detallada
    pub description: String,
    /// Potencial impacto (0-100)
    pub potential_impact: f64,
    /// Esfuerzo estimado (horas)
    pub estimated_effort_hours: f64,
    /// Recursos necesarios
    pub required_resources: Vec<String>,
    /// Condiciones previas
    pub prerequisites: Vec<String>,
    /// Potencial retorno
    pub expected_return: Option<f64>,
    /// Ventana de tiempo (opcional)
    pub time_window_hours: Option<u32>,
    /// Componente o sistema afectado
    pub target_component: Option<String>,
    /// Contexto adicional
    pub context: HashMap<String, serde_json::Value>,
    /// Etiquetas
    pub tags: Vec<String>,
}

/// Detector de oportunidades
#[derive(Debug, Clone)]
pub struct OpportunityDetector {
    /// Oportunidades activas
    active_opportunities: Arc<RwLock<Vec<Opportunity>>>,
    /// Historial de oportunidades
    opportunity_history: Arc<RwLock<Vec<Opportunity>>>,
    /// Estadísticas
    statistics: Arc<RwLock<OpportunityStatistics>>,
    /// Configuración
    config: Arc<RwLock<OpportunityConfig>>,
}

/// Configuración del detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityConfig {
    /// Sensibilidad del detector
    pub sensitivity: f64,
    /// Mínimo impacto para detectar
    pub minimum_impact_threshold: f64,
    /// Máximo de oportunidades activas
    pub max_active_opportunities: usize,
    /// Fuentes de monitoreo
    pub monitored_sources: Vec<String>,
    /// Período de escaneo (minutos)
    pub scan_period_minutes: u32,
    /// Auto-evaluación de oportunidades
    pub auto_evaluation_enabled: bool,
    /// Tipos de oportunidades a detectar
    pub enabled_opportunity_types: Vec<OpportunityType>,
}

impl Default for OpportunityConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.7,
            minimum_impact_threshold: 15.0,
            max_active_opportunities: 50,
            monitored_sources: vec![
                "system_metrics".to_string(),
                "user_feedback".to_string(),
                "performance_logs".to_string(),
                "error_reports".to_string(),
                "resource_usage".to_string(),
            ],
            scan_period_minutes: 30,
            auto_evaluation_enabled: true,
            enabled_opportunity_types: vec![
                OpportunityType::Technical,
                OpportunityType::Optimization,
                OpportunityType::Innovation,
                OpportunityType::Growth,
            ],
        }
    }
}

/// Estadísticas de oportunidades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityStatistics {
    /// Total de oportunidades detectadas
    pub total_opportunities: u64,
    /// Oportunidades por tipo
    pub opportunities_by_type: HashMap<String, u64>,
    /// Oportunidades por estado
    pub opportunities_by_status: HashMap<String, u64>,
    /// Tasa de éxito (completadas / totales)
    pub success_rate: f64,
    /// Impacto promedio de las aprovechadas
    pub average_realized_impact: f64,
    /// Tiempo promedio de detección a acción
    pub average_time_to_action_hours: f64,
    /// Oportunidades expiradas
    pub expired_opportunities: u64,
    /// Última actualización
    pub last_updated: DateTime<Utc>,
}

impl Default for OpportunityStatistics {
    fn default() -> Self {
        Self {
            total_opportunities: 0,
            opportunities_by_type: HashMap::new(),
            opportunities_by_status: HashMap::new(),
            success_rate: 0.0,
            average_realized_impact: 0.0,
            average_time_to_action_hours: 0.0,
            expired_opportunities: 0,
            last_updated: Utc::now(),
        }
    }
}

impl OpportunityDetector {
    /// Crea un nuevo detector de oportunidades
    pub fn new(config: OpportunityConfig) -> Self {
        Self {
            active_opportunities: Arc::new(RwLock::new(Vec::new())),
            opportunity_history: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(OpportunityStatistics::default())),
            config: Arc::new(RwLock::new(config)),
        }
    }
    
    /// Escanear nuevas oportunidades
    pub async fn scan_opportunities(&self) -> Result<Vec<Opportunity>, ActorError> {
        let config = self.config.read().await;
        let mut opportunities = Vec::new();
        
        // Escanear diferentes fuentes de oportunidades
        for source in &config.monitored_sources {
            match source.as_str() {
                "system_metrics" => {
                    if let Some(opp) = self.detect_system_optimization().await {
                        opportunities.push(opp);
                    }
                }
                "user_feedback" => {
                    if let Some(opp) = self.detect_user_improvement_opportunity().await {
                        opportunities.push(opp);
                    }
                }
                "performance_logs" => {
                    if let Some(opp) = self.detect_performance_opportunity().await {
                        opportunities.push(opp);
                    }
                }
                "error_reports" => {
                    if let Some(opp) = self.detect_error_pattern_opportunity().await {
                        opportunities.push(opp);
                    }
                }
                "resource_usage" => {
                    if let Some(opp) = self.detect_resource_optimization().await {
                        opportunities.push(opp);
                    }
                }
                _ => {}
            }
        }
        
        // Filtrar por umbrales
        opportunities.retain(|opp| {
            config.enabled_opportunity_types.contains(&opp.opportunity_type) &&
            opp.potential_impact >= config.minimum_impact_threshold
        });
        
        // Aplicar sensibilidad
        if config.sensitivity < 1.0 {
            let count = (opportunities.len() as f64 * config.sensitivity).ceil() as usize;
            opportunities.truncate(count);
        }
        
        info!("🔍 Escaneo completado: {} oportunidades detectadas", opportunities.len());
        Ok(opportunities)
    }
    
    /// Agregar oportunidad manualmente
    pub async fn add_opportunity(&self, opportunity: Opportunity) -> Result<(), ActorError> {
        let mut active = self.active_opportunities.write().await;
        
        // Verificar límite
        let config = self.config.read().await;
        if active.len() >= config.max_active_opportunities {
            return Err(ActorError::validation_error(
                GodName::Aurora,
                &format!("Límite de {} oportunidades activas alcanzado", config.max_active_opportunities)
            ));
        }
        
        active.push(opportunity.clone());
        self.update_statistics(&opportunity, true).await;
        
        info!("🔍 Oportunidad añadida: {}", opportunity.title);
        Ok(())
    }
    
    /// Evaluar oportunidad
    pub async fn evaluate_opportunity(&self, opportunity_id: &str, approved: bool) -> Result<(), ActorError> {
        let mut active = self.active_opportunities.write().await;
        
        if let Some(index) = active.iter().position(|o| o.opportunity_id == opportunity_id) {
            let mut opportunity = active.remove(index);
            
            // Actualizar estado
            opportunity.status = if approved {
                OpportunityStatus::Approved
            } else {
                OpportunityStatus::Cancelled
            };
            
            let _opportunity_id_clone = opportunity.opportunity_id.clone();
            let opportunity_title_clone = opportunity.title.clone();
            
            // Re-agregar si fue aprobada
            if approved {
                active.push(opportunity.clone());
            } else {
                // Mover al historial
                let mut history = self.opportunity_history.write().await;
                history.push(opportunity.clone());
            }
            
            self.update_statistics(&opportunity, false).await;
            
            info!("🔍 Oportunidad evaluada {}: {}", opportunity_title_clone, if approved { "APROBADA" } else { "RECHAZADA" });
        } else {
            return Err(ActorError::validation_error(
                GodName::Aurora,
                &format!("Oportunidad {} no encontrada", opportunity_id)
            ));
        }
        
        Ok(())
    }
    
    /// Marcar oportunidad como completada
    pub async fn complete_opportunity(&self, opportunity_id: &str, actual_impact: f64) -> Result<(), ActorError> {
        let mut active = self.active_opportunities.write().await;
        
        if let Some(index) = active.iter().position(|o| o.opportunity_id == opportunity_id) {
            let mut opportunity = active.remove(index);
            opportunity.status = OpportunityStatus::Completed;
            
            // Agregar al historial
            let mut history = self.opportunity_history.write().await;
            history.push(opportunity.clone());
            
            // Actualizar estadísticas de impacto real
            self.update_impact_statistics(actual_impact).await;
            self.update_statistics(&opportunity, false).await;
            
            info!("✅ Oportunidad completada: {} (impacto: {:.1})", opportunity.title, actual_impact);
        } else {
            return Err(ActorError::validation_error(
                GodName::Aurora,
                &format!("Oportunidad {} no encontrada", opportunity_id)
            ));
        }
        
        Ok(())
    }
    
    /// Obtener oportunidades activas
    pub async fn get_active_opportunities(&self) -> Vec<Opportunity> {
        self.active_opportunities.read().await.clone()
    }
    
    /// Obtener historial reciente
    pub async fn get_recent_history(&self, limit: usize) -> Vec<Opportunity> {
        let history = self.opportunity_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
    
    /// Obtener estadísticas
    pub async fn get_statistics(&self) -> OpportunityStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Detectar oportunidad de optimización del sistema
    async fn detect_system_optimization(&self) -> Option<Opportunity> {
        // Simular detección de alta CPU
        if rand::random::<f64>() > 0.7 {
            Some(Opportunity {
                opportunity_id: uuid::Uuid::new_v4().to_string(),
                detected_at: Utc::now(),
                opportunity_type: OpportunityType::Technical,
                priority: OpportunityPriority::High { deadline_hours: 24 },
                status: OpportunityStatus::Detected,
                title: "Optimización de CPU detectada".to_string(),
                description: "Uso elevado de CPU puede ser optimizado mediante cache agresivo".to_string(),
                potential_impact: 65.0,
                estimated_effort_hours: 8.0,
                required_resources: vec!["dev_team".to_string(), "test_environment".to_string()],
                prerequisites: vec!["performance_analysis".to_string()],
                expected_return: Some(120.0),
                time_window_hours: Some(48),
                target_component: Some("core_service".to_string()),
                context: HashMap::from([
                    ("cpu_usage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(85.5).unwrap())),
                    ("recommendation".to_string(), serde_json::Value::String("implement_redis_cache".to_string())),
                ]),
                tags: vec!["optimization".to_string(), "performance".to_string(), "cpu".to_string()],
            })
        } else {
            None
        }
    }
    
    /// Detectar oportunidad basada en feedback de usuario
    async fn detect_user_improvement_opportunity(&self) -> Option<Opportunity> {
        if rand::random::<f64>() > 0.8 {
            Some(Opportunity {
                opportunity_id: uuid::Uuid::new_v4().to_string(),
                detected_at: Utc::now(),
                opportunity_type: OpportunityType::Personal,
                priority: OpportunityPriority::Medium { effort_estimate: 12 },
                status: OpportunityStatus::Detected,
                title: "Mejora en experiencia de usuario".to_string(),
                description: "Patrones de uso sugieren necesidad de mejorar la interfaz de configuración".to_string(),
                potential_impact: 45.0,
                estimated_effort_hours: 16.0,
                required_resources: vec!["ui_team".to_string(), "user_research".to_string()],
                prerequisites: vec!["user_interviews".to_string()],
                expected_return: Some(85.0),
                time_window_hours: Some(72),
                target_component: Some("user_interface".to_string()),
                context: HashMap::from([
                    ("user_satisfaction".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(6.5).unwrap())),
                    ("support_tickets".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(23.0).unwrap())),
                ]),
                tags: vec!["ux".to_string(), "user_satisfaction".to_string(), "interface".to_string()],
            })
        } else {
            None
        }
    }
    
    /// Detectar oportunidad de rendimiento
    async fn detect_performance_opportunity(&self) -> Option<Opportunity> {
        if rand::random::<f64>() > 0.75 {
            Some(Opportunity {
                opportunity_id: uuid::Uuid::new_v4().to_string(),
                detected_at: Utc::now(),
                opportunity_type: OpportunityType::Optimization,
                priority: OpportunityPriority::Low { benefit_ratio: 2.8 },
                status: OpportunityStatus::Detected,
                title: "Optimización de consultas de base de datos".to_string(),
                description: "Consultas N+1 detectadas pueden ser optimizadas con joins eficientes".to_string(),
                potential_impact: 35.0,
                estimated_effort_hours: 6.0,
                required_resources: vec!["database_expert".to_string()],
                prerequisites: vec!["query_analysis".to_string()],
                expected_return: Some(70.0),
                time_window_hours: Some(24),
                target_component: Some("database_layer".to_string()),
                context: HashMap::from([
                    ("slow_queries".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(15.0).unwrap())),
                    ("avg_response_time".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(850.0).unwrap())),
                ]),
                tags: vec!["database".to_string(), "performance".to_string(), "optimization".to_string()],
            })
        } else {
            None
        }
    }
    
    /// Detectar oportunidad en patrones de error
    async fn detect_error_pattern_opportunity(&self) -> Option<Opportunity> {
        if rand::random::<f64>() > 0.85 {
            Some(Opportunity {
                opportunity_id: uuid::Uuid::new_v4().to_string(),
                detected_at: Utc::now(),
                opportunity_type: OpportunityType::Technical,
                priority: OpportunityPriority::Critical { urgency_score: 200 },
                status: OpportunityStatus::Detected,
                title: "Patrón crítico de errores detectado".to_string(),
                description: "Fuga de memoria creciente requiere intervención inmediata".to_string(),
                potential_impact: 90.0,
                estimated_effort_hours: 4.0,
                required_resources: vec!["oncall_engineer".to_string()],
                prerequisites: vec!["emergency_access".to_string()],
                expected_return: Some(200.0),
                time_window_hours: Some(4),
                target_component: Some("memory_management".to_string()),
                context: HashMap::from([
                    ("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(15.0).unwrap())),
                    ("memory_growth".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(50.0).unwrap())),
                ]),
                tags: vec!["critical".to_string(), "memory".to_string(), "emergency".to_string()],
            })
        } else {
            None
        }
    }
    
    /// Detectar oportunidad de optimización de recursos
    async fn detect_resource_optimization(&self) -> Option<Opportunity> {
        if rand::random::<f64>() > 0.6 {
            Some(Opportunity {
                opportunity_id: uuid::Uuid::new_v4().to_string(),
                detected_at: Utc::now(),
                opportunity_type: OpportunityType::Growth,
                priority: OpportunityPriority::Low { benefit_ratio: 1.5 },
                status: OpportunityStatus::Detected,
                title: "Optimización de almacenamiento".to_string(),
                description: "30% del espacio de almacenamiento sin usar puede ser liberado con políticas de retención".to_string(),
                potential_impact: 25.0,
                estimated_effort_hours: 3.0,
                required_resources: vec!["storage_admin".to_string()],
                prerequisites: vec!["storage_audit".to_string()],
                expected_return: Some(40.0),
                time_window_hours: Some(48),
                target_component: Some("storage_system".to_string()),
                context: HashMap::from([
                    ("unused_space".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(30.0).unwrap())),
                    ("storage_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1200.0).unwrap())),
                ]),
                tags: vec!["storage".to_string(), "cost_optimization".to_string(), "resources".to_string()],
            })
        } else {
            None
        }
    }
    
    /// Actualizar estadísticas generales
    async fn update_statistics(&self, opportunity: &Opportunity, is_new: bool) {
        let mut stats = self.statistics.write().await;
        
        if is_new {
            stats.total_opportunities += 1;
        }
        
        // Actualizar contadores por tipo
        let type_key = format!("{:?}", opportunity.opportunity_type);
        *stats.opportunities_by_type.entry(type_key).or_insert(0) += 1;
        
        // Actualizar contadores por estado
        let status_key = format!("{:?}", opportunity.status);
        *stats.opportunities_by_status.entry(status_key).or_insert(0) += 1;
        
        // Actualizar tasa de éxito
        let completed_count = stats.opportunities_by_status.get("Completed").unwrap_or(&0);
        if stats.total_opportunities > 0 {
            stats.success_rate = *completed_count as f64 / stats.total_opportunities as f64;
        }
        
        stats.last_updated = Utc::now();
    }
    
    /// Actualizar estadísticas de impacto real
    async fn update_impact_statistics(&self, actual_impact: f64) {
        let mut stats = self.statistics.write().await;
        
        let completed_count = stats.opportunities_by_status.get("Completed").unwrap_or(&0);
        if *completed_count > 0 {
            stats.average_realized_impact = (stats.average_realized_impact * (*completed_count - 1) as f64 + actual_impact) / *completed_count as f64;
        }
    }
}