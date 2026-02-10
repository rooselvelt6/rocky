// src/actors/chronos/mod.rs
// OLYMPUS v15 - Chronos: Dios del Tiempo y Scheduling
// Responsabilidad: Programación y ejecución de tareas con sintaxis cron

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

// Submódulos
pub mod scheduler;
pub mod tasks;
pub mod time_events;
pub mod statistics;

pub use scheduler::{TaskScheduler, CronExpression};
pub use tasks::{ScheduledTask, TaskDefinition, TaskStatus, TaskType, TaskResult};
pub use time_events::TimeEvent;
pub use statistics::SchedulerMetrics;

/// Chronos - Dios del Scheduling
/// Gestiona la programación y ejecución de tareas en el sistema
#[derive(Debug)]
pub struct Chronos {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Motor de scheduling principal
    scheduler: Arc<RwLock<TaskScheduler>>,
    /// Registro de tareas activas
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// Métricas del scheduler
    metrics: Arc<RwLock<SchedulerMetrics>>,
    /// Flag para controlar el loop de scheduling
    running: Arc<RwLock<bool>>,
}

impl Chronos {
    pub async fn new() -> Self {
        info!("⏰ Chronos: Inicializando scheduler de tareas...");
        
        Self {
            name: GodName::Chronos,
            state: ActorState::new(GodName::Chronos),
            config: ActorConfig::default(),
            scheduler: Arc::new(RwLock::new(TaskScheduler::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Programa una nueva tarea
    pub async fn schedule_task(&self, definition: TaskDefinition) -> Result<String, ActorError> {
        let task_id = format!("task_{}_{}", Utc::now().timestamp_millis(), std::process::id());
        let name = definition.name.clone(); // Clonar antes de mover
        
        let task = ScheduledTask::new(
            task_id.clone(),
            definition.name,
            definition.task_type,
            definition.cron_expression,
            definition.payload,
            definition.creator.unwrap_or(GodName::Zeus),
        );
        
        // Agregar al scheduler
        let mut scheduler = self.scheduler.write().await;
        scheduler.schedule_task(&task)?;
        
        // Registrar la tarea
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);
        
        // Actualizar métricas
        let mut metrics = self.metrics.write().await;
        metrics.tasks_scheduled += 1;
        
        info!("⏰ Chronos: Tarea '{}' programada con ID {}", name, task_id);
        
        Ok(task_id)
    }

    /// Cancela una tarea programada
    pub async fn cancel_task(&self, task_id: &str) -> Result<(), ActorError> {
        let mut scheduler = self.scheduler.write().await;
        scheduler.cancel_task(task_id)?;
        
        let mut tasks = self.tasks.write().await;
        if let Some(mut task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            task.updated_at = Utc::now();
            info!("⏰ Chronos: Tarea {} cancelada", task_id);
            Ok(())
        } else {
            Err(ActorError::NotFound { 
                god: GodName::Chronos
            })
        }
    }

    /// Pausa una tarea recurrente
    pub async fn pause_task(&self, task_id: &str) -> Result<(), ActorError> {
        let mut tasks = self.tasks.write().await;
        if let Some(mut task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Running {
                return Err(ActorError::InvalidCommand {
                    god: GodName::Chronos,
                    reason: "No se puede pausar una tarea en ejecución".to_string(),
                });
            }
            task.status = TaskStatus::Paused;
            task.updated_at = Utc::now();
            info!("⏰ Chronos: Tarea {} pausada", task_id);
            Ok(())
        } else {
            Err(ActorError::NotFound { 
                god: GodName::Chronos
            })
        }
    }

    /// Reanuda una tarea pausada
    pub async fn resume_task(&self, task_id: &str) -> Result<(), ActorError> {
        let mut tasks = self.tasks.write().await;
        if let Some(mut task) = tasks.get_mut(task_id) {
            if task.status != TaskStatus::Paused {
                return Err(ActorError::InvalidCommand {
                    god: GodName::Chronos,
                    reason: "La tarea no está pausada".to_string(),
                });
            }
            task.status = TaskStatus::Pending;
            task.updated_at = Utc::now();
            
            // Recalcular próxima ejecución
            let mut scheduler = self.scheduler.write().await;
            scheduler.reschedule_task(&task)?;
            
            info!("⏰ Chronos: Tarea {} reanudada", task_id);
            Ok(())
        } else {
            Err(ActorError::NotFound { 
                god: GodName::Chronos
            })
        }
    }

    /// Ejecuta una tarea inmediatamente (one-shot)
    pub async fn execute_now(&self, task_id: &str) -> Result<TaskResult, ActorError> {
        let tasks = self.tasks.read().await;
        if let Some(task) = tasks.get(task_id) {
            let task = task.clone();
            drop(tasks); // Liberar lock
            
            info!("⏰ Chronos: Ejecutando tarea {} inmediatamente", task_id);
            
            // Actualizar estado
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.status = TaskStatus::Running;
                t.execution_count += 1;
                t.last_execution = Some(Utc::now());
            }
            drop(tasks);
            
            // Ejecutar la tarea
            let result = self.execute_task(&task).await;
            
            // Actualizar resultado
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.status = if result.success { TaskStatus::Completed } else { TaskStatus::Failed };
                t.last_result = Some(result.clone());
                t.updated_at = Utc::now();
            }
            
            // Actualizar métricas
            let mut metrics = self.metrics.write().await;
            metrics.tasks_executed += 1;
            if result.success {
                metrics.tasks_successful += 1;
            } else {
                metrics.tasks_failed += 1;
            }
            
            Ok(result)
        } else {
            Err(ActorError::NotFound { 
                god: GodName::Chronos
            })
        }
    }

    /// Obtiene el estado de una tarea
    pub async fn get_task_status(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// Lista todas las tareas
    pub async fn list_tasks(&self, status_filter: Option<TaskStatus>) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|t| status_filter.as_ref().map_or(true, |s| t.status == *s))
            .cloned()
            .collect()
    }

    /// Obtiene las próximas ejecuciones
    pub async fn get_next_executions(&self, limit: usize) -> Vec<(String, DateTime<Utc>)> {
        let scheduler = self.scheduler.read().await;
        scheduler.get_next_executions(limit)
    }

    /// Loop principal del scheduler
    async fn start_scheduler_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        
        loop {
            interval.tick().await;
            
            // Verificar si debemos detenernos
            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }
            
            // Revisar tareas pendientes
            let now = Utc::now();
            let due_tasks = {
                let scheduler = self.scheduler.read().await;
                scheduler.get_due_tasks(now)
            };
            
            // Ejecutar tareas vencidas
            for task_id in due_tasks {
                if let Err(e) = self.execute_task_by_id(&task_id).await {
                    error!("⏰ Chronos: Error ejecutando tarea {}: {:?}", task_id, e);
                }
            }
        }
    }

    /// Ejecuta una tarea por ID
    async fn execute_task_by_id(&self, task_id: &str) -> Result<(), ActorError> {
        let tasks = self.tasks.read().await;
        if let Some(task) = tasks.get(task_id) {
            let task = task.clone();
            drop(tasks);
            
            // Ignorar si está pausada o cancelada
            if task.status == TaskStatus::Paused || task.status == TaskStatus::Cancelled {
                return Ok(());
            }
            
            debug!("⏰ Chronos: Ejecutando tarea programada {}", task_id);
            
            // Actualizar estado
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.status = TaskStatus::Running;
                t.execution_count += 1;
                t.last_execution = Some(Utc::now());
            }
            drop(tasks);
            
            // Ejecutar
            let result = self.execute_task(&task).await;
            
            // Actualizar estado post-ejecución
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(task_id) {
                match task.task_type {
                    TaskType::OneShot => {
                        t.status = if result.success { TaskStatus::Completed } else { TaskStatus::Failed };
                    }
                    TaskType::Recurring | TaskType::Interval(_) => {
                        t.status = TaskStatus::Pending;
                        // Recalcular próxima ejecución
                        let mut scheduler = self.scheduler.write().await;
                        if let Err(e) = scheduler.reschedule_task(&t) {
                            warn!("⏰ Chronos: Error reprogramando tarea {}: {:?}", task_id, e);
                            t.status = TaskStatus::Failed;
                        }
                    }
                }
                t.last_result = Some(result.clone());
                t.updated_at = Utc::now();
            }
            
            // Actualizar métricas
            let mut metrics = self.metrics.write().await;
            metrics.tasks_executed += 1;
            if result.success {
                metrics.tasks_successful += 1;
            } else {
                metrics.tasks_failed += 1;
            }
            
            Ok(())
        } else {
            Err(ActorError::NotFound { 
                god: GodName::Chronos
            })
        }
    }

    /// Ejecuta el payload de una tarea
    async fn execute_task(&self, task: &ScheduledTask) -> TaskResult {
        info!("⏰ Chronos: Ejecutando payload de tarea '{}' ({})", task.name, task.id);
        
        // Emitir evento de inicio de ejecución
        let event = TimeEvent::task_started(&task.id, &task.name, task.creator);
        self.emit_event(event).await;
        
        // Simular ejecución del payload
        // En una implementación real, esto ejecutaría el payload específico
        let start_time = std::time::Instant::now();
        
        // Simular trabajo
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let duration = start_time.elapsed();
        
        // Por defecto asumimos éxito (en implementación real habría lógica específica)
        let success = true;
        let message = format!("Tarea ejecutada exitosamente en {:?}", duration);
        
        let result = TaskResult {
            success,
            message: message.clone(),
            executed_at: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            output: task.payload.clone(),
        };
        
        // Emitir evento de fin de ejecución
        let event = TimeEvent::task_completed(&task.id, &task.name, success, duration.as_millis() as u64);
        self.emit_event(event).await;
        
        result
    }

    /// Emite un evento temporal a Apollo
    async fn emit_event(&self, event: TimeEvent) {
        debug!("⏰ Chronos: Emitiendo evento temporal {:?}", event);
        // En una implementación completa, esto enviaría un mensaje a Apollo
        // Por ahora solo registramos localmente
    }
}

#[async_trait]
impl OlympianActor for Chronos {
    fn name(&self) -> GodName { 
        GodName::Chronos 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::Scheduling 
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = Utc::now();

        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            _ => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }

    async fn persistent_state(&self) -> serde_json::Value {
        let tasks = self.tasks.read().await;
        let metrics = self.metrics.read().await;
        
        serde_json::json!({
            "name": "Chronos",
            "messages": self.state.message_count,
            "tasks_count": tasks.len(),
            "tasks_scheduled": metrics.tasks_scheduled,
            "tasks_executed": metrics.tasks_executed,
            "status": self.state.status,
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        // TODO: Restaurar tareas desde persistencia
        Ok(())
    }

    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name.clone(),
            status: self.state.status.clone(),
            last_seen: Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }

    async fn health_check(&self) -> HealthStatus {
        let metrics = self.metrics.read().await;
        let tasks = self.tasks.read().await;
        
        let running_tasks = tasks.values().filter(|t| t.status == TaskStatus::Running).count() as u64;
        
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: metrics.tasks_failed,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("⏰ Chronos: Scheduler v15 iniciado");
        info!("⏰ Chronos: Listo para programar tareas");
        
        // Iniciar el loop de scheduling
        let mut running = self.running.write().await;
        *running = true;
        
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("⏰ Chronos: Deteniendo scheduler...");
        
        let mut running = self.running.write().await;
        *running = false;
        
        info!("⏰ Chronos: Scheduler detenido");
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados para manejo de comandos y queries
impl Chronos {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("schedule_task") => {
                        let definition: TaskDefinition = serde_json::from_value(
                            data.get("definition").cloned().unwrap_or_default()
                        ).map_err(|e| ActorError::InvalidCommand {
                            god: GodName::Chronos,
                            reason: format!("Definición de tarea inválida: {}", e),
                        })?;
                        
                        let task_id = self.schedule_task(definition).await?;
                        Ok(ResponsePayload::Success { 
                            message: format!("Tarea programada: {}", task_id) 
                        })
                    }
                    Some("cancel_task") => {
                        let task_id = data.get("task_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Chronos,
                                reason: "task_id requerido".to_string(),
                            })?;
                        
                        self.cancel_task(task_id).await?;
                        Ok(ResponsePayload::Success { 
                            message: format!("Tarea {} cancelada", task_id) 
                        })
                    }
                    Some("pause_task") => {
                        let task_id = data.get("task_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Chronos,
                                reason: "task_id requerido".to_string(),
                            })?;
                        
                        self.pause_task(task_id).await?;
                        Ok(ResponsePayload::Success { 
                            message: format!("Tarea {} pausada", task_id) 
                        })
                    }
                    Some("resume_task") => {
                        let task_id = data.get("task_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Chronos,
                                reason: "task_id requerido".to_string(),
                            })?;
                        
                        self.resume_task(task_id).await?;
                        Ok(ResponsePayload::Success { 
                            message: format!("Tarea {} reanudada", task_id) 
                        })
                    }
                    Some("execute_now") => {
                        let task_id = data.get("task_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Chronos,
                                reason: "task_id requerido".to_string(),
                            })?;
                        
                        let result = self.execute_now(task_id).await?;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "success": result.success,
                                "message": result.message,
                                "duration_ms": result.duration_ms,
                            })
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Chronos, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Chronos, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let metrics = self.metrics.read().await;
                let tasks = self.tasks.read().await;
                
                let pending = tasks.values().filter(|t| t.status == TaskStatus::Pending).count() as u64;
                let running = tasks.values().filter(|t| t.status == TaskStatus::Running).count() as u64;
                let completed = tasks.values().filter(|t| t.status == TaskStatus::Completed).count() as u64;
                let failed = tasks.values().filter(|t| t.status == TaskStatus::Failed).count() as u64;
                
                Ok(ResponsePayload::Stats { 
                    data: serde_json::json!({
                        "tasks_scheduled": metrics.tasks_scheduled,
                        "tasks_executed": metrics.tasks_executed,
                        "tasks_successful": metrics.tasks_successful,
                        "tasks_failed": metrics.tasks_failed,
                        "tasks_pending": pending,
                        "tasks_running": running,
                        "tasks_completed": completed,
                        "tasks_failed_total": failed,
                    })
                })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                match query_type {
                    "list_tasks" => {
                        let status = data.get("status")
                            .and_then(|v| serde_json::from_value::<TaskStatus>(v.clone()).ok());
                        let tasks = self.list_tasks(status).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&tasks).unwrap_or_default()
                        })
                    }
                    "get_task" => {
                        let task_id = data.get("task_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Chronos,
                                reason: "task_id requerido".to_string(),
                            })?;
                        
                        if let Some(task) = self.get_task_status(task_id).await {
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&task).unwrap_or_default()
                            })
                        } else {
                            Err(ActorError::NotFound { 
                                god: GodName::Chronos
                            })
                        }
                    }
                    "get_next_executions" => {
                        let limit = data.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                        let executions = self.get_next_executions(limit).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "executions": executions,
                            })
                        })
                    }
                    "scheduler_health" => {
                        let running = *self.running.read().await;
                        let scheduler = self.scheduler.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "running": running,
                                "scheduled_count": scheduler.task_count(),
                                "timestamp": Utc::now(),
                            })
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Chronos, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Chronos, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::message::MessagePriority;
    use serde_json::json;

    #[tokio::test]
    async fn test_chronos_initialization() -> Result<(), ActorError> {
        let mut chronos = Chronos::new().await;
        chronos.initialize().await?;
        
        assert_eq!(chronos.name(), GodName::Chronos);
        assert_eq!(chronos.domain(), DivineDomain::Scheduling);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chronos_schedule_task() -> Result<(), ActorError> {
        let chronos = Chronos::new().await;
        
        let definition = TaskDefinition {
            name: "Test Task".to_string(),
            task_type: TaskType::OneShot,
            cron_expression: Some("0 * * * * *".to_string()), // Cada minuto
            payload: json!({"action": "test"}),
            creator: Some(GodName::Zeus),
        };
        
        let task_id = chronos.schedule_task(definition).await?;
        assert!(!task_id.is_empty());
        
        // Verificar que existe
        let task = chronos.get_task_status(&task_id).await;
        assert!(task.is_some());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chronos_cancel_task() -> Result<(), ActorError> {
        let chronos = Chronos::new().await;
        
        let definition = TaskDefinition {
            name: "Task to Cancel".to_string(),
            task_type: TaskType::OneShot,
            cron_expression: Some("0 0 * * * *".to_string()),
            payload: json!({}),
            creator: None,
        };
        
        let task_id = chronos.schedule_task(definition).await?;
        chronos.cancel_task(&task_id).await?;
        
        let task = chronos.get_task_status(&task_id).await;
        assert_eq!(task.unwrap().status, TaskStatus::Cancelled);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chronos_execute_task() -> Result<(), ActorError> {
        let chronos = Chronos::new().await;
        
        let definition = TaskDefinition {
            name: "Executable Task".to_string(),
            task_type: TaskType::OneShot,
            cron_expression: None,
            payload: json!({"test": true}),
            creator: Some(GodName::Athena),
        };
        
        let task_id = chronos.schedule_task(definition).await?;
        let result = chronos.execute_now(&task_id).await?;
        
        assert!(result.success);
        assert!(!result.message.is_empty());
        
        // Verificar que se completó
        let task = chronos.get_task_status(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        
        Ok(())
    }
}


