// src/actors/chaos/injection.rs
// OLYMPUS v15 - Utilidades Avanzadas de Inyección de Fallos para Chaos

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use tracing::{info, warn};

use crate::actors::{GodName};
use crate::errors::ActorError;

/// Utilidades avanzadas para inyección de fallos
/// 
/// Proporciona mecanismos sofisticados para inyectar fallos
/// de manera controlada y segura en el sistema
#[derive(Debug, Clone)]
pub struct AdvancedFailureInjection {
    /// Configuración de inyección
    config: Arc<RwLock<InjectionConfig>>,
    /// Inyecciones activas
    active_injections: Arc<RwLock<HashMap<String, ActiveInjection>>>,
    /// Historial de inyecciones
    injection_history: Arc<RwLock<Vec<InjectionRecord>>>,
    /// Patrones de inyección predefinidos
    injection_patterns: Arc<RwLock<HashMap<String, InjectionPattern>>>,
}

/// Configuración para inyección avanzada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionConfig {
    /// Máximo de inyecciones concurrentes
    pub max_concurrent_injections: usize,
    /// Duración máxima por defecto (segundos)
    pub default_duration: u64,
    /// Probabilidad base de inyección
    pub base_probability: f64,
    /// Habilitar inyecciones en cascada
    pub enable_cascade_injections: bool,
    /// Máxima profundidad de cascada
    pub max_cascade_depth: u8,
    /// Tipos de fallo permitidos
    pub allowed_failure_types: Vec<AdvancedFailureType>,
    /// Actores protegidos contra inyección
    pub protected_actors: Vec<GodName>,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_injections: 10,
            default_duration: 60, // 1 minuto
            base_probability: 0.05, // 5%
            enable_cascade_injections: true,
            max_cascade_depth: 3,
            allowed_failure_types: vec![
                AdvancedFailureType::MemoryLeak,
                AdvancedFailureType::CpuExhaustion,
                AdvancedFailureType::NetworkPartition,
                AdvancedFailureType::DiskIoDelay,
                AdvancedFailureType::ResourceContention,
            ],
            protected_actors: vec![GodName::Zeus], // Proteger siempre al supervisor
        }
    }
}

/// Tipos avanzados de fallo
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdvancedFailureType {
    /// Fuga de memoria
    MemoryLeak,
    /// Agotamiento de CPU
    CpuExhaustion,
    /// Partición de red
    NetworkPartition,
    /// Retraso en I/O de disco
    DiskIoDelay,
    /// Contención de recursos
    ResourceContention,
    /// Corruption de datos
    DataCorruption,
    /// Deadlock artificial
    ArtificialDeadlock,
    /// Sobrecarga de cola
    QueueOverload,
    /// Falsificación de tiempo
    TimeSkew,
    /// Agotamiento de handles
    HandleExhaustion,
}

/// Patrón de inyección predefinido
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionPattern {
    /// ID único del patrón
    pub pattern_id: String,
    /// Nombre descriptivo
    pub name: String,
    /// Descripción
    pub description: String,
    /// Secuencia de inyecciones
    pub injection_sequence: Vec<PatternStep>,
    /// Condiciones de activación
    pub trigger_conditions: Vec<TriggerCondition>,
    /// Efectos esperados
    pub expected_effects: Vec<String>,
    /// Mitigaciones recomendadas
    pub recommended_mitigations: Vec<String>,
}

/// Paso en un patrón de inyección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStep {
    /// Tipo de fallo a inyectar
    pub failure_type: AdvancedFailureType,
    /// Target del paso
    pub target: GodName,
    /// Duración del paso (segundos)
    pub duration: u64,
    /// Intensidad del fallo (0.0-1.0)
    pub intensity: f64,
    /// Retraso antes de este paso (segundos)
    pub delay_before: u64,
    /// Condiciones para este paso
    pub step_conditions: Vec<String>,
}

/// Condición de activación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Basado en tiempo
    TimeBased {
        /// Hora del día (0-23)
        hour: u8,
        /// Día de la semana (1-7)
        day_of_week: u8,
    },
    /// Basado en carga del sistema
    LoadBased {
        /// Umbral de CPU (%)
        cpu_threshold: f64,
        /// Umbral de memoria (%)
        memory_threshold: f64,
    },
    /// Basado en eventos
    EventBased {
        /// Tipo de evento
        event_type: String,
        /// Contador mínimo
        min_count: u32,
    },
    /// Basado en estado del actor
    ActorStateBased {
        /// Actor objetivo
        target: GodName,
        /// Estado requerido
        required_state: String,
    },
}

/// Inyección activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveInjection {
    /// ID único
    pub injection_id: String,
    /// Tipo de fallo
    pub failure_type: AdvancedFailureType,
    /// Target
    pub target: GodName,
    /// Estado actual
    pub status: InjectionStatus,
    /// Momento de inicio
    pub start_time: DateTime<Utc>,
    /// Duración planificada
    pub planned_duration: u64,
    /// Intensidad
    pub intensity: f64,
    /// Parámetros específicos
    pub parameters: HashMap<String, serde_json::Value>,
    /// Efectos observados
    pub observed_effects: Vec<String>,
    /// Métricas durante la inyección
    pub metrics: InjectionMetrics,
}

/// Estado de una inyección
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionStatus {
    /// Programada
    Scheduled,
    /// Iniciando
    Starting,
    /// Activa
    Active,
    /// Pausada
    Paused,
    /// Deteniendo
    Stopping,
    /// Completada
    Completed,
    /// Falló al iniciar
    Failed,
    /// Cancelada
    Cancelled,
}

/// Métricas durante la inyección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionMetrics {
    /// Tiempo transcurrido (segundos)
    pub elapsed_seconds: u64,
    /// Impacto en latencia (ms)
    pub latency_impact: f64,
    /// Tasa de errores inducida (%)
    pub induced_error_rate: f64,
    /// Recursos consumidos
    pub resources_consumed: HashMap<String, f64>,
    /// Eventos generados
    pub events_generated: u32,
    /// Sistemas afectados
    pub affected_systems: Vec<GodName>,
}

/// Registro histórico de inyección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionRecord {
    /// ID del registro
    pub record_id: String,
    /// ID de la inyección
    pub injection_id: String,
    /// Tipo de fallo
    pub failure_type: AdvancedFailureType,
    /// Target
    pub target: GodName,
    /// Estado final
    pub final_status: InjectionStatus,
    /// Duración real (segundos)
    pub actual_duration: u64,
    /// Intensidad final
    pub final_intensity: f64,
    /// Si tuvo éxito
    pub successful: bool,
    /// Lecciones aprendidas
    pub lessons_learned: Vec<String>,
    /// Timestamp del registro
    pub recorded_at: DateTime<Utc>,
}

impl AdvancedFailureInjection {
    /// Crea una nueva instancia de inyección avanzada
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(InjectionConfig::default())),
            active_injections: Arc::new(RwLock::new(HashMap::new())),
            injection_history: Arc::new(RwLock::new(Vec::new())),
            injection_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Inicializa patrones de inyección predefinidos
    pub async fn initialize_patterns(&self) -> Result<(), ActorError> {
        let mut patterns = self.injection_patterns.write().await;
        
        // Patrón: "Cascada de Memoria"
        patterns.insert("memory_cascade".to_string(), InjectionPattern {
            pattern_id: "memory_cascade".to_string(),
            name: "Cascada de Agotamiento de Memoria".to_string(),
            description: "Inyecta fugas de memoria en cascada para probar límites del sistema".to_string(),
            injection_sequence: vec![
                PatternStep {
                    failure_type: AdvancedFailureType::MemoryLeak,
                    target: GodName::Hestia,
                    duration: 30,
                    intensity: 0.3,
                    delay_before: 0,
                    step_conditions: vec!["memoria disponible > 70%".to_string()],
                },
                PatternStep {
                    failure_type: AdvancedFailureType::MemoryLeak,
                    target: GodName::Athena,
                    duration: 45,
                    intensity: 0.5,
                    delay_before: 10,
                    step_conditions: vec!["fuga anterior activa".to_string()],
                },
                PatternStep {
                    failure_type: AdvancedFailureType::ResourceContention,
                    target: GodName::Hades,
                    duration: 60,
                    intensity: 0.7,
                    delay_before: 20,
                    step_conditions: vec!["memoria del sistema < 30%".to_string()],
                },
            ],
            trigger_conditions: vec![
                TriggerCondition::LoadBased {
                    cpu_threshold: 80.0,
                    memory_threshold: 50.0,
                },
            ],
            expected_effects: vec![
                "Degradación progresiva del rendimiento".to_string(),
                "Aumento en uso de memoria".to_string(),
                "Posibles timeouts en operaciones críticas".to_string(),
            ],
            recommended_mitigations: vec![
                "Implementar límites de memoria por proceso".to_string(),
                "Configurar garbage collection agresivo".to_string(),
                "Habilitar modo degradado automático".to_string(),
            ],
        });
        
        // Patrón: "Partición de Red Controlada"
        patterns.insert("network_partition".to_string(), InjectionPattern {
            pattern_id: "network_partition".to_string(),
            name: "Partición de Red Controlada".to_string(),
            description: "Simula particiones de red para testing de resiliencia".to_string(),
            injection_sequence: vec![
                PatternStep {
                    failure_type: AdvancedFailureType::NetworkPartition,
                    target: GodName::Hermes,
                    duration: 20,
                    intensity: 0.5,
                    delay_before: 0,
                    step_conditions: vec!["conexiones activas > 10".to_string()],
                },
                PatternStep {
                    failure_type: AdvancedFailureType::QueueOverload,
                    target: GodName::Poseidon,
                    duration: 30,
                    intensity: 0.8,
                    delay_before: 5,
                    step_conditions: vec!["partición activa".to_string()],
                },
            ],
            trigger_conditions: vec![
                TriggerCondition::EventBased {
                    event_type: "high_message_volume".to_string(),
                    min_count: 100,
                },
            ],
            expected_effects: vec![
                "Pérdida temporal de conectividad".to_string(),
                "Acumulación de mensajes en colas".to_string(),
                "Activación de mecanismos de retry".to_string(),
            ],
            recommended_mitigations: vec![
                "Implementar circuit breakers".to_string(),
                "Configurar timeouts apropiados".to_string(),
                "Habilitar modo offline".to_string(),
            ],
        });
        
        // Patrón: "Agotamiento de CPU Progresivo"
        patterns.insert("cpu_exhaustion".to_string(), InjectionPattern {
            pattern_id: "cpu_exhaustion".to_string(),
            name: "Agotamiento de CPU Progresivo".to_string(),
            description: "Incrementa gradualmente el uso de CPU para probar auto-escalado".to_string(),
            injection_sequence: vec![
                PatternStep {
                    failure_type: AdvancedFailureType::CpuExhaustion,
                    target: GodName::Apollo,
                    duration: 25,
                    intensity: 0.3,
                    delay_before: 0,
                    step_conditions: vec!["cpu_actual < 70%".to_string()],
                },
                PatternStep {
                    failure_type: AdvancedFailureType::CpuExhaustion,
                    target: GodName::Artemis,
                    duration: 30,
                    intensity: 0.5,
                    delay_before: 15,
                    step_conditions: vec!["cpu_actual > 70%".to_string()],
                },
                PatternStep {
                    failure_type: AdvancedFailureType::CpuExhaustion,
                    target: GodName::Demeter,
                    duration: 35,
                    intensity: 0.7,
                    delay_before: 25,
                    step_conditions: vec!["escalado no activado".to_string()],
                },
            ],
            trigger_conditions: vec![
                TriggerCondition::TimeBased {
                    hour: 14, // 2 PM
                    day_of_week: 2, // Martes
                },
            ],
            expected_effects: vec![
                "Aumento progresivo en uso de CPU".to_string(),
                "Disminución en capacidad de respuesta".to_string(),
                "Posible activación de auto-escalado".to_string(),
            ],
            recommended_mitigations: vec![
                "Configurar límites de CPU por proceso".to_string(),
                "Implementar auto-escalado horizontal".to_string(),
                "Habilitar carga balanceada".to_string(),
            ],
        });
        
        info!("🔪 Patrones de inyección inicializados");
        Ok(())
    }
    
    /// Inyecta un fallo simple
    pub async fn inject_failure(
        &self,
        target: GodName,
        failure_type: AdvancedFailureType,
        duration: Option<u64>,
        intensity: f64,
    ) -> Result<String, ActorError> {
        let config = self.config.read().await;
        
        // Verificar protecciones
        if config.protected_actors.contains(&target) {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Actor protegido contra inyección: {:?}", target),
            });
        }
        
        // Verificar tipo permitido
        if !config.allowed_failure_types.contains(&failure_type) {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Tipo de fallo no permitido: {:?}", failure_type),
            });
        }
        
        // Verificar límite concurrente
        {
            let active = self.active_injections.read().await;
            if active.len() >= config.max_concurrent_injections {
                return Err(ActorError::Unknown {
                    god: GodName::Chaos,
                    message: "Máximo de inyecciones concurrentes alcanzado".to_string(),
                });
            }
        }
        
        let injection_id = Uuid::new_v4().to_string();
        let actual_duration = duration.unwrap_or(config.default_duration);
        
        // Crear inyección activa
        let active_injection = ActiveInjection {
            injection_id: injection_id.clone(),
            failure_type: failure_type.clone(),
            target,
            status: InjectionStatus::Starting,
            start_time: Utc::now(),
            planned_duration: actual_duration,
            intensity,
            parameters: self.generate_failure_parameters(&failure_type, intensity).await,
            observed_effects: Vec::new(),
            metrics: InjectionMetrics {
                elapsed_seconds: 0,
                latency_impact: 0.0,
                induced_error_rate: 0.0,
                resources_consumed: HashMap::new(),
                events_generated: 0,
                affected_systems: Vec::new(),
            },
        };
        
        // Agregar a inyecciones activas
        {
            let mut active = self.active_injections.write().await;
            active.insert(injection_id.clone(), active_injection);
        }
        
        // Iniciar inyección asíncrona
        self.execute_injection(injection_id.clone()).await?;
        
        info!("🔪 Inyección iniciada: {} -> {:?} (intensidad: {:.2})", injection_id, failure_type, intensity);
        Ok(injection_id)
    }
    
    /// Ejecuta un patrón de inyección completo
    pub async fn execute_pattern(
        &self,
        pattern_id: String,
        intensity_multiplier: Option<f64>,
    ) -> Result<Vec<String>, ActorError> {
        let patterns = self.injection_patterns.read().await;
        
        if let Some(pattern) = patterns.get(&pattern_id) {
            let intensity_mult = intensity_multiplier.unwrap_or(1.0);
            let mut injection_ids = Vec::new();
            
            for (step_index, step) in pattern.injection_sequence.iter().enumerate() {
                // Esperar delay_before
                if step.delay_before > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(step.delay_before)).await;
                }
                
                // Verificar condiciones del paso
                if !self.check_step_conditions(&step.step_conditions).await {
                    warn!("🔪 Condiciones no cumplidas para paso {} del patrón {}", step_index, pattern_id);
                    continue;
                }
                
                // Inyectar fallo del paso
                let injection_id = self.inject_failure(
                    step.target,
                    step.failure_type.clone(),
                    Some(step.duration),
                    (step.intensity * intensity_mult).min(1.0),
                ).await?;
                
                injection_ids.push(injection_id);
            }
            
            info!("🔪 Patrón {} ejecutado con {} inyecciones", pattern_id, injection_ids.len());
            Ok(injection_ids)
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Patón no encontrado: {}", pattern_id),
            })
        }
    }
    
    /// Genera parámetros específicos para un tipo de fallo
    async fn generate_failure_parameters(
        &self,
        failure_type: &AdvancedFailureType,
        intensity: f64,
    ) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        
        match failure_type {
            AdvancedFailureType::MemoryLeak => {
                params.insert("leak_rate_mb_per_sec".to_string(), 
                              serde_json::json!(intensity * 10.0));
                params.insert("target_memory_mb".to_string(), 
                              serde_json::json!(intensity * 512.0));
            },
            AdvancedFailureType::CpuExhaustion => {
                params.insert("cpu_cores_to_consume".to_string(), 
                              serde_json::json!((intensity * 4.0).max(1.0)));
                params.insert("consumption_pattern".to_string(), 
                              serde_json::json!("continuous"));
            },
            AdvancedFailureType::NetworkPartition => {
                params.insert("packet_loss_rate".to_string(), 
                              serde_json::json!(intensity * 0.5));
                params.insert("latency_increase_ms".to_string(), 
                              serde_json::json!(intensity * 1000.0));
                params.insert("bandwidth_reduction_percent".to_string(), 
                              serde_json::json!(intensity * 90.0));
            },
            AdvancedFailureType::DiskIoDelay => {
                params.insert("delay_ms".to_string(), 
                              serde_json::json!(intensity * 500.0));
                params.insert("operations_to_delay".to_string(), 
                              serde_json::json!("read_write"));
            },
            AdvancedFailureType::ResourceContention => {
                params.insert("resource_type".to_string(), 
                              serde_json::json!("mutex_semaphore"));
                params.insert("contention_probability".to_string(), 
                              serde_json::json!(intensity));
                params.insert("hold_time_ms".to_string(), 
                              serde_json::json!(intensity * 1000.0));
            },
            AdvancedFailureType::DataCorruption => {
                params.insert("corruption_rate".to_string(), 
                              serde_json::json!(intensity * 0.01));
                params.insert("target_data_types".to_string(), 
                              serde_json::json!(["messages", "cache_entries"]));
            },
            AdvancedFailureType::ArtificialDeadlock => {
                params.insert("lock_acquisition_order".to_string(), 
                              serde_json::json!("random"));
                params.insert("timeout_ms".to_string(), 
                              serde_json::json!(30000));
            },
            AdvancedFailureType::QueueOverload => {
                params.insert("messages_per_second".to_string(), 
                              serde_json::json!(intensity * 1000.0));
                params.insert("target_queue".to_string(), 
                              serde_json::json!("all"));
            },
            AdvancedFailureType::TimeSkew => {
                params.insert("skew_seconds".to_string(), 
                              serde_json::json!(intensity * 3600.0));
                params.insert("skew_direction".to_string(), 
                              serde_json::json!("forward"));
            },
            AdvancedFailureType::HandleExhaustion => {
                params.insert("handles_to_consume".to_string(), 
                              serde_json::json!((intensity * 1000.0) as u32));
                params.insert("handle_types".to_string(), 
                              serde_json::json!(["file", "network", "timer"]));
            },
        }
        
        params
    }
    
    /// Verifica las condiciones para un paso
    async fn check_step_conditions(&self, conditions: &[String]) -> bool {
        // Implementación simplificada - en un sistema real verificaría
        // condiciones reales del sistema
        for condition in conditions {
            if condition.contains("memoria") {
                // Simular verificación de memoria
                continue;
            }
            if condition.contains("fuga") {
                // Simular verificación de fugas activas
                continue;
            }
            if condition.contains("cpu") {
                // Simular verificación de CPU
                continue;
            }
        }
        true
    }
    
    /// Ejecuta una inyección de forma asíncrona
    async fn execute_injection(&self, injection_id: String) -> Result<(), ActorError> {
        let active_injections = self.active_injections.clone();
        let failure_type = {
            let active = active_injections.read().await;
            if let Some(injection) = active.get(&injection_id) {
                injection.failure_type.clone()
            } else {
                return Err(ActorError::Unknown {
                    god: GodName::Chaos,
                    message: "Inyección no encontrada".to_string(),
                });
            }
        };
        
        let injection_id_clone = injection_id.clone();
        
        tokio::spawn(async move {
            // Simular ejecución del fallo
            let execution_time = match failure_type {
                AdvancedFailureType::MemoryLeak => 2000,
                AdvancedFailureType::CpuExhaustion => 5000,
                AdvancedFailureType::NetworkPartition => 3000,
                AdvancedFailureType::DiskIoDelay => 1500,
                _ => 1000,
            };
            
            tokio::time::sleep(tokio::time::Duration::from_millis(execution_time)).await;
            
            // Actualizar estado a activo
            {
                let mut active = active_injections.write().await;
                if let Some(injection) = active.get_mut(&injection_id_clone) {
                    injection.status = InjectionStatus::Active;
                    injection.metrics.elapsed_seconds = 1;
                }
            }
            
            // Simular duración de la inyección
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Finalizar inyección
            {
                let mut active = active_injections.write().await;
                if let Some(injection) = active.get_mut(&injection_id_clone) {
                    injection.status = InjectionStatus::Completed;
                    injection.metrics.elapsed_seconds = 6;
                }
            }
        });
        
        Ok(())
    }
    
    /// Detiene una inyección activa
    pub async fn stop_injection(&self, injection_id: &str) -> Result<(), ActorError> {
        let mut active = self.active_injections.write().await;
        
        if let Some(injection) = active.get_mut(injection_id) {
            injection.status = InjectionStatus::Stopping;
            info!("🔪 Deteniendo inyección: {}", injection_id);
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Inyección no encontrada: {}", injection_id),
            })
        }
    }
    
    /// Obtiene el estado de una inyección activa
    pub async fn get_injection_status(&self, injection_id: &str) -> Option<ActiveInjection> {
        let active = self.active_injections.read().await;
        active.get(injection_id).cloned()
    }
    
    /// Obtiene todas las inyecciones activas
    pub async fn get_active_injections(&self) -> HashMap<String, ActiveInjection> {
        self.active_injections.read().await.clone()
    }
    
    /// Obtiene patrones disponibles
    pub async fn get_available_patterns(&self) -> HashMap<String, InjectionPattern> {
        self.injection_patterns.read().await.clone()
    }
    
    /// Limpia inyecciones antiguas
    pub async fn cleanup_old_injections(&self, older_than_minutes: u64) {
        let cutoff = Utc::now() - chrono::Duration::minutes(older_than_minutes as i64);
        
        {
            let mut active = self.active_injections.write().await;
            let original_len = active.len();
            active.retain(|_, injection| {
                injection.start_time > cutoff && injection.status == InjectionStatus::Active
            });
            
            let removed = original_len - active.len();
            if removed > 0 {
                info!("🔪 {} inyecciones antiguas removidas", removed);
            }
        }
        
        // Mover inyecciones completadas al historial
        {
            let mut active = self.active_injections.write().await;
            let mut history = self.injection_history.write().await;
            
            let completed: Vec<_> = active.drain()
                .filter(|(_, injection)| matches!(injection.status, InjectionStatus::Completed | InjectionStatus::Failed))
                .map(|(id, injection)| {
                    let record = InjectionRecord {
                        record_id: Uuid::new_v4().to_string(),
                        injection_id: id.clone(),
                        failure_type: injection.failure_type,
                        target: injection.target,
                        final_status: injection.status.clone(),
                        actual_duration: injection.metrics.elapsed_seconds,
                        final_intensity: injection.intensity,
                        successful: matches!(injection.status, InjectionStatus::Completed),
                        lessons_learned: injection.observed_effects,
                        recorded_at: Utc::now(),
                    };
                    record
                })
                .collect();
            
            history.extend(completed);
        }
    }
    
    /// Obtiene estadísticas de inyecciones
    pub async fn get_injection_stats(&self) -> serde_json::Value {
        let active = self.active_injections.read().await;
        let history = self.injection_history.read().await;
        
        let total_injections = history.len();
        let successful_injections = history.iter().filter(|r| r.successful).count();
        let active_count = active.len();
        
        let success_rate = if total_injections > 0 {
            successful_injections as f64 / total_injections as f64 * 100.0
        } else {
            0.0
        };
        
        // Contar por tipo de fallo
        let mut failure_type_counts: HashMap<String, u32> = HashMap::new();
        for injection in active.values() {
            let type_name = format!("{:?}", injection.failure_type);
            *failure_type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        serde_json::json!({
            "total_injections": total_injections,
            "successful_injections": successful_injections,
            "active_injections": active_count,
            "success_rate_percentage": success_rate,
            "failure_type_distribution": failure_type_counts,
            "available_patterns": self.injection_patterns.read().await.len()
        })
    }
}