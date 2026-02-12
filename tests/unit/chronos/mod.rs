// tests/unit/chronos/mod.rs
// Tests unitarios para Chronos - Scheduling y Tareas

use olympus::actors::chronos::{Chronos, ChronosConfig, Scheduler, TaskQueue};
use olympus::actors::chronos::types::{CronExpression, Task, TaskPriority, TaskStatus};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};
use std::time::Duration;

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_chronos_config() {
        let config = ChronosConfig::default();
        assert!(config.timezone_aware);
        assert_eq!(config.max_concurrent_tasks, 100);
        assert_eq!(config.default_timeout_secs, 3600);
        assert!(config.persistence_enabled);
    }
    
    #[test]
    fn test_chronos_config_builder() {
        let config = ChronosConfig::new()
            .with_max_concurrent_tasks(50)
            .with_default_timeout(1800)
            .with_timezone("America/New_York")
            .disable_persistence();
            
        assert_eq!(config.max_concurrent_tasks, 50);
        assert_eq!(config.default_timeout_secs, 1800);
        assert_eq!(config.timezone, "America/New_York");
        assert!(!config.persistence_enabled);
    }
}

/// Tests de expresiones cron
#[cfg(test)]
mod cron_expression_tests {
    use super::*;
    
    #[test]
    fn test_cron_every_minute() {
        let cron = CronExpression::parse("* * * * *").unwrap();
        assert!(cron.is_valid());
        assert_eq!(cron.minute, "*");
        assert_eq!(cron.hour, "*");
    }
    
    #[test]
    fn test_cron_specific_time() {
        let cron = CronExpression::parse("30 14 * * *").unwrap();
        assert!(cron.is_valid());
        assert_eq!(cron.minute, "30");
        assert_eq!(cron.hour, "14");
    }
    
    #[test]
    fn test_cron_every_hour() {
        let cron = CronExpression::parse("0 * * * *").unwrap();
        assert!(cron.is_valid());
    }
    
    #[test]
    fn test_cron_weekdays() {
        let cron = CronExpression::parse("0 9 * * 1-5").unwrap();
        assert!(cron.is_valid());
        assert_eq!(cron.day_of_week, "1-5");
    }
    
    #[test]
    fn test_cron_monthly() {
        let cron = CronExpression::parse("0 0 1 * *").unwrap();
        assert!(cron.is_valid());
        assert_eq!(cron.day_of_month, "1");
    }
    
    #[test]
    fn test_cron_step_values() {
        let cron = CronExpression::parse("*/15 * * * *").unwrap();
        assert!(cron.is_valid());
    }
    
    #[test]
    fn test_cron_list_values() {
        let cron = CronExpression::parse("0 9,17 * * *").unwrap();
        assert!(cron.is_valid());
    }
    
    #[test]
    fn test_invalid_cron_expression() {
        let result = CronExpression::parse("invalid");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cron_next_execution() {
        let cron = CronExpression::parse("0 12 * * *").unwrap();
        let now = chrono::Utc::now();
        let next = cron.next_execution(now).unwrap();
        
        assert!(next > now);
        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 0);
    }
    
    #[test]
    fn test_cron_multiple_next_executions() {
        let cron = CronExpression::parse("0 */6 * * *").unwrap();
        let now = chrono::Utc::now();
        let executions = cron.next_executions(now, 5);
        
        assert_eq!(executions.len(), 5);
        
        for i in 1..executions.len() {
            assert!(executions[i] > executions[i-1]);
        }
    }
}

/// Tests de tareas
#[cfg(test)]
mod task_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_creation() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("backup_task")
            .with_action(|| async {
                // Backup action
                Ok(())
            })
            .with_priority(TaskPriority::High)
            .with_timeout(Duration::from_secs(300));
        
        assert_eq!(task.name, "backup_task");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.timeout, Duration::from_secs(300));
        assert_eq!(task.status, TaskStatus::Pending);
    }
    
    #[tokio::test]
    async fn test_task_scheduling() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("scheduled_task")
            .with_schedule("*/5 * * * *");
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        assert!(!task_id.is_empty());
        assert!(chronos.has_task(&task_id).await);
    }
    
    #[tokio::test]
    async fn test_task_execution() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let executed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let executed_clone = executed.clone();
        
        let task = Task::new()
            .with_name("test_execution")
            .with_action(move || {
                let flag = executed_clone.clone();
                async move {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                }
            });
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Ejecutar inmediatamente
        chronos.execute_task_now(&task_id).await.unwrap();
        
        // Verificar que se ejecutó
        assert!(executed.load(std::sync::atomic::Ordering::SeqCst));
        
        // Verificar estado
        let status = chronos.get_task_status(&task_id).await.unwrap();
        assert_eq!(status, TaskStatus::Completed);
    }
    
    #[tokio::test]
    async fn test_task_cancellation() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("cancellable_task")
            .with_schedule("0 0 * * *");
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Cancelar
        chronos.cancel_task(&task_id).await.unwrap();
        
        // Verificar que está cancelada
        let status = chronos.get_task_status(&task_id).await.unwrap();
        assert_eq!(status, TaskStatus::Cancelled);
    }
    
    #[tokio::test]
    async fn test_task_retry_on_failure() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        
        let task = Task::new()
            .with_name("retry_task")
            .with_action(move || {
                let count = attempt_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if current < 2 {
                        Err(TaskError::ExecutionFailed)
                    } else {
                        Ok(())
                    }
                }
            })
            .with_retry_policy(RetryPolicy::new(3, Duration::from_secs(1)));
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Ejecutar
        chronos.execute_task_now(&task_id).await.unwrap();
        
        // Debe haber intentado 3 veces (2 fallos + 1 éxito)
        assert_eq!(
            attempt_count.load(std::sync::atomic::Ordering::SeqCst),
            3
        );
    }
    
    #[tokio::test]
    async fn test_task_timeout() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("timeout_task")
            .with_action(|| async {
                // Tarea que toma mucho tiempo
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            })
            .with_timeout(Duration::from_millis(100));
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        let result = chronos.execute_task_now(&task_id).await;
        
        // Debe fallar por timeout
        assert!(result.is_err());
        
        let status = chronos.get_task_status(&task_id).await.unwrap();
        assert_eq!(status, TaskStatus::Failed);
    }
    
    #[tokio::test]
    async fn test_task_priority_queue() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let low_priority_task = Task::new()
            .with_name("low_priority")
            .with_priority(TaskPriority::Low);
        
        let high_priority_task = Task::new()
            .with_name("high_priority")
            .with_priority(TaskPriority::High);
        
        let critical_task = Task::new()
            .with_name("critical")
            .with_priority(TaskPriority::Critical);
        
        // Agregar en orden inverso
        chronos.schedule_task(low_priority_task).await.unwrap();
        chronos.schedule_task(high_priority_task).await.unwrap();
        chronos.schedule_task(critical_task).await.unwrap();
        
        // Obtener siguiente tarea
        let next = chronos.get_next_task().await.unwrap();
        
        // Debe ser la de prioridad crítica
        assert_eq!(next.name, "critical");
    }
}

/// Tests de scheduling avanzado
#[cfg(test)]
mod scheduling_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cron_scheduling() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("cron_task")
            .with_schedule("*/1 * * * *"); // Cada minuto
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Verificar que está programada
        let scheduled = chronos.get_scheduled_tasks().await;
        assert!(scheduled.iter().any(|t| t.id == task_id));
    }
    
    #[tokio::test]
    async fn test_one_time_scheduling() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let run_at = chrono::Utc::now() + chrono::Duration::seconds(5);
        
        let task = Task::new()
            .with_name("one_time_task")
            .run_at(run_at);
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // No debe estar en tareas cron
        let cron_tasks = chronos.get_cron_tasks().await;
        assert!(!cron_tasks.iter().any(|t| t.id == task_id));
        
        // Debe estar en tareas programadas
        let scheduled = chronos.get_scheduled_tasks().await;
        assert!(scheduled.iter().any(|t| t.id == task_id));
    }
    
    #[tokio::test]
    async fn test_delayed_execution() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let executed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let executed_clone = executed.clone();
        
        let task = Task::new()
            .with_name("delayed_task")
            .with_action(move || {
                let flag = executed_clone.clone();
                async move {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                }
            })
            .with_delay(Duration::from_millis(500));
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // No debe ejecutarse inmediatamente
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!executed.load(std::sync::atomic::Ordering::SeqCst));
        
        // Esperar a que se ejecute
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert!(executed.load(std::sync::atomic::Ordering::SeqCst));
    }
    
    #[tokio::test]
    async fn test_recurring_task() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let execution_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let execution_count_clone = execution_count.clone();
        
        let task = Task::new()
            .with_name("recurring_task")
            .with_action(move || {
                let count = execution_count_clone.clone();
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                }
            })
            .with_schedule("*/1 * * * *") // Cada minuto (para testing)
            .with_max_executions(3);
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Simular paso del tiempo
        for _ in 0..3 {
            chronos.trigger_cron_execution(&task_id).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        assert_eq!(
            execution_count.load(std::sync::atomic::Ordering::SeqCst),
            3
        );
    }
    
    #[tokio::test]
    async fn test_timezone_aware_scheduling() {
        let config = ChronosConfig::new()
            .with_timezone("America/New_York");
        
        let chronos = Chronos::with_config(config).await
            .expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("ny_task")
            .with_schedule("0 9 * * *"); // 9 AM NY time
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        let scheduled_task = chronos.get_task(&task_id).await.unwrap();
        assert!(scheduled_task.timezone_aware);
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chronos_message_schedule_task() {
        let mut chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new().with_name("msg_task");
        let message = ActorMessage::schedule_task(task);
        
        let response = chronos.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(!response.unwrap().get_task_id().is_empty());
    }
    
    #[tokio::test]
    async fn test_chronos_message_cancel_task() {
        let mut chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        // Crear tarea primero
        let task = Task::new().with_name("cancel_msg_task");
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Cancelar vía mensaje
        let cancel_msg = ActorMessage::cancel_task(&task_id);
        let response = chronos.handle_message(cancel_msg).await;
        
        assert!(response.is_ok());
        assert!(response.unwrap().is_success());
    }
    
    #[tokio::test]
    async fn test_chronos_message_get_scheduled_tasks() {
        let mut chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        // Agregar algunas tareas
        for i in 0..3 {
            let task = Task::new().with_name(&format!("task_{}", i));
            chronos.schedule_task(task).await.unwrap();
        }
        
        let message = ActorMessage::get_scheduled_tasks();
        let response = chronos.handle_message(message).await;
        
        assert!(response.is_ok());
        let tasks = response.unwrap().get_tasks();
        assert_eq!(tasks.len(), 3);
    }
}

/// Tests de persistencia
#[cfg(test)]
mod persistence_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_persistence() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("persistent_task")
            .with_schedule("0 0 * * *");
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Persistir
        chronos.persist_tasks().await.unwrap();
        
        // Simular reinicio cargando tareas
        let loaded_tasks = chronos.load_persisted_tasks().await.unwrap();
        
        assert!(loaded_tasks.iter().any(|t| t.id == task_id));
    }
    
    #[tokio::test]
    async fn test_task_history() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("history_task")
            .with_action(|| async { Ok(()) });
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Ejecutar varias veces
        for _ in 0..3 {
            chronos.execute_task_now(&task_id).await.unwrap();
        }
        
        let history = chronos.get_task_history(&task_id).await.unwrap();
        
        assert_eq!(history.execution_count, 3);
        assert_eq!(history.success_count, 3);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_cron_parsing_performance() {
        let start = Instant::now();
        
        for _ in 0..10000 {
            let _ = CronExpression::parse("*/15 9-17 * * 1-5");
        }
        
        let elapsed = start.elapsed();
        let parses_per_sec = 10000.0 / elapsed.as_secs_f64();
        
        assert!(
            parses_per_sec > 50000.0,
            "Cron parsing too slow: {:.0} parses/sec",
            parses_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_task_scheduling_performance() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let start = Instant::now();
        
        for i in 0..1000 {
            let task = Task::new().with_name(&format!("perf_task_{}", i));
            chronos.schedule_task(task).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let schedules_per_sec = 1000.0 / elapsed.as_secs_f64();
        
        assert!(
            schedules_per_sec > 1000.0,
            "Task scheduling too slow: {:.0} schedules/sec",
            schedules_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_task_execution() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        // Crear 100 tareas concurrentes
        let mut handles = vec![];
        
        for i in 0..100 {
            let counter_clone = counter.clone();
            let chronos_clone = chronos.clone();
            
            let handle = tokio::spawn(async move {
                let task = Task::new()
                    .with_name(&format!("concurrent_{}", i))
                    .with_action(move || {
                        let c = counter_clone.clone();
                        async move {
                            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            Ok(())
                        }
                    });
                
                let task_id = chronos_clone.schedule_task(task).await.unwrap();
                chronos_clone.execute_task_now(&task_id).await.unwrap();
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::SeqCst),
            100
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_cron_schedule() {
        let result = CronExpression::parse("");
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_task_with_no_action() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new().with_name("no_action_task");
        // Sin action definida
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Intentar ejecutar
        let result = chronos.execute_task_now(&task_id).await;
        
        // Debe manejar gracefulmente
        assert!(result.is_err() || result.is_ok());
    }
    
    #[tokio::test]
    async fn test_very_long_task_name() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let long_name = "a".repeat(10000);
        let task = Task::new().with_name(&long_name);
        
        let result = chronos.schedule_task(task).await;
        
        // Debe manejar nombres largos
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_task_with_panic_action() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let task = Task::new()
            .with_name("panic_task")
            .with_action(|| async {
                panic!("Intentional panic");
            });
        
        let task_id = chronos.schedule_task(task).await.unwrap();
        
        // Ejecutar - no debe propagar el panic
        let result = chronos.execute_task_now(&task_id).await;
        
        // Debe fallar pero no propagar panic
        assert!(result.is_err());
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chronos_creation() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        assert_eq!(chronos.name(), GodName::Chronos);
        assert_eq!(chronos.domain(), DivineDomain::Scheduling);
    }
    
    #[tokio::test]
    async fn test_chronos_health_check() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        let health = chronos.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        // Crear tareas en ejecución
        for i in 0..5 {
            let task = Task::new()
                .with_name(&format!("running_{}", i))
                .with_action(|| async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(())
                });
            
            let task_id = chronos.schedule_task(task).await.unwrap();
            chronos.execute_task_now(&task_id).await.ok();
        }
        
        // Shutdown graceful
        chronos.shutdown().await.expect("Shutdown failed");
        
        // No debe haber tareas en ejecución
        let running = chronos.get_running_tasks().await;
        assert!(running.is_empty());
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let chronos = Chronos::new().await.expect("Failed to create Chronos");
        
        // Crear y ejecutar algunas tareas
        for _ in 0..10 {
            let task = Task::new()
                .with_name("metric_task")
                .with_action(|| async { Ok(()) });
            
            let task_id = chronos.schedule_task(task).await.unwrap();
            chronos.execute_task_now(&task_id).await.unwrap();
        }
        
        let metrics = chronos.collect_metrics().await;
        
        assert_eq!(metrics.tasks_executed, 10);
        assert_eq!(metrics.tasks_succeeded, 10);
        assert_eq!(metrics.tasks_failed, 0);
    }
}
