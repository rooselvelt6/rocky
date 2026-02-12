// src/actors/chaos/recovery.rs
// OLYMPUS v15 - Sistema de Recuperación para Chaos Engineering

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use tracing::{info, warn};

use crate::actors::{GodName, DivineDomain};
use crate::errors::ActorError;

/// Sistema de recuperación automática para Chaos
/// 
/// Implementa estrategias de recuperación cuando se detectan fallos
/// o degradaciones del sistema
#[derive(Debug, Clone)]
pub struct RecoverySystem {
    /// Estrategias de recuperación configuradas
    strategies: Arc<RwLock<HashMap<RecoveryStrategyType, RecoveryStrategy>>>,
    /// Recuperaciones activas
    active_recoveries: Arc<RwLock<HashMap<String, ActiveRecovery>>>,
    /// Historial de recuperaciones
    recovery_history: Arc<RwLock<Vec<RecoveryRecord>>>,
    /// Configuración del sistema
    config: Arc<RwLock<RecoveryConfig>>,
    /// Canal de eventos de recuperación
    recovery_tx: mpsc::Sender<RecoveryEvent>,
}

/// Configuración del sistema de recuperación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Máximo de recuperaciones concurrentes
    pub max_concurrent_recoveries: usize,
    /// Tiempo máximo de recuperación (segundos)
    pub max_recovery_time: u64,
    /// Intervalo de verificación de recuperación (milisegundos)
    pub recovery_check_interval: u64,
    /// Estrategias permitidas
    pub allowed_strategies: Vec<RecoveryStrategyType>,
    /// Umbral de confianza para recuperación automática
    pub auto_recovery_threshold: f64,
    /// Habilitar recuperación automática
    pub enable_auto_recovery: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_concurrent_recoveries: 5,
            max_recovery_time: 300, // 5 minutos
            recovery_check_interval: 2000, // 2 segundos
            allowed_strategies: vec![
                RecoveryStrategyType::RestartActor,
                RecoveryStrategyType::ScaleUp,
                RecoveryStrategyType::Failover,
                RecoveryStrategyType::CircuitBreaker,
            ],
            auto_recovery_threshold: 0.8,
            enable_auto_recovery: true,
        }
    }
}

/// Tipos de estrategias de recuperación
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryStrategyType {
    /// Reiniciar el actor afectado
    RestartActor,
    /// Escalar horizontalmente
    ScaleUp,
    /// Cambiar a servidor secundario
    Failover,
    /// Activar circuit breaker
    CircuitBreaker,
    /// Reducir carga (throttling)
    ThrottleLoad,
    /// Cambiar a modo degradado
    DegradedMode,
    /// Purgar caché
    PurgeCache,
    /// Reconectar servicios
    ReconnectServices,
    /// Restaurar desde backup
    RestoreBackup,
    /// Reiniciar componentes críticos
    RestartCriticalComponents,
}

/// Estrategia de recuperación específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Tipo de estrategia
    pub strategy_type: RecoveryStrategyType,
    /// Nombre descriptivo
    pub name: String,
    /// Descripción
    pub description: String,
    /// Prioridad (menor = mayor prioridad)
    pub priority: u8,
    /// Tiempo estimado de ejecución (segundos)
    pub estimated_duration: u64,
    /// Probabilidad de éxito (0.0-1.0)
    pub success_probability: f64,
    /// Efectos secundarios conocidos
    pub side_effects: Vec<String>,
    /// Prerrequisitos
    pub prerequisites: Vec<String>,
}

/// Recuperación activa en progreso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRecovery {
    /// ID único de la recuperación
    pub recovery_id: String,
    /// ID del experimento que causó el problema
    pub experiment_id: String,
    /// Actor/Componente afectado
    pub target: GodName,
    /// Estrategia being utilizada
    pub strategy: RecoveryStrategyType,
    /// Estado actual
    pub status: RecoveryStatus,
    /// Momento de inicio
    pub start_time: DateTime<Utc>,
    /// Última actualización
    pub last_update: DateTime<Utc>,
    /// Pasos completados
    pub completed_steps: Vec<String>,
    /// Pasos pendientes
    pub pending_steps: Vec<String>,
    /// Errores encontrados
    pub errors: Vec<String>,
    /// Métricas de recuperación
    pub metrics: RecoveryMetrics,
}

/// Estado de una recuperación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    /// Iniciando
    Initializing,
    /// En progreso
    InProgress,
    /// Verificando
    Verifying,
    /// Completada exitosamente
    Completed,
    /// Falló
    Failed,
    /// Cancelada
    Cancelled,
    /// Timeout
    Timeout,
}

/// Métricas de recuperación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    /// Tiempo transcurrido (segundos)
    pub elapsed_seconds: u64,
    /// Porcentaje de progreso
    pub progress_percentage: f64,
    /// Recursos utilizados
    pub resources_used: HashMap<String, f64>,
    /// Impacto del sistema durante recuperación
    pub system_impact: f64,
    /// Tiempo estimado restante
    pub estimated_remaining_seconds: u64,
}

/// Registro histórico de recuperaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecord {
    /// ID único
    pub id: String,
    /// ID de la recuperación
    pub recovery_id: String,
    /// Experimento relacionado
    pub experiment_id: String,
    /// Target
    pub target: GodName,
    /// Estrategia utilizada
    pub strategy: RecoveryStrategyType,
    /// Estado final
    pub final_status: RecoveryStatus,
    /// Duración total (segundos)
    pub duration_seconds: u64,
    /// Si fue exitosa
    pub successful: bool,
    /// Lecciones aprendidas
    pub lessons_learned: Vec<String>,
    /// Timestamp del registro
    pub recorded_at: DateTime<Utc>,
}

/// Eventos del sistema de recuperación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryEvent {
    /// Recuperación iniciada
    RecoveryStarted {
        recovery_id: String,
        strategy: RecoveryStrategyType,
        target: GodName,
    },
    /// Progreso de recuperación
    RecoveryProgress {
        recovery_id: String,
        step: String,
        progress_percentage: f64,
    },
    /// Recuperación completada
    RecoveryCompleted {
        recovery_id: String,
        success: bool,
        duration_seconds: u64,
    },
    /// Recuperación falló
    RecoveryFailed {
        recovery_id: String,
        error: String,
        completed_steps: u32,
    },
    /// Estrategia adaptada
    StrategyAdapted {
        recovery_id: String,
        old_strategy: RecoveryStrategyType,
        new_strategy: RecoveryStrategyType,
        reason: String,
    },
}

impl RecoverySystem {
    /// Crea un nuevo sistema de recuperación
    pub fn new() -> Self {
        let (recovery_tx, _) = mpsc::channel(1000);
        
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            active_recoveries: Arc::new(RwLock::new(HashMap::new())),
            recovery_history: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(RecoveryConfig::default())),
            recovery_tx,
        }
    }
    
    /// Inicializa las estrategias de recuperación por defecto
    pub async fn initialize_strategies(&self) -> Result<(), ActorError> {
        let mut strategies = self.strategies.write().await;
        
        // Estrategia: Reiniciar Actor
        strategies.insert(RecoveryStrategyType::RestartActor, RecoveryStrategy {
            strategy_type: RecoveryStrategyType::RestartActor,
            name: "Reinicio de Actor".to_string(),
            description: "Reinicia el actor afectado para limpiar estado".to_string(),
            priority: 1,
            estimated_duration: 30,
            success_probability: 0.9,
            side_effects: vec!["Pérdida de estado temporal".to_string()],
            prerequisites: vec!["Actor debe ser reiniciable".to_string()],
        });
        
        // Estrategia: Escalar Horizontalmente
        strategies.insert(RecoveryStrategyType::ScaleUp, RecoveryStrategy {
            strategy_type: RecoveryStrategyType::ScaleUp,
            name: "Escalamiento Horizontal".to_string(),
            description: "Añade instancias adicionales para distribuir carga".to_string(),
            priority: 2,
            estimated_duration: 120,
            success_probability: 0.85,
            side_effects: vec!["Mayor consumo de recursos".to_string()],
            prerequisites: vec!["Sistema debe soportar escalado".to_string()],
        });
        
        // Estrategia: Failover
        strategies.insert(RecoveryStrategyType::Failover, RecoveryStrategy {
            strategy_type: RecoveryStrategyType::Failover,
            name: "Failover a Secundario".to_string(),
            description: "Cambia a servidor secundario para mantener servicio".to_string(),
            priority: 3,
            estimated_duration: 60,
            success_probability: 0.95,
            side_effects: vec!["Possible pérdida de datos recientes".to_string()],
            prerequisites: vec!["Servidor secundario disponible".to_string()],
        });
        
        // Estrategia: Circuit Breaker
        strategies.insert(RecoveryStrategyType::CircuitBreaker, RecoveryStrategy {
            strategy_type: RecoveryStrategyType::CircuitBreaker,
            name: "Circuit Breaker".to_string(),
            description: "Activa modo de aislamiento para prevenir fallos en cascada".to_string(),
            priority: 4,
            estimated_duration: 10,
            success_probability: 0.8,
            side_effects: vec!["Servicio temporalmente no disponible".to_string()],
            prerequisites: vec!["Componente debe soportar aislamiento".to_string()],
        });
        
        info!("🔄 Estrategias de recuperación inicializadas");
        Ok(())
    }
    
    /// Inicia recuperación automática para un experimento fallido
    pub async fn start_auto_recovery(
        &self,
        experiment_id: String,
        target: GodName,
        impact_level: String,
    ) -> Result<String, ActorError> {
        let config = self.config.read().await;
        
        if !config.enable_auto_recovery {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: "Recuperación automática deshabilitada".to_string(),
            });
        }
        
        // Seleccionar estrategia basada en el impacto
        let strategy = self.select_recovery_strategy(&impact_level).await?;
        
        self.start_recovery(experiment_id, target, strategy).await
    }
    
    /// Inicia una recuperación específica
    pub async fn start_recovery(
        &self,
        experiment_id: String,
        target: GodName,
        strategy: RecoveryStrategyType,
    ) -> Result<String, ActorError> {
        let recovery_id = Uuid::new_v4().to_string();
        
        // Verificar límites concurrentes
        {
            let active_recoveries = self.active_recoveries.read().await;
            if active_recoveries.len() >= self.config.read().await.max_concurrent_recoveries {
                return Err(ActorError::Unknown {
                    god: GodName::Chaos,
                    message: "Máximo de recuperaciones concurrentes alcanzado".to_string(),
                });
            }
        }
        
        // Verificar que la estrategia está permitida
        {
            let config = self.config.read().await;
            if !config.allowed_strategies.contains(&strategy) {
                return Err(ActorError::Unknown {
                    god: GodName::Chaos,
                    message: format!("Estrategia no permitida: {:?}", strategy),
                });
            }
        }
        
        // Crear recuperación activa
        let active_recovery = ActiveRecovery {
            recovery_id: recovery_id.clone(),
            experiment_id: experiment_id.clone(),
            target,
            strategy: strategy.clone(),
            status: RecoveryStatus::Initializing,
            start_time: Utc::now(),
            last_update: Utc::now(),
            completed_steps: Vec::new(),
            pending_steps: self.get_recovery_steps(&strategy).await,
            errors: Vec::new(),
            metrics: RecoveryMetrics {
                elapsed_seconds: 0,
                progress_percentage: 0.0,
                resources_used: HashMap::new(),
                system_impact: 0.0,
                estimated_remaining_seconds: 0,
            },
        };
        
        // Agregar a recuperaciones activas
        {
            let mut active_recoveries = self.active_recoveries.write().await;
            active_recoveries.insert(recovery_id.clone(), active_recovery);
        }
        
        // Emitir evento de inicio
        self.emit_event(RecoveryEvent::RecoveryStarted {
            recovery_id: recovery_id.clone(),
            strategy,
            target,
        }).await;
        
        // Iniciar ejecución asíncrona
        self.execute_recovery(recovery_id.clone()).await?;
        
        info!("🔄 Recuperación iniciada: {} para experimento {}", recovery_id, experiment_id);
        Ok(recovery_id)
    }
    
    /// Selecciona la mejor estrategia de recuperación basada en el impacto
    async fn select_recovery_strategy(&self, impact_level: &str) -> Result<RecoveryStrategyType, ActorError> {
        match impact_level.to_lowercase().as_str() {
            "minimal" | "low" => Ok(RecoveryStrategyType::CircuitBreaker),
            "medium" => Ok(RecoveryStrategyType::ThrottleLoad),
            "high" => Ok(RecoveryStrategyType::RestartActor),
            "critical" | "catastrophic" => Ok(RecoveryStrategyType::Failover),
            _ => Ok(RecoveryStrategyType::RestartActor), // Default
        }
    }
    
    /// Obtiene los pasos para una estrategia de recuperación
    async fn get_recovery_steps(&self, strategy: &RecoveryStrategyType) -> Vec<String> {
        match strategy {
            RecoveryStrategyType::RestartActor => vec![
                "Verificar estado actual del actor".to_string(),
                "Guardar estado crítico si es posible".to_string(),
                "Detener actor gracefulmente".to_string(),
                "Limpiar recursos asociados".to_string(),
                "Reiniciar actor con configuración limpia".to_string(),
                "Verificar funcionamiento normal".to_string(),
            ],
            RecoveryStrategyType::ScaleUp => vec![
                "Verificar disponibilidad de recursos".to_string(),
                "Identificar componentes para escalar".to_string(),
                "Provisionar nuevas instancias".to_string(),
                "Configurar balanceo de carga".to_string(),
                "Verificar distribución correcta".to_string(),
            ],
            RecoveryStrategyType::Failover => vec![
                "Verificar estado de servidor secundario".to_string(),
                "Sincronizar estado actual".to_string(),
                "Redirigir tráfico al secundario".to_string(),
                "Verificar funcionamiento del secundario".to_string(),
                "Marcar primario como en recuperación".to_string(),
            ],
            RecoveryStrategyType::CircuitBreaker => vec![
                "Identificar punto de fallo".to_string(),
                "Activar modo de aislamiento".to_string(),
                "Redirigir tráfico alternativo".to_string(),
                "Monitorizar sistema aislado".to_string(),
            ],
            RecoveryStrategyType::ThrottleLoad => vec![
                "Analizar carga actual del sistema".to_string(),
                "Identificar servicios críticos vs secundarios".to_string(),
                "Implementar límites de tasa".to_string(),
                "Priorizar tráfico importante".to_string(),
                "Monitorizar rendimiento con throttling".to_string(),
            ],
            RecoveryStrategyType::DegradedMode => vec![
                "Identificar funcionalidades no críticas".to_string(),
                "Desactivar servicios secundarios".to_string(),
                "Mantener solo funcionalidades esenciales".to_string(),
                "Notificar modo degradado a usuarios".to_string(),
            ],
            RecoveryStrategyType::PurgeCache => vec![
                "Identificar cachés afectadas".to_string(),
                "Verificar integridad de datos".to_string(),
                "Limpiar cachés corrompidas".to_string(),
                "Reconstruir caché si es necesario".to_string(),
            ],
            RecoveryStrategyType::ReconnectServices => vec![
                "Identificar conexiones rotas".to_string(),
                "Reiniciar servicios de red".to_string(),
                "Reestablecer conexiones".to_string(),
                "Verificar comunicación correcta".to_string(),
            ],
            RecoveryStrategyType::RestoreBackup => vec![
                "Identificar backup más reciente".to_string(),
                "Verificar integridad del backup".to_string(),
                "Preparar sistema para restauración".to_string(),
                "Restaurar desde backup".to_string(),
                "Verificar sistema restaurado".to_string(),
            ],
            RecoveryStrategyType::RestartCriticalComponents => vec![
                "Identificar componentes críticos afectados".to_string(),
                "Planificar orden de reinicio".to_string(),
                "Reiniciar componentes en orden correcto".to_string(),
                "Verificar funcionamiento del sistema".to_string(),
            ],
        }
    }
    
    /// Ejecuta una recuperación de forma asíncrona
    async fn execute_recovery(&self, recovery_id: String) -> Result<(), ActorError> {
        let recovery_id_clone = recovery_id.clone();
        let active_recoveries = self.active_recoveries.clone();
        let recovery_tx = self.recovery_tx.clone();
        
        tokio::spawn(async move {
            let mut current_step = 0;
            let mut success = false;
            
            loop {
                // Obtener estado actual de la recuperación
                let (strategy, mut recovery_state) = {
                    let active_recoveries = active_recoveries.read().await;
                    if let Some(recovery) = active_recoveries.get(&recovery_id_clone) {
                        (recovery.strategy.clone(), recovery.clone())
                    } else {
                        break; // Recuperación no encontrada
                    }
                };
                
                // Verificar timeout
                let elapsed = Utc::now() - recovery_state.start_time;
                if elapsed.num_seconds() > 300 { // 5 minutos timeout
                    recovery_state.status = RecoveryStatus::Timeout;
                    break;
                }
                
                // Ejecutar siguiente paso
                if current_step < recovery_state.pending_steps.len() {
                    let step = &recovery_state.pending_steps[current_step];
                    
                    // Simular ejecución del paso
                    let step_success = Self::execute_step(&strategy, step).await;
                    
                    if step_success {
                        // Actualizar progreso
                        {
                            let mut active_recoveries = active_recoveries.write().await;
                            if let Some(recovery) = active_recoveries.get_mut(&recovery_id_clone) {
                                recovery.completed_steps.push(step.clone());
                                recovery.pending_steps.remove(0);
                                recovery.last_update = Utc::now();
                                
                                let total_steps = recovery.completed_steps.len() + recovery.pending_steps.len();
                                recovery.metrics.progress_percentage = 
                                    (recovery.completed_steps.len() as f64 / total_steps as f64) * 100.0;
                                recovery.metrics.elapsed_seconds = (Utc::now() - recovery.start_time).num_seconds() as u64;
                            }
                        }
                        
                        current_step += 1;
                    } else {
                        // Paso falló
                        recovery_state.status = RecoveryStatus::Failed;
                        break;
                    }
                } else {
                    // Todos los pasos completados
                    recovery_state.status = RecoveryStatus::Completed;
                    success = true;
                    break;
                }
                
                // Pequeña pausa entre pasos
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            
            // Actualizar estado final
            {
                let mut active_recoveries = active_recoveries.write().await;
                if let Some(recovery) = active_recoveries.get_mut(&recovery_id_clone) {
                    recovery.status = if success { RecoveryStatus::Completed } else { RecoveryStatus::Failed };
                    recovery.metrics.elapsed_seconds = ((Utc::now() - recovery.start_time).num_seconds()) as u64;
                }
            }
            
            // Emitir evento de completado
            let duration = {
                let active_recoveries_guard = active_recoveries.read().await;
                if let Some(recovery) = active_recoveries_guard.get(&recovery_id_clone) {
                    (Utc::now().timestamp() - recovery.start_time.timestamp()) as u64
                } else {
                    0
                }
            };
            
            if let Err(_) = recovery_tx.send(
                if success {
                    RecoveryEvent::RecoveryCompleted { recovery_id: recovery_id_clone.clone(), success, duration_seconds: duration }
                } else {
                    RecoveryEvent::RecoveryFailed { recovery_id: recovery_id_clone.clone(), error: "Step execution failed".to_string(), completed_steps: current_step as u32 }
                }
            ).await {
                warn!("🔄 Error emitiendo evento de recuperación");
            }
        });
        
        Ok(())
    }
    
    /// Simula la ejecución de un paso de recuperación
    async fn execute_step(strategy: &RecoveryStrategyType, step: &str) -> bool {
        // Simular tiempo de ejecución
        let delay = match strategy {
            RecoveryStrategyType::RestartActor => 2000,
            RecoveryStrategyType::ScaleUp => 5000,
            RecoveryStrategyType::Failover => 3000,
            _ => 1000,
        };
        
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        
        // Simular 90% de éxito
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < 0.9
    }
    
    /// Cancela una recuperación activa
    pub async fn cancel_recovery(&self, recovery_id: &str) -> Result<(), ActorError> {
        let mut active_recoveries = self.active_recoveries.write().await;
        
        if let Some(recovery) = active_recoveries.get_mut(recovery_id) {
            recovery.status = RecoveryStatus::Cancelled;
            recovery.last_update = Utc::now();
            
            info!("🔄 Recuperación cancelada: {}", recovery_id);
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Recuperación no encontrada: {}", recovery_id),
            })
        }
    }
    
    /// Obtiene el estado de una recuperación activa
    pub async fn get_recovery_status(&self, recovery_id: &str) -> Option<ActiveRecovery> {
        let active_recoveries = self.active_recoveries.read().await;
        active_recoveries.get(recovery_id).cloned()
    }
    
    /// Obtiene todas las recuperaciones activas
    pub async fn get_active_recoveries(&self) -> HashMap<String, ActiveRecovery> {
        self.active_recoveries.read().await.clone()
    }
    
    /// Obtiene el historial de recuperaciones
    pub async fn get_recovery_history(&self) -> Vec<RecoveryRecord> {
        self.recovery_history.read().await.clone()
    }
    
    /// Emite un evento de recuperación
    async fn emit_event(&self, event: RecoveryEvent) {
        if let Err(_) = self.recovery_tx.send(event).await {
            warn!("🔄 Error emitiendo evento de recuperación");
        }
    }
    
    /// Limpia recuperaciones antiguas del historial
    pub async fn cleanup_old_recoveries(&self, older_than_hours: u64) {
        let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
        
        {
            let mut history = self.recovery_history.write().await;
            let original_len = history.len();
            history.retain(|record| record.recorded_at > cutoff);
            
            let removed = original_len - history.len();
            if removed > 0 {
                info!("🔄 Limpieza completada: {} recuperaciones antiguas removidas", removed);
            }
        }
        
        // Limpiar recuperaciones activas que han estado mucho tiempo
        {
            let mut active_recoveries = self.active_recoveries.write().await;
            let original_len = active_recoveries.len();
            active_recoveries.retain(|_, recovery| {
                Utc::now() - recovery.start_time < Duration::hours(1) // Máximo 1 hora activa
            });
            
            let removed = original_len - active_recoveries.len();
            if removed > 0 {
                info!("🔄 {} recuperaciones activas expiradas removidas", removed);
            }
        }
    }
    
    /// Obtiene estadísticas del sistema de recuperación
    pub async fn get_recovery_stats(&self) -> serde_json::Value {
        let active_recoveries = self.active_recoveries.read().await;
        let history = self.recovery_history.read().await;
        
        let total_recoveries = history.len();
        let successful_recoveries = history.iter().filter(|r| r.successful).count();
        let active_count = active_recoveries.len();
        
        let success_rate = if total_recoveries > 0 {
            successful_recoveries as f64 / total_recoveries as f64 * 100.0
        } else {
            0.0
        };
        
        // Calcular tiempo promedio de recuperación
        let avg_recovery_time = if total_recoveries > 0 {
            let total_time: u64 = history.iter().map(|r| r.duration_seconds).sum();
            total_time as f64 / total_recoveries as f64
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_recoveries": total_recoveries,
            "successful_recoveries": successful_recoveries,
            "active_recoveries": active_count,
            "success_rate_percentage": success_rate,
            "average_recovery_time_seconds": avg_recovery_time,
            "recovery_strategies_available": self.strategies.read().await.len()
        })
    }
}