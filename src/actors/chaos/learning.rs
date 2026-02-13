// src/actors/chaos/learning.rs
// OLYMPUS v15 - Chaos Learning: Sistema de Aprendizaje Automático para Optimización de Estrategias

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use tracing::info;

use crate::actors::{GodName};
use crate::actors::chaos::{ChaosStrategy, ExperimentResults, ImpactMetrics};

/// Sistema de aprendizaje automático para Chaos
/// 
/// Aprende de los resultados de experimentos para optimizar estrategias
/// y generar insights sobre la resiliencia del sistema
#[derive(Debug, Clone)]
pub struct ChaosLearner {
    /// Historial de experimentos aprendidos
    experiment_history: Arc<RwLock<Vec<LearntExperiment>>>,
    
    /// Base de conocimiento de estrategias
    strategy_knowledge: Arc<RwLock<HashMap<ChaosStrategy, StrategyKnowledge>>>,
    
    /// Patrones de comportamiento aprendidos
    behavior_patterns: Arc<RwLock<Vec<BehaviorPattern>>>,
    
    /// Insights generados
    insights: Arc<RwLock<Vec<ChaosInsight>>>,
    
    /// Configuración del aprendizaje
    config: Arc<RwLock<LearningConfig>>,
}

/// Configuración del sistema de aprendizaje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Mínimo de experimentos para generar insights
    pub min_experiments_for_insights: usize,
    /// Umbral de confianza para recomendaciones
    pub confidence_threshold: f64,
    /// Ventana temporal de aprendizaje (horas)
    pub learning_window_hours: u64,
    /// Máximo de experimentos en historial
    pub max_history_size: usize,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            min_experiments_for_insights: 10,
            confidence_threshold: 0.75,
            learning_window_hours: 24 * 7, // 1 semana
            max_history_size: 1000,
        }
    }
}

/// Experimento aprendido
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearntExperiment {
    pub experiment_id: String,
    pub strategy: ChaosStrategy,
    pub targets: Vec<GodName>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: u64,
    pub success: bool,
    pub impact_metrics: ImpactMetrics,
    pub recovery_time: Option<u64>,
    pub key_findings: Vec<String>,
}

/// Conocimiento sobre una estrategia específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyKnowledge {
    pub strategy: ChaosStrategy,
    pub total_experiments: u32,
    pub successful_experiments: u32,
    pub average_impact: f64,
    pub average_recovery_time: f64,
    pub effectiveness_score: f64,
    pub best_targets: Vec<GodName>,
    pub worst_targets: Vec<GodName>,
    pub learned_patterns: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Patrón de comportamiento detectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_id: String,
    pub description: String,
    pub confidence: f64,
    pub triggers: Vec<String>,
    pub effects: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// Insight generado por el sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosInsight {
    pub insight_id: String,
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub impact_level: String,
    pub actionable_recommendations: Vec<String>,
    pub supporting_evidence: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Categorías de insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    /// Estrategias efectivas
    EffectiveStrategy,
    /// Vulnerabilidades del sistema
    SystemVulnerability,
    /// Patrones de recuperación
    RecoveryPattern,
    /// Optimización de configuración
    ConfigurationOptimization,
    /// Riesgos emergentes
    EmergingRisk,
    /// Mejoras de resiliencia
    ResilienceImprovement,
}

impl ChaosLearner {
    /// Crea una nueva instancia del sistema de aprendizaje
    pub fn new() -> Self {
        Self {
            experiment_history: Arc::new(RwLock::new(Vec::new())),
            strategy_knowledge: Arc::new(RwLock::new(HashMap::new())),
            behavior_patterns: Arc::new(RwLock::new(Vec::new())),
            insights: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(LearningConfig::default())),
        }
    }

    /// Aprende de los resultados de un experimento
    pub async fn learn_from_experiment(&self, results: ExperimentResults) -> Result<(), String> {
        let mut history = self.experiment_history.write().await;
        
        // Crear entrada de aprendizaje
        let learnt = LearntExperiment {
            experiment_id: results.experiment_id.clone(),
            strategy: results.strategy,
            targets: results.affected_actors,
            start_time: results.start_time,
            end_time: results.end_time,
            duration_seconds: results.duration_seconds,
            success: results.success,
            impact_metrics: results.impact_metrics,
            recovery_time: None, // Se calculará más tarde
            key_findings: results.observations,
        };

        // Agregar al historial
        history.push(learnt.clone());
        
        // Limpiar historial si excede el máximo
        if history.len() > self.config.read().await.max_history_size {
            history.remove(0);
        }

        // Actualizar conocimiento de estrategia
        self.update_strategy_knowledge(&learnt).await?;
        
        // Detectar patrones de comportamiento
        self.detect_behavior_patterns(&learnt).await?;
        
        // Generar nuevos insights
        self.generate_insights().await?;

        info!("🧠 Aprendizaje completado para experimento: {}", results.experiment_id);
        
        Ok(())
    }

    /// Actualiza el conocimiento sobre estrategias
    async fn update_strategy_knowledge(&self, experiment: &LearntExperiment) -> Result<(), String> {
        let mut knowledge = self.strategy_knowledge.write().await;
        
        let strategy_knowledge = knowledge.entry(experiment.strategy.clone())
            .or_insert_with(|| StrategyKnowledge {
                strategy: experiment.strategy.clone(),
                total_experiments: 0,
                successful_experiments: 0,
                average_impact: 0.0,
                average_recovery_time: 0.0,
                effectiveness_score: 0.0,
                best_targets: Vec::new(),
                worst_targets: Vec::new(),
                learned_patterns: Vec::new(),
                recommendations: Vec::new(),
            });

        // Actualizar estadísticas básicas
        strategy_knowledge.total_experiments += 1;
        if experiment.success {
            strategy_knowledge.successful_experiments += 1;
        }

        // Actualizar promedios
        let old_count = strategy_knowledge.total_experiments - 1;
        strategy_knowledge.average_impact = 
            (strategy_knowledge.average_impact * old_count as f64 + experiment.impact_metrics.impact_score as f64) 
            / strategy_knowledge.total_experiments as f64;

        // Calcular score de efectividad
        strategy_knowledge.effectiveness_score = 
            strategy_knowledge.successful_experiments as f64 / strategy_knowledge.total_experiments as f64;

        // Actualizar mejores y peores targets
        self.update_target_rankings(strategy_knowledge, experiment).await?;

        // Generar recomendaciones basadas en el nuevo conocimiento
        self.generate_strategy_recommendations(strategy_knowledge).await?;

        Ok(())
    }

    /// Actualiza rankings de targets para una estrategia
    async fn update_target_rankings(&self, knowledge: &mut StrategyKnowledge, experiment: &LearntExperiment) -> Result<(), String> {
        // Implementación simplificada - en un sistema real usaría más métricas
        for target in &experiment.targets {
            if experiment.success {
                if !knowledge.best_targets.contains(target) {
                    knowledge.best_targets.push(target.clone());
                }
            } else {
                if !knowledge.worst_targets.contains(target) {
                    knowledge.worst_targets.push(target.clone());
                }
            }
        }

        Ok(())
    }

    /// Genera recomendaciones para una estrategia
    async fn generate_strategy_recommendations(&self, knowledge: &mut StrategyKnowledge) -> Result<(), String> {
        knowledge.recommendations.clear();

        if knowledge.effectiveness_score > 0.8 {
            knowledge.recommendations.push(
                format!("Estrategia {:?} es altamente efectiva ({:.1}% éxito)", 
                    knowledge.strategy, knowledge.effectiveness_score * 100.0)
            );
        } else if knowledge.effectiveness_score < 0.3 {
            knowledge.recommendations.push(
                format!("Estrategia {:?} tiene baja efectividad ({:.1}% éxito) - reconsiderar uso", 
                    knowledge.strategy, knowledge.effectiveness_score * 100.0)
            );
        }

        if knowledge.average_impact > 7.0 {
            knowledge.recommendations.push(
                format!("Alto impacto detectado - usar con moderación")
            );
        }

        Ok(())
    }

    /// Detecta patrones de comportamiento
    async fn detect_behavior_patterns(&self, experiment: &LearntExperiment) -> Result<(), String> {
        let mut patterns = self.behavior_patterns.write().await;

        // Ejemplo: Detectar patrones de recuperación rápida
        if experiment.success && experiment.duration_seconds < 60 {
            let pattern = BehaviorPattern {
                pattern_id: Uuid::new_v4().to_string(),
                description: "Recuperación rápida ante fallas".to_string(),
                confidence: 0.8,
                triggers: vec![format!("Estrategia: {:?}", experiment.strategy)],
                effects: vec!["Recuperación < 60s".to_string()],
                mitigation_strategies: vec!["Mantener configuración actual".to_string()],
                detected_at: Utc::now(),
            };
            patterns.push(pattern);
        }

        // Ejemplo: Detectar vulnerabilidades específicas
        if experiment.impact_metrics.impact_score > 8 {
            let pattern = BehaviorPattern {
                pattern_id: Uuid::new_v4().to_string(),
                description: "Vulnerabilidad a fallos críticos".to_string(),
                confidence: 0.9,
                triggers: vec![format!("Targets: {:?}", experiment.targets)],
                effects: vec!["Alto impacto del sistema".to_string()],
                mitigation_strategies: vec!["Mejorar resiliencia de componentes críticos".to_string()],
                detected_at: Utc::now(),
            };
            patterns.push(pattern);
        }

        Ok(())
    }

    /// Genera insights basados en el conocimiento acumulado
    async fn generate_insights(&self) -> Result<(), String> {
        let config = self.config.read().await;
        let history = self.experiment_history.read().await;
        
        if history.len() < config.min_experiments_for_insights {
            return Ok(());
        }

        let mut new_insights = Vec::new();

        // Generar insights sobre estrategias efectivas
        new_insights.extend(self.generate_effectiveness_insights().await?);
        
        // Generar insights sobre vulnerabilidades
        new_insights.extend(self.generate_vulnerability_insights().await?);
        
        // Generar insights sobre optimización
        new_insights.extend(self.generate_optimization_insights().await?);

        // Agregar insights nuevos
        let mut insights = self.insights.write().await;
        for insight in new_insights {
            insights.push(insight);
        }

        // Limpiar insights antiguos
        self.cleanup_old_insights().await?;

        Ok(())
    }

    /// Genera insights sobre efectividad de estrategias
    async fn generate_effectiveness_insights(&self) -> Result<Vec<ChaosInsight>, String> {
        let mut insights = Vec::new();
        let knowledge = self.strategy_knowledge.read().await;

        for (strategy, knowledge) in knowledge.iter() {
            if knowledge.total_experiments >= 5 && knowledge.effectiveness_score > 0.8 {
                let insight = ChaosInsight {
                    insight_id: Uuid::new_v4().to_string(),
                    category: InsightCategory::EffectiveStrategy,
                    title: format!("Estrategia {:?} altamente efectiva", strategy),
                    description: format!(
                        "La estrategia {:?} ha mostrado una efectividad del {:.1}% en {} experimentos",
                        strategy, knowledge.effectiveness_score * 100.0, knowledge.total_experiments
                    ),
                    confidence: knowledge.effectiveness_score,
                    impact_level: "Alto".to_string(),
                    actionable_recommendations: vec![
                        format!("Continuar usando {:?} para pruebas de resiliencia", strategy),
                        format!("Considerar {:?} como estrategia primaria para targets: {:?}", 
                            strategy, knowledge.best_targets)
                    ],
                    supporting_evidence: vec![
                        format!("{} experimentos exitosos de {}", 
                            knowledge.successful_experiments, knowledge.total_experiments)
                    ],
                    generated_at: Utc::now(),
                    expires_at: None,
                };
                insights.push(insight);
            }
        }

        Ok(insights)
    }

    /// Genera insights sobre vulnerabilidades del sistema
    async fn generate_vulnerability_insights(&self) -> Result<Vec<ChaosInsight>, String> {
        let mut insights = Vec::new();
        let history = self.experiment_history.read().await;

        // Analizar patrones de fallas por target
        let mut target_failures: HashMap<GodName, u32> = HashMap::new();
        let mut target_experiments: HashMap<GodName, u32> = HashMap::new();

        for experiment in history.iter() {
            for target in &experiment.targets {
                *target_experiments.entry(target.clone()).or_insert(0) += 1;
                if !experiment.success {
                    *target_failures.entry(target.clone()).or_insert(0) += 1;
                }
            }
        }

        // Identificar targets vulnerables
        for (target, failures) in target_failures.iter() {
            if let Some(total) = target_experiments.get(target) {
                let failure_rate = *failures as f64 / *total as f64;
                if failure_rate > 0.7 && *total >= 3 {
                    let insight = ChaosInsight {
                        insight_id: Uuid::new_v4().to_string(),
                        category: InsightCategory::SystemVulnerability,
                        title: format!("Componente {:?} potencialmente vulnerable", target),
                        description: format!(
                            "El componente {:?} ha fallado en {:.1}% de {} experimentos",
                            target, failure_rate * 100.0, total
                        ),
                        confidence: failure_rate,
                        impact_level: if failure_rate > 0.9 { "Crítico" } else { "Alto" }.to_string(),
                        actionable_recommendations: vec![
                            "Revisar implementación del componente".to_string(),
                            "Añadir monitoreo adicional".to_string(),
                            "Considerar refactoring del componente".to_string(),
                        ],
                        supporting_evidence: vec![
                            format!("{} fallas de {} experimentos", failures, total)
                        ],
                        generated_at: Utc::now(),
                        expires_at: None,
                    };
                    insights.push(insight);
                }
            }
        }

        Ok(insights)
    }

    /// Genera insights sobre optimización
    async fn generate_optimization_insights(&self) -> Result<Vec<ChaosInsight>, String> {
        let mut insights = Vec::new();
        let knowledge = self.strategy_knowledge.read().await;

        // Buscar estrategias con bajo impacto y alta efectividad
        for (strategy, strategy_knowledge) in knowledge.iter() {
            if strategy_knowledge.effectiveness_score > 0.8 && strategy_knowledge.average_impact < 3.0 {
                let insight = ChaosInsight {
                    insight_id: Uuid::new_v4().to_string(),
                    category: InsightCategory::ConfigurationOptimization,
                    title: format!("Estrategia {:?} optimizada", strategy),
                    description: format!(
                        "La estrategia {:?} ofrece excelente balance efectividad/impacto",
                        strategy
                    ),
                    confidence: 0.9,
                    impact_level: "Medio".to_string(),
                    actionable_recommendations: vec![
                        "Usar esta estrategia para testing continuo".to_string(),
                        "Considerar como default para pruebas automatizadas".to_string(),
                    ],
                    supporting_evidence: vec![
                        format!("Efectividad: {:.1}%", strategy_knowledge.effectiveness_score * 100.0),
                        format!("Impacto promedio: {:.1}", strategy_knowledge.average_impact)
                    ],
                    generated_at: Utc::now(),
                    expires_at: None,
                };
                insights.push(insight);
            }
        }

        Ok(insights)
    }

    /// Limpia insights antiguos
    async fn cleanup_old_insights(&self) -> Result<(), String> {
        let config = self.config.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(config.learning_window_hours as i64);
        
        let mut insights = self.insights.write().await;
        insights.retain(|insight| {
            insight.generated_at > cutoff_time || 
            insight.expires_at.map_or(true, |exp| exp > Utc::now())
        });

        Ok(())
    }

    /// Obtiene todos los insights disponibles
    pub async fn get_insights(&self) -> Vec<ChaosInsight> {
        let insights = self.insights.read().await;
        insights.clone()
    }

    /// Obtiene conocimiento sobre estrategias
    pub async fn get_strategy_knowledge(&self) -> HashMap<ChaosStrategy, StrategyKnowledge> {
        let knowledge = self.strategy_knowledge.read().await;
        knowledge.clone()
    }

    /// Obtiene patrones de comportamiento detectados
    pub async fn get_behavior_patterns(&self) -> Vec<BehaviorPattern> {
        let patterns = self.behavior_patterns.read().await;
        patterns.clone()
    }

    /// Obtiene recomendaciones personalizadas para un target específico
    pub async fn get_recommendations_for_target(&self, target: GodName) -> Vec<String> {
        let knowledge = self.strategy_knowledge.read().await;
        let mut recommendations = Vec::new();

        for (strategy, strategy_knowledge) in knowledge.iter() {
            if strategy_knowledge.best_targets.contains(&target) {
                recommendations.push(format!(
                    "Usar {:?} - efectividad: {:.1}%", 
                    strategy, strategy_knowledge.effectiveness_score * 100.0
                ));
            } else if strategy_knowledge.worst_targets.contains(&target) {
                recommendations.push(format!(
                    "Evitar {:?} - baja efectividad para este target", 
                    strategy
                ));
            }
        }

        recommendations
    }

    /// Exporta todo el conocimiento aprendido
    pub async fn export_knowledge(&self) -> Result<serde_json::Value, String> {
        let history = self.experiment_history.read().await;
        let knowledge = self.strategy_knowledge.read().await;
        let patterns = self.behavior_patterns.read().await;
        let insights = self.insights.read().await;

        Ok(serde_json::json!({
            "experiment_history": *history,
            "strategy_knowledge": *knowledge,
            "behavior_patterns": *patterns,
            "insights": *insights,
            "exported_at": Utc::now()
        }))
    }

    /// Importa conocimiento aprendido
    pub async fn import_knowledge(&self, data: serde_json::Value) -> Result<(), String> {
        // Implementación para importar conocimiento previamente exportado
        // Esto permitiría persistir y compartir conocimiento entre instancias
        
        if let Ok(new_insights) = serde_json::from_value::<Vec<ChaosInsight>>(data["insights"].clone()) {
            let mut insights = self.insights.write().await;
            insights.extend(new_insights);
        }

        if let Ok(new_patterns) = serde_json::from_value::<Vec<BehaviorPattern>>(data["behavior_patterns"].clone()) {
            let mut patterns = self.behavior_patterns.write().await;
            patterns.extend(new_patterns);
        }

        info!("🧠 Conocimiento importado exitosamente");
        Ok(())
    }
}