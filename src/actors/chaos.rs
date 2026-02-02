/// 🌀 Chaos - Dios del Caos, Testing e Ingeniería del Caos
/// ⚡ Generador de aleatoriedad controlada y pruebas de estrés
/// 🔥 Gestiona Chaos Engineering, pruebas de resiliencia y entropía del sistema

use crate::actors::{OlympianGod, GodName, DivineDomain, OlympicResult, OlympianMessage};
use rand;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 🌀 Experimento de caos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosExperiment {
    pub experiment_id: String,
    pub experiment_type: ChaosExperimentType,
    pub target_system: String,
    pub chaos_intensity: f64,
    pub duration_seconds: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ExperimentStatus,
    pub impact_metrics: HashMap<String, f64>,
    pub hypothesis: String,
}

/// 🌀 Tipos de experimentos de caos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosExperimentType {
    NetworkLatencyInjection,
    ProcessTermination,
    ResourceExhaustion,
    DatabaseConnectionFailure,
    ServiceUnavailability,
    MemoryLeakSimulation,
    DiskSpaceExhaustion,
    RandomFailureInjection,
    TimeDriftSimulation,
    PacketLossInjection,
}

/// 📊 Estados del experimento
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Planned,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// 🎯 Impacto del caos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosImpact {
    pub affected_services: Vec<String>,
    pub error_rate_increase: f64,
    pub response_time_degradation: f64,
    pub user_impact_score: f64,
    pub recovery_time_seconds: u64,
    pub business_cost_estimate: f64,
}

/// 🌀 Configuración de Chaos
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub enable_auto_experiments: bool,
    pub max_chaos_intensity: f64,
    pub safe_mode_enabled: bool,
    pub experiment_budget_per_hour: u32,
    pub minimum_health_threshold: f64,
    pub rollback_strategy: RollbackStrategy,
}

/// 🔄 Estrategias de rollback
#[derive(Debug, Clone)]
pub enum RollbackStrategy {
    Immediate,
    Gradual,
    Manual,
    AutomaticDetection,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enable_auto_experiments: false,
            max_chaos_intensity: 0.8,
            safe_mode_enabled: true,
            experiment_budget_per_hour: 5,
            minimum_health_threshold: 0.7,
            rollback_strategy: RollbackStrategy::AutomaticDetection,
        }
    }
}

/// 📊 Estadísticas del caos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosStatistics {
    pub total_experiments_run: u64,
    pub successful_experiments: u64,
    pub failed_experiments: u64,
    pub average_chaos_intensity: f64,
    pub total_downtime_seconds: u64,
    pub chaos_effectiveness_score: f64,
    pub most_common_failure_type: Option<ChaosExperimentType>,
}

/// 🌀 Chaos V12 - Dios del Caos y Testing
pub struct ChaosV12 {
    name: GodName,
    domain: DivineDomain,
    config: ChaosConfig,
    active_experiments: RwLock<HashMap<String, ChaosExperiment>>,
    experiment_history: RwLock<Vec<ChaosExperiment>>,
    chaos_statistics: RwLock<ChaosStatistics>,
    system_health_baseline: RwLock<HashMap<String, f64>>,
    chaos_monkey: RwLock<ChaosMonkey>,
}

/// 🐵 Mono del caos para inyección aleatoria
#[derive(Debug, Clone)]
pub struct ChaosMonkey {
    pub enabled: bool,
    pub attack_probability: f64,
    pub attack_types: Vec<ChaosExperimentType>,
    pub last_attack: Option<chrono::DateTime<chrono::Utc>>,
}

impl ChaosV12 {
    /// 🌀 Crear nueva instancia de Chaos
    pub fn new() -> Self {
        let initial_stats = ChaosStatistics {
            total_experiments_run: 0,
            successful_experiments: 0,
            failed_experiments: 0,
            average_chaos_intensity: 0.0,
            total_downtime_seconds: 0,
            chaos_effectiveness_score: 0.0,
            most_common_failure_type: None,
        };

        let chaos_monkey = ChaosMonkey {
            enabled: false,
            attack_probability: 0.1,
            attack_types: vec![
                ChaosExperimentType::NetworkLatencyInjection,
                ChaosExperimentType::RandomFailureInjection,
                ChaosExperimentType::ProcessTermination,
            ],
            last_attack: None,
        };

        Self {
            name: GodName::Chaos,
            domain: DivineDomain::ChaosEngineering,
            config: ChaosConfig::default(),
            active_experiments: RwLock::new(HashMap::new()),
            experiment_history: RwLock::new(Vec::new()),
            chaos_statistics: RwLock::new(initial_stats),
            system_health_baseline: RwLock::new(HashMap::new()),
            chaos_monkey: RwLock::new(chaos_monkey),
        }
    }

    /// 🌀 Iniciar experimento de caos
    pub async fn start_chaos_experiment(&self, experiment: ChaosExperiment) -> OlympicResult<String> {
        // Verificar límites y seguridad
        if !self.is_experiment_safe(&experiment).await? {
            return Err("Experimento rechazado por medidas de seguridad".into());
        }

        // Establecer línea base de salud
        self.capture_health_baseline(&experiment.target_system).await?;
        
        let mut active = self.active_experiments.write().await;
        let experiment_id = experiment.experiment_id.clone();
        active.insert(experiment_id.clone(), experiment.clone());

        // Ejecutar experimento
        self.execute_chaos_experiment(&experiment).await?;

        tracing::warn!("🌀 Chaos: Experimento iniciado - {} en {} (intensidad: {:.2})", 
            experiment_id, experiment.target_system, experiment.chaos_intensity);

        Ok(experiment_id)
    }

    /// 🔍 Verificar seguridad del experimento
    async fn is_experiment_safe(&self, experiment: &ChaosExperiment) -> OlympicResult<bool> {
        // Verificar intensidad máxima
        if experiment.chaos_intensity > self.config.max_chaos_intensity {
            return Ok(false);
        }

        // Verificar modo seguro
        if self.config.safe_mode_enabled {
            // Solo permitir experimentos de baja intensidad en modo seguro
            if experiment.chaos_intensity > 0.5 {
                return Ok(false);
            }
        }

        // Verificar salud del sistema
        let baseline = self.system_health_baseline.read().await;
        if let Some(&health) = baseline.get(&experiment.target_system) {
            if health < self.config.minimum_health_threshold {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 📊 Capturar línea base de salud
    async fn capture_health_baseline(&self, system_name: &str) -> OlympicResult<()> {
        let mut baseline = self.system_health_baseline.write().await;
        
        // Simular métricas de salud
        let health_score = 0.95; // Simulación de sistema saludable
        baseline.insert(system_name.to_string(), health_score);
        
        tracing::info!("🌀 Chaos: Línea base capturada para {} - salud: {:.2}", system_name, health_score);
        Ok(())
    }

    /// ⚡ Ejecutar experimento de caos
    async fn execute_chaos_experiment(&self, experiment: &ChaosExperiment) -> OlympicResult<()> {
        match experiment.experiment_type {
            ChaosExperimentType::NetworkLatencyInjection => {
                self.inject_network_latency(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::ProcessTermination => {
                self.simulate_process_termination(&experiment.target_system).await?;
            }
            ChaosExperimentType::ResourceExhaustion => {
                self.simulate_resource_exhaustion(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::DatabaseConnectionFailure => {
                self.simulate_database_failure(&experiment.target_system).await?;
            }
            ChaosExperimentType::ServiceUnavailability => {
                self.simulate_service_unavailability(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::MemoryLeakSimulation => {
                self.simulate_memory_leak(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::DiskSpaceExhaustion => {
                self.simulate_disk_exhaustion(&experiment.target_system).await?;
            }
            ChaosExperimentType::RandomFailureInjection => {
                self.inject_random_failures(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::TimeDriftSimulation => {
                self.simulate_time_drift(&experiment.target_system, experiment.chaos_intensity).await?;
            }
            ChaosExperimentType::PacketLossInjection => {
                self.inject_packet_loss(&experiment.target_system, experiment.chaos_intensity).await?;
            }
        }

        // Iniciar monitoreo del experimento
        self.monitor_experiment_progress(&experiment).await?;

        Ok(())
    }

    /// 🌐 Inyectar latencia de red
    async fn inject_network_latency(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let latency_ms = (intensity * 1000.0) as u64; // Convertir intensidad a milisegundos
        
        tracing::warn!("🌀 Chaos: Inyectando {}ms de latencia en {}", latency_ms, target);
        
        // Simular inyección de latencia
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    /// ⚡ Simular terminación de proceso
    async fn simulate_process_termination(&self, target: &str) -> OlympicResult<()> {
        tracing::error!("🌀 Chaos: Simulando terminación de proceso en {}", target);
        
        // Simular efectos de terminación
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(())
    }

    /// 💾 Simular agotamiento de recursos
    async fn simulate_resource_exhaustion(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let cpu_usage = intensity * 100.0;
        
        tracing::warn!("🌀 Chaos: Simulando agotamiento de recursos en {} - CPU: {:.1}%", target, cpu_usage);
        
        // Simular carga de recursos
        let load_duration = (intensity * 5.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_secs(load_duration)).await;
        
        Ok(())
    }

    /// 🗄️ Simular fallo de base de datos
    async fn simulate_database_failure(&self, target: &str) -> OlympicResult<()> {
        tracing::error!("🌀 Chaos: Simulando fallo de base de datos en {}", target);
        
        // Simular impacto del fallo
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(())
    }

    /// 🚫 Simular indisponibilidad de servicio
    async fn simulate_service_unavailability(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let downtime_seconds = (intensity * 30.0) as u64;
        
        tracing::error!("🌀 Chaos: Simulando indisponibilidad en {} por {} segundos", target, downtime_seconds);
        
        // Simular tiempo de inactividad
        tokio::time::sleep(tokio::time::Duration::from_secs(downtime_seconds.min(5))).await; // Limitado para tests
        
        Ok(())
    }

    /// 🧠 Simular fuga de memoria
    async fn simulate_memory_leak(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let memory_mb = (intensity * 1024.0) as u64;
        
        tracing::warn!("🌀 Chaos: Simulando fuga de memoria en {} - {}MB", target, memory_mb);
        
        // Simular crecimiento de memoria
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        Ok(())
    }

    /// 💿 Simular agotamiento de disco
    async fn simulate_disk_exhaustion(&self, target: &str) -> OlympicResult<()> {
        tracing::warn!("🌀 Chaos: Simulando agotamiento de disco en {}", target);
        
        // Simular operaciones de disco
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        
        Ok(())
    }

    /// 🎲 Inyectar fallos aleatorios
    async fn inject_random_failures(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let failure_count = (intensity * 10.0) as u32;
        
        tracing::warn!("🌀 Chaos: Inyectando {} fallos aleatorios en {}", failure_count, target);
        
        // Simular fallos aleatorios
        for i in 0..failure_count.min(5) { // Limitado para tests
            tracing::error!("🌀 Chaos: Fallo #{} inyectado en {}", i + 1, target);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Ok(())
    }

    /// ⏰ Simular deriva temporal
    async fn simulate_time_drift(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let drift_seconds = (intensity * 60.0) as i64;
        
        tracing::warn!("🌀 Chaos: Simulando deriva temporal en {} - {} segundos", target, drift_seconds);
        
        // Simular efectos de deriva
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(())
    }

    /// 📦 Inyectar pérdida de paquetes
    async fn inject_packet_loss(&self, target: &str, intensity: f64) -> OlympicResult<()> {
        let loss_percentage = intensity * 100.0;
        
        tracing::warn!("🌀 Chaos: Inyectando {:.1}% de pérdida de paquetes en {}", loss_percentage, target);
        
        // Simular pérdida de paquetes
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        Ok(())
    }

    /// 📊 Monitorear progreso del experimento
    async fn monitor_experiment_progress(&self, experiment: &ChaosExperiment) -> OlympicResult<()> {
        let experiment_id = experiment.experiment_id.clone();
        let target_system = experiment.target_system.clone();
        
        // Simular monitoreo asíncrono
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            tracing::info!("🌀 Chaos: Monitoreo completado para experimento {} en {}", experiment_id, target_system);
        });
        
        Ok(())
    }

    /// 🛑 Detener experimento de caos
    pub async fn stop_chaos_experiment(&self, experiment_id: &str) -> OlympicResult<ChaosImpact> {
        let mut active = self.active_experiments.write().await;
        if let Some(mut experiment) = active.remove(experiment_id) {
            experiment.end_time = Some(chrono::Utc::now());
            experiment.status = ExperimentStatus::Completed;
            
            // Calcular impacto
            let impact = self.calculate_chaos_impact(&experiment).await?;
            
            // Mover a historial
            let mut history = self.experiment_history.write().await;
            history.push(experiment.clone());
            
            // Actualizar estadísticas
            self.update_chaos_statistics(&experiment, &impact).await;
            
            tracing::info!("🌀 Chaos: Experimento {} detenido - impacto calculado", experiment_id);
            Ok(impact)
        } else {
            Err("Experimento no encontrado".into())
        }
    }

    /// 📊 Calcular impacto del caos
    async fn calculate_chaos_impact(&self, experiment: &ChaosExperiment) -> OlympicResult<ChaosImpact> {
        let impact = ChaosImpact {
            affected_services: vec![experiment.target_system.clone()],
            error_rate_increase: experiment.chaos_intensity * 0.3,
            response_time_degradation: experiment.chaos_intensity * 0.5,
            user_impact_score: experiment.chaos_intensity * 0.7,
            recovery_time_seconds: (experiment.chaos_intensity * 120.0) as u64,
            business_cost_estimate: experiment.chaos_intensity * 1000.0,
        };
        
        Ok(impact)
    }

    /// 📈 Actualizar estadísticas del caos
    async fn update_chaos_statistics(&self, experiment: &ChaosExperiment, impact: &ChaosImpact) {
        let mut stats = self.chaos_statistics.write().await;
        
        stats.total_experiments_run += 1;
        if experiment.status == ExperimentStatus::Completed {
            stats.successful_experiments += 1;
        } else {
            stats.failed_experiments += 1;
        }
        
        // Actualizar promedio de intensidad
        let total_intensity = stats.average_chaos_intensity * (stats.total_experiments_run - 1) as f64 
            + experiment.chaos_intensity;
        stats.average_chaos_intensity = total_intensity / stats.total_experiments_run as f64;
        
        // Actualizar tiempo de inactividad total
        stats.total_downtime_seconds += impact.recovery_time_seconds;
        
        // Calcular puntuación de efectividad
        stats.chaos_effectiveness_score = if stats.total_experiments_run > 0 {
            stats.successful_experiments as f64 / stats.total_experiments_run as f64
        } else {
            0.0
        };
    }

    /// 🐵 Activar Chaos Monkey
    pub async fn activate_chaos_monkey(&self, attack_probability: f64) -> OlympicResult<()> {
        let mut monkey = self.chaos_monkey.write().await;
        monkey.enabled = true;
        monkey.attack_probability = attack_probability;
        monkey.last_attack = Some(chrono::Utc::now());
        
        tracing::warn!("🌀 Chaos: Chaos Monkey activado con probabilidad de ataque: {:.2}", attack_probability);
        
        // Iniciar ataques aleatorios
        self.start_chaos_monkey_attacks().await?;
        
        Ok(())
    }

    /// 🐵 Iniciar ataques del Chaos Monkey
    async fn start_chaos_monkey_attacks(&self) -> OlympicResult<()> {
        let monkey = self.chaos_monkey.read().await;
        
        if !monkey.enabled {
            return Ok(());
        }
        
        tracing::info!("🌀 Chaos: Iniciando ciclo de ataques del Chaos Monkey");
        
        // Simular ciclo de ataques
        for _ in 0..3 { // Limitado para pruebas
            if rand::random::<f64>() < monkey.attack_probability {
                let attack_type = monkey.attack_types[rand::random::<usize>() % monkey.attack_types.len()].clone();
                let target = "system_under_test";
                
                let experiment = ChaosExperiment {
                    experiment_id: format!("monkey_attack_{:?}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                    experiment_type: attack_type,
                    target_system: target.to_string(),
                    chaos_intensity: 0.3 + rand::random::<f64>() * 0.4,
                    duration_seconds: 30,
                    start_time: chrono::Utc::now(),
                    end_time: None,
                    status: ExperimentStatus::Running,
                    impact_metrics: HashMap::new(),
                    hypothesis: "Chaos Monkey random attack".to_string(),
                };
                
                self.execute_chaos_experiment(&experiment).await?;
            }
        }
        
        Ok(())
    }

    /// 📊 Obtener estadísticas del caos
    pub async fn get_chaos_statistics(&self) -> OlympicResult<ChaosStatistics> {
        let stats = self.chaos_statistics.read().await;
        Ok(stats.clone())
    }

    /// 🛑 Apagar Chaos Monkey
    pub async fn deactivate_chaos_monkey(&self) -> OlympicResult<()> {
        let mut monkey = self.chaos_monkey.write().await;
        monkey.enabled = false;
        
        tracing::info!("🌀 Chaos: Chaos Monkey desactivado");
        Ok(())
    }
}

#[async_trait]
impl OlympianGod for ChaosV12 {
    async fn process_message(&self, message: OlympianMessage) -> OlympicResult<OlympianMessage> {
        match message.command.as_str() {
            "start_experiment" => {
                if let Some(experiment_data) = message.metadata.get("experiment") {
                    let experiment: ChaosExperiment = serde_json::from_value(experiment_data.clone())?;
                    let experiment_id = self.start_chaos_experiment(experiment).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "experiment_started".to_string(),
                        data: serde_json::json!({"experiment_id": experiment_id}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing experiment data".into())
                }
            }
            "stop_experiment" => {
                if let Some(experiment_id) = message.metadata.get("experiment_id").and_then(|e| e.as_str()) {
                    let impact = self.stop_chaos_experiment(experiment_id).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "experiment_stopped".to_string(),
                        data: serde_json::to_value(impact)?,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing experiment_id".into())
                }
            }
            "activate_monkey" => {
                if let Some(probability) = message.metadata.get("probability").and_then(|p| p.as_f64()) {
                    self.activate_chaos_monkey(probability).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "monkey_activated".to_string(),
                        data: serde_json::json!({"probability": probability}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing probability".into())
                }
            }
            "deactivate_monkey" => {
                self.deactivate_chaos_monkey().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "monkey_deactivated".to_string(),
                    data: serde_json::json!({"status": "deactivated"}),
                    metadata: HashMap::new(),
                })
            }
            "get_statistics" => {
                let stats = self.get_chaos_statistics().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "statistics_ready".to_string(),
                    data: serde_json::to_value(stats)?,
                    metadata: HashMap::new(),
                })
            }
            _ => Err(format!("Unknown command: {}", message.command).into()),
        }
    }

    fn get_name(&self) -> GodName {
        self.name.clone()
    }

    fn get_domain(&self) -> DivineDomain {
        self.domain.clone()
    }

    async fn get_status(&self) -> OlympicResult<serde_json::Value> {
        let stats = self.get_chaos_statistics().await?;
        let active_experiments = self.active_experiments.read().await.len();
        let monkey = self.chaos_monkey.read().await;
        
        Ok(serde_json::json!({
            "god": "Chaos",
            "domain": "ChaosEngineering",
            "chaos_statistics": stats,
            "active_experiments_count": active_experiments,
            "chaos_monkey_enabled": monkey.enabled,
            "status": "Creating controlled chaos"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_experiment_creation() {
        let chaos = ChaosV12::new();
        let experiment = ChaosExperiment {
            experiment_id: "test_exp".to_string(),
            experiment_type: ChaosExperimentType::NetworkLatencyInjection,
            target_system: "test_system".to_string(),
            chaos_intensity: 0.5,
            duration_seconds: 60,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: ExperimentStatus::Planned,
            impact_metrics: HashMap::new(),
            hypothesis: "Test hypothesis".to_string(),
        };
        
        let result = chaos.start_chaos_experiment(experiment).await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_chaos_monkey_activation() {
        let chaos = ChaosV12::new();
        let result = chaos.activate_chaos_monkey(0.3).await.unwrap();
        assert_eq!(result, ());
    }
}