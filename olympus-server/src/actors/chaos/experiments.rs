// src/actors/chaos/experiments.rs
// OLYMPUS v15 - Motor de Experimentos Chaos para Chaos

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use tracing::info;

use crate::actors::GodName;
use crate::errors::ActorError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChaosExportFormat {
    Json,
    Csv,
    Yaml,
    Prometheus,
}

/// Estrategias de experimentos Chaos
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChaosStrategy {
    /// Inyección de fallos aleatorios
    RandomFailure,
    
    /// Inyección de latencia
    LatencyInjection,
    
    /// Partición de red
    NetworkPartition,
    
    /// Agotamiento de recursos
    ResourceExhaustion,
    
    /// Fallos en cascada
    CascadingFailure,
    
    /// Degradación gradual
    GradualDegradation,
    
    /// Recuperación forzada
    ForcedRecovery,
    
    /// Combinación de múltiples fallos
    CombinedFailures,
    
    /// Storm de fallos (alta intensidad)
    FailureStorm,
    
    /// Escenario catastrófico
    CatastrophicScenario,
}

impl ChaosStrategy {
    /// Obtiene la descripción de la estrategia
    pub fn description(&self) -> &'static str {
        match self {
            ChaosStrategy::RandomFailure => "Inyección de fallos aleatorios",
            ChaosStrategy::LatencyInjection => "Inyección de latencia artificial",
            ChaosStrategy::NetworkPartition => "Creación de particiones de red",
            ChaosStrategy::ResourceExhaustion => "Agotamiento controlado de recursos",
            ChaosStrategy::CascadingFailure => "Simulación de fallos en cascada",
            ChaosStrategy::GradualDegradation => "Degradación gradual del rendimiento",
            ChaosStrategy::ForcedRecovery => "Forzar procesos de recuperación",
            ChaosStrategy::CombinedFailures => "Combinación de múltiples tipos de fallos",
            ChaosStrategy::FailureStorm => "Tormenta de fallos simultáneos",
            ChaosStrategy::CatastrophicScenario => "Escenario catastrófico controlado",
        }
    }
    
    /// Obtiene el nivel de riesgo base
    pub fn base_risk_level(&self) -> u8 {
        match self {
            ChaosStrategy::RandomFailure => 3,
            ChaosStrategy::LatencyInjection => 2,
            ChaosStrategy::NetworkPartition => 7,
            ChaosStrategy::ResourceExhaustion => 6,
            ChaosStrategy::CascadingFailure => 8,
            ChaosStrategy::GradualDegradation => 4,
            ChaosStrategy::ForcedRecovery => 3,
            ChaosStrategy::CombinedFailures => 7,
            ChaosStrategy::FailureStorm => 9,
            ChaosStrategy::CatastrophicScenario => 10,
        }
    }
    
    /// Obtiene la duración recomendada (segundos)
    pub fn recommended_duration(&self) -> u64 {
        match self {
            ChaosStrategy::RandomFailure => 30,
            ChaosStrategy::LatencyInjection => 60,
            ChaosStrategy::NetworkPartition => 120,
            ChaosStrategy::ResourceExhaustion => 90,
            ChaosStrategy::CascadingFailure => 180,
            ChaosStrategy::GradualDegradation => 300,
            ChaosStrategy::ForcedRecovery => 60,
            ChaosStrategy::CombinedFailures => 150,
            ChaosStrategy::FailureStorm => 240,
            ChaosStrategy::CatastrophicScenario => 300,
        }
    }
}

/// Estado de un experimento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Planeado pero no iniciado
    Planned,
    /// En ejecución
    Running,
    /// Pausado temporalmente
    Paused,
    /// Completado exitosamente
    Completed,
    /// Fallido
    Failed,
    /// Cancelado
    Cancelled,
    /// En espera de aprobación
    PendingApproval,
}

impl std::fmt::Display for ExperimentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperimentStatus::Planned => write!(f, "Planned"),
            ExperimentStatus::Running => write!(f, "Running"),
            ExperimentStatus::Paused => write!(f, "Paused"),
            ExperimentStatus::Completed => write!(f, "Completed"),
            ExperimentStatus::Failed => write!(f, "Failed"),
            ExperimentStatus::Cancelled => write!(f, "Cancelled"),
            ExperimentStatus::PendingApproval => write!(f, "PendingApproval"),
        }
    }
}

/// Experimento Chaos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// ID único del experimento
    pub id: String,
    /// Estrategia utilizada
    pub strategy: ChaosStrategy,
    /// Actores objetivo
    pub targets: Vec<GodName>,
    /// Momento de inicio
    pub start_time: DateTime<Utc>,
    /// Momento de fin (si aplica)
    pub end_time: Option<DateTime<Utc>>,
    /// Duración planificada (segundos)
    pub duration: u64,
    /// Intensidad (0.0 - 1.0)
    pub intensity: f64,
    /// Estado actual
    pub status: ExperimentStatus,
    /// Resultados del experimento
    pub results: Option<ExperimentResults>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
    /// Etapa actual de ejecución
    pub current_phase: Option<String>,
    /// Progresso actual (0-100)
    pub progress: u8,
}

/// Resultados de un experimento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    /// ID del experimento
    pub experiment_id: String,
    /// Si fue exitoso
    pub success: bool,
    /// Tiempo total de ejecución (segundos)
    pub execution_time_seconds: u64,
    /// Número de fallos inyectados
    pub failures_injected: u32,
    /// Sistemas afectados
    pub affected_systems: Vec<String>,
    /// Métricas de impacto
    pub impact_metrics: HashMap<String, f64>,
    /// Observaciones durante el experimento
    pub observations: Vec<String>,
    /// Lecciones aprendidas
    pub lessons_learned: Vec<String>,
    /// Recomendaciones
    pub recommendations: Vec<String>,
    /// Timestamp de finalización
    pub completed_at: DateTime<Utc>,
}

impl Experiment {
    /// Crea un nuevo experimento
    pub fn new(
        strategy: ChaosStrategy,
        targets: Vec<GodName>,
        duration: Option<u64>,
        intensity: f64,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let duration = duration.unwrap_or(strategy.recommended_duration());
        
        Self {
            id: id.clone(),
            strategy,
            targets,
            start_time: Utc::now(),
            end_time: None,
            duration,
            intensity: intensity.clamp(0.0, 1.0),
            status: ExperimentStatus::Planned,
            results: None,
            metadata: HashMap::new(),
            current_phase: None,
            progress: 0,
        }
    }
    
    /// Verifica si el experimento está activo
    pub fn is_active(&self) -> bool {
        matches!(self.status, ExperimentStatus::Running | ExperimentStatus::Paused)
    }
    
    /// Verifica si el experimento está completado
    pub fn is_completed(&self) -> bool {
        matches!(self.status, ExperimentStatus::Completed | ExperimentStatus::Failed | ExperimentStatus::Cancelled)
    }
    
    /// Obtiene el tiempo transcurrido
    pub fn elapsed_time(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(|| Utc::now());
        end - self.start_time
    }
    
    /// Agrega metadato
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Actualiza el progreso
    pub fn update_progress(&mut self, progress: u8, phase: Option<String>) {
        self.progress = progress.min(100);
        self.current_phase = phase;
    }
}

/// Motor de gestión de experimentos
#[derive(Debug, Clone)]
pub struct ExperimentManager {
    /// Experimentos activos
    active_experiments: Arc<RwLock<HashMap<String, Experiment>>>,
    
    /// Historial completo de experimentos
    experiment_history: Arc<RwLock<Vec<Experiment>>>,
    
    /// Plantillas de experimentos predefinidas
    experiment_templates: Arc<RwLock<HashMap<String, ExperimentTemplate>>>,
    
    /// Configuración del manager
    config: Arc<RwLock<ExperimentManagerConfig>>,
    
    /// Estadísticas de ejecución
    stats: Arc<RwLock<ExperimentStats>>,
}

/// Plantilla de experimento predefinido
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentTemplate {
    /// Nombre de la plantilla
    pub name: String,
    /// Descripción
    pub description: String,
    /// Estrategia base
    pub strategy: ChaosStrategy,
    /// Parámetros por defecto
    pub default_parameters: HashMap<String, serde_json::Value>,
    /// Categoría
    pub category: String,
    /// Nivel de riesgo
    pub risk_level: u8,
}

/// Configuración del ExperimentManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentManagerConfig {
    /// Máximo de experimentos concurrentes
    pub max_concurrent_experiments: usize,
    /// Duración máxima de experimentos (segundos)
    pub max_experiment_duration: u64,
    /// Auto-aprobación automática para experimentos de bajo riesgo
    pub auto_approve_low_risk: bool,
    /// Requiere confirmación para experimentos de alto riesgo
    pub require_approval_high_risk: bool,
    /// Umbral de riesgo para aprobación manual
    pub manual_approval_threshold: u8,
    /// Habilitado para producción
    pub production_enabled: bool,
}

impl Default for ExperimentManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_experiments: 5,
            max_experiment_duration: 600, // 10 minutos
            auto_approve_low_risk: true,
            require_approval_high_risk: true,
            manual_approval_threshold: 7,
            production_enabled: false,
        }
    }
}

/// Estadísticas de experimentos
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentStats {
    /// Total de experimentos ejecutados
    pub total_experiments: u64,
    /// Experimentos exitosos
    pub successful_experiments: u64,
    /// Experimentos fallidos
    pub failed_experiments: u64,
    /// Promedio de duración (segundos)
    pub average_duration_seconds: f64,
    /// Promedio de intensidad
    pub average_intensity: f64,
    /// Experimentos por estrategia
    pub experiments_by_strategy: HashMap<String, u64>,
    /// Tiempo total de experimentos
    pub total_experiment_time_seconds: u64,
    /// Impacto promedio
    pub average_impact_score: f64,
}

impl ExperimentManager {
    /// Crea un nuevo gestor de experimentos
    pub fn new() -> Self {
        Self {
            active_experiments: Arc::new(RwLock::new(HashMap::new())),
            experiment_history: Arc::new(RwLock::new(Vec::new())),
            experiment_templates: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(ExperimentManagerConfig::default())),
            stats: Arc::new(RwLock::new(ExperimentStats::default())),
        }
    }
    
    /// Inicia un nuevo experimento
    pub async fn start_experiment(&mut self, mut experiment: Experiment) -> Result<(), ActorError> {
        let config = self.config.read().await;
        
        // Verificar límites
        self.check_experiment_limits(&experiment, &config).await?;
        
        // Cambiar estado a running
        experiment.status = ExperimentStatus::Running;
        experiment.current_phase = Some("Iniciando".to_string());
        experiment.update_progress(0, None);
        
        // Guardar experimento
        {
            let mut active = self.active_experiments.write().await;
            active.insert(experiment.id.clone(), experiment.clone());
        }
        
        // Iniciar ejecución del experimento
        self.execute_experiment(experiment.id.clone()).await?;
        
        info!("🧪 Experimento iniciado: {}", experiment.id);
        Ok(())
    }
    
    /// Detiene un experimento específico
    pub async fn stop_experiment(&mut self, experiment_id: &str) -> Result<ExperimentResults, ActorError> {
        let mut active = self.active_experiments.write().await;
        
        if let Some(mut experiment) = active.remove(experiment_id) {
            experiment.end_time = Some(Utc::now());
            experiment.status = ExperimentStatus::Completed;
            experiment.update_progress(100, Some("Completado".to_string()));
            
            // Generar resultados
            let results = self.generate_experiment_results(&experiment).await?;
            
            // Guardar en historial
            {
                let mut history = self.experiment_history.write().await;
                history.push(experiment.clone());
            }
            
            // Actualizar estadísticas
            self.update_stats(&experiment, &results).await;
            
            info!("🧪 Experimento detenido: {}", experiment_id);
            Ok(results)
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Experimento no encontrado: {}", experiment_id),
            })
        }
    }
    
    /// Detiene todos los experimentos activos
    pub async fn stop_all_experiments(&mut self) -> Result<(), ActorError> {
        let mut active = self.active_experiments.write().await;
        let experiment_ids: Vec<String> = active.keys().cloned().collect();
        
        for experiment_id in experiment_ids {
            if let Some(mut experiment) = active.remove(&experiment_id) {
                experiment.end_time = Some(Utc::now());
                experiment.status = ExperimentStatus::Cancelled;
                experiment.update_progress(0, Some("Cancelado".to_string()));
                
                // Mover al historial
                {
                    let mut history = self.experiment_history.write().await;
                    history.push(experiment);
                }
            }
        }
        
        info!("🧪 Todos los experimentos activos han sido detenidos");
        Ok(())
    }
    
    /// Obtiene experimentos activos
    pub async fn get_active_experiments(&self) -> HashMap<String, Experiment> {
        self.active_experiments.read().await.clone()
    }
    
    /// Obtiene todos los experimentos del historial
    pub async fn get_all_experiments(&self) -> Vec<Experiment> {
        let history = self.experiment_history.read().await;
        history.clone()
    }
    
    /// Obtiene estadísticas de experimentos
    pub async fn get_stats(&self) -> ExperimentStats {
        self.stats.read().await.clone()
    }
    
    /// Limpia experimentos antiguos
    pub async fn cleanup_older_than(&mut self, hours: u64) -> Result<(), ActorError> {
        let cutoff = Utc::now() - Duration::hours(hours as i64);
        
        let mut history = self.experiment_history.write().await;
        let original_len = history.len();
        
        history.retain(|exp| exp.start_time > cutoff);
        
        let removed = original_len - history.len();
        if removed > 0 {
            info!("🧪 Limpiados {} experimentos antiguos", removed);
        }
        
        Ok(())
    }
    
    /// Exporta resultados en formato específico
    pub async fn export_results(&self, format: ChaosExportFormat) -> Result<String, ActorError> {
        let history = self.experiment_history.read().await.clone();
        
        match format {
            ChaosExportFormat::Json => {
                serde_json::to_string_pretty(&history)
                    .map_err(|e| ActorError::Unknown {
                        god: GodName::Chaos,
                        message: format!("Error exportando JSON: {}", e),
                    })
            },
            ChaosExportFormat::Csv => {
                self.export_to_csv(&history).await
            },
            ChaosExportFormat::Yaml => {
                serde_yaml::to_string(&history)
                    .map_err(|e| ActorError::Unknown {
                        god: GodName::Chaos,
                        message: format!("Error exportando YAML: {}", e),
                    })
            },
            ChaosExportFormat::Prometheus => {
                self.export_to_prometheus(&history).await
            },
        }
    }
    
    /// Verifica los límites antes de iniciar experimento
    async fn check_experiment_limits(
        &self,
        experiment: &Experiment,
        config: &ExperimentManagerConfig,
    ) -> Result<(), ActorError> {
        let active = self.active_experiments.read().await;
        
        // Verificar límite de experimentos concurrentes
        if active.len() >= config.max_concurrent_experiments {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: "Límite de experimentos concurrentes alcanzado".to_string(),
            });
        }
        
        // Verificar duración máxima
        if experiment.duration > config.max_experiment_duration {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: "Duración del experimento excede el máximo permitido".to_string(),
            });
        }
        
        // Verificar aprobación para alto riesgo
        let risk_level = experiment.strategy.base_risk_level();
        if risk_level >= config.manual_approval_threshold && config.require_approval_high_risk {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Experimento de alto riesgo requiere aprobación manual (nivel: {})", risk_level),
            });
        }
        
        Ok(())
    }
    
    /// Ejecuta el experimento específico
    async fn execute_experiment(&self, experiment_id: String) -> Result<(), ActorError> {
        // Esta función implementaría la lógica de ejecución real
        // Por ahora, simulamos la ejecución
        
        tokio::spawn(async move {
            // Simular ejecución del experimento
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            info!("🧪 Ejecutando experimento: {}", experiment_id);
            
            // Aquí iría la lógica real de inyección de fallos
            // según la estrategia del experimento
            
        });
        
        Ok(())
    }
    
    /// Genera resultados para un experimento
    async fn generate_experiment_results(&self, experiment: &Experiment) -> Result<ExperimentResults, ActorError> {
        let execution_time = experiment.elapsed_time().num_seconds() as u64;
        
        Ok(ExperimentResults {
            experiment_id: experiment.id.clone(),
            success: experiment.status == ExperimentStatus::Completed,
            execution_time_seconds: execution_time,
            failures_injected: (experiment.intensity * 10.0) as u32, // Simulación
            affected_systems: experiment.targets.iter().map(|t| format!("{:?}", t)).collect(),
            impact_metrics: HashMap::new(), // Se llenaría con métricas reales
            observations: vec![
                format!("Estrategia: {:?}", experiment.strategy),
                format!("Intensidad: {:.2}", experiment.intensity),
                format!("Duración: {}s", execution_time),
            ],
            lessons_learned: vec![
                "El sistema respondió según lo esperado".to_string(),
                "Se identificaron áreas de mejora".to_string(),
            ],
            recommendations: vec![
                "Continuar monitoreando los sistemas afectados".to_string(),
                "Considerar ajustar umbrales de alerta".to_string(),
            ],
            completed_at: Utc::now(),
        })
    }
    
    /// Actualiza estadísticas de experimentos
    async fn update_stats(&self, experiment: &Experiment, results: &ExperimentResults) {
        let mut stats = self.stats.write().await;
        
        stats.total_experiments += 1;
        
        if results.success {
            stats.successful_experiments += 1;
        } else {
            stats.failed_experiments += 1;
        }
        
        // Actualizar promedios
        let total_time = stats.average_duration_seconds * (stats.total_experiments - 1) as f64 + results.execution_time_seconds as f64;
        stats.average_duration_seconds = total_time / stats.total_experiments as f64;
        
        let total_intensity = stats.average_intensity * (stats.total_experiments - 1) as f64 + experiment.intensity;
        stats.average_intensity = total_intensity / stats.total_experiments as f64;
        
        stats.total_experiment_time_seconds += results.execution_time_seconds;
        
        // Actualizar contador por estrategia
        let strategy_key = format!("{:?}", experiment.strategy);
        *stats.experiments_by_strategy.entry(strategy_key).or_insert(0) += 1;
    }
    
    /// Exporta a formato CSV
    async fn export_to_csv(&self, history: &[Experiment]) -> Result<String, ActorError> {
        let mut csv_content = "id,strategy,targets,start_time,end_time,duration,intensity,status\n".to_string();
        
        for experiment in history {
            let targets_str = format!("{:?}", experiment.targets);
            let row = format!(
                "{},{:?},{},{},{},{},{},{}\n",
                experiment.id,
                experiment.strategy,
                targets_str,
                experiment.start_time.format("%Y-%m-%d %H:%M:%S"),
                experiment.end_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "".to_string()),
                experiment.duration,
                experiment.intensity,
                experiment.status
            );
            csv_content.push_str(&row);
        }
        
        Ok(csv_content)
    }
    
    /// Exporta a formato Prometheus
    async fn export_to_prometheus(&self, history: &[Experiment]) -> Result<String, ActorError> {
        let mut prometheus_content = String::new();
        
        // Métricas de conteo
        prometheus_content.push_str("# HELP chaos_experiments_total Total number of chaos experiments\n");
        prometheus_content.push_str("# TYPE chaos_experiments_total counter\n");
        prometheus_content.push_str(&format!("chaos_experiments_total {}\n", history.len()));
        
        // Métricas por estrategia
        prometheus_content.push_str("# HELP chaos_experiments_by_strategy Number of experiments by strategy\n");
        prometheus_content.push_str("# TYPE chaos_experiments_by_strategy gauge\n");
        
        let mut strategy_counts = HashMap::new();
        for experiment in history {
            *strategy_counts.entry(format!("{:?}", experiment.strategy)).or_insert(0) += 1;
        }
        
        for (strategy, count) in strategy_counts {
            prometheus_content.push_str(&format!("chaos_experiments_by_strategy{{strategy=\"{}\"}} {}\n", strategy, count));
        }
        
        Ok(prometheus_content)
    }
    
    /// Obtiene conteos de experimentos
    pub async fn get_active_count(&self) -> usize {
        self.active_experiments.read().await.len()
    }
    
    pub async fn get_total_count(&self) -> usize {
        self.experiment_history.read().await.len()
    }
    
    pub async fn get_successful_count(&self) -> u64 {
        self.stats.read().await.successful_experiments
    }
    
    pub async fn get_failed_count(&self) -> u64 {
        self.stats.read().await.failed_experiments
    }
    
    pub async fn get_average_duration(&self) -> f64 {
        self.stats.read().await.average_duration_seconds
    }
    
    pub async fn get_experiment_results(&self, experiment_id: &str) -> Option<ExperimentResults> {
        let history = self.experiment_history.read().await;
        
        for experiment in history.iter().rev() {
            if experiment.id == experiment_id {
                if let Some(results) = &experiment.results {
                    return Some(results.clone());
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::GodName;

    #[test]
    fn test_experiment_creation() {
        let experiment = Experiment::new(
            ChaosStrategy::RandomFailure,
            vec![GodName::Zeus, GodName::Hades],
            Some(60),
            0.5,
        );
        
        assert_eq!(experiment.strategy, ChaosStrategy::RandomFailure);
        assert_eq!(experiment.targets.len(), 2);
        assert_eq!(experiment.duration, 60);
        assert_eq!(experiment.intensity, 0.5);
        assert_eq!(experiment.status, ExperimentStatus::Planned);
    }

    #[test]
    fn test_chaos_strategy_properties() {
        assert_eq!(ChaosStrategy::NetworkPartition.base_risk_level(), 7);
        assert_eq!(ChaosStrategy::CatastrophicScenario.base_risk_level(), 10);
        assert_eq!(ChaosStrategy::RandomFailure.recommended_duration(), 30);
        assert_eq!(ChaosStrategy::FailureStorm.recommended_duration(), 240);
    }

    #[test]
    fn test_experiment_lifecycle() {
        let mut experiment = Experiment::new(
            ChaosStrategy::LatencyInjection,
            vec![GodName::Athena],
            Some(30),
            0.7,
        );
        
        assert!(!experiment.is_active());
        assert!(!experiment.is_completed());
        
        experiment.status = ExperimentStatus::Running;
        assert!(experiment.is_active());
        assert!(!experiment.is_completed());
        
        experiment.status = ExperimentStatus::Completed;
        assert!(!experiment.is_active());
        assert!(experiment.is_completed());
    }

    #[tokio::test]
    async fn test_experiment_manager() {
        let mut manager = ExperimentManager::new();
        
        assert_eq!(manager.get_active_count().await, 0);
        assert_eq!(manager.get_total_count().await, 0);
        
        let experiment = Experiment::new(
            ChaosStrategy::RandomFailure,
            vec![GodName::Hermes],
            Some(30),
            0.5,
        );
        
        let result = manager.start_experiment(experiment).await;
        assert!(result.is_ok());
        
        assert_eq!(manager.get_active_count().await, 1);
    }
}