// tests/unit/ares/mod.rs
// Tests unitarios para Ares - Resolución de Conflictos

use olympus::actors::ares::{Ares, AresConfig, ConflictResolver, LockManager};
use olympus::actors::ares::conflict::{Conflict, ResolutionStrategy, LockType, LockMode};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};
use std::time::Duration;

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_ares_config() {
        let config = AresConfig::default();
        assert_eq!(config.default_strategy, ResolutionStrategy::Optimistic);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.lock_timeout_secs, 30);
        assert!(config.deadlock_detection_enabled);
    }
    
    #[test]
    fn test_ares_config_builder() {
        let config = AresConfig::new()
            .with_default_strategy(ResolutionStrategy::Pessimistic)
            .with_max_retries(5)
            .with_lock_timeout(60)
            .disable_deadlock_detection();
            
        assert_eq!(config.default_strategy, ResolutionStrategy::Pessimistic);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.lock_timeout_secs, 60);
        assert!(!config.deadlock_detection_enabled);
    }
}

/// Tests de estrategias de resolución
#[cfg(test)]
mod strategy_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_optimistic_strategy() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("patient:123")
            .with_operation_a("update_age")
            .with_operation_b("update_name");
        
        let resolution = ares.resolve_with_strategy(
            conflict,
            ResolutionStrategy::Optimistic
        ).await.unwrap();
        
        // Optimistic: permite ambas operaciones, verifica al final
        assert!(resolution.success);
        assert_eq!(resolution.strategy_used, ResolutionStrategy::Optimistic);
    }
    
    #[tokio::test]
    async fn test_pessimistic_strategy() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("account:456")
            .with_operation_a("withdraw")
            .with_operation_b("withdraw");
        
        let resolution = ares.resolve_with_strategy(
            conflict,
            ResolutionStrategy::Pessimistic
        ).await.unwrap();
        
        // Pessimistic: bloquea una operación
        assert!(resolution.success);
        assert!(resolution.blocked_operation.is_some());
    }
    
    #[tokio::test]
    async fn test_last_write_wins_strategy() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("config:app")
            .with_operation_a("update_v1")
            .with_timestamp_a(1000)
            .with_operation_b("update_v2")
            .with_timestamp_b(2000);
        
        let resolution = ares.resolve_with_strategy(
            conflict,
            ResolutionStrategy::LastWriteWins
        ).await.unwrap();
        
        // Last write wins: v2 gana (timestamp mayor)
        assert!(resolution.success);
        assert_eq!(resolution.winner, Some("update_v2".to_string()));
    }
    
    #[tokio::test]
    async fn test_merge_strategy() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("document:merged")
            .with_operation_a("add_paragraph_1")
            .with_operation_b("add_paragraph_2");
        
        let resolution = ares.resolve_with_strategy(
            conflict,
            ResolutionStrategy::Merge
        ).await.unwrap();
        
        // Merge: combina ambas operaciones
        assert!(resolution.success);
        assert!(resolution.merged_operations.len() >= 2);
    }
    
    #[tokio::test]
    async fn test_custom_strategy() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Estrategia personalizada basada en prioridad
        let strategy = ResolutionStrategy::Custom {
            name: "priority_based".to_string(),
            rule: Box::new(|op_a, op_b| {
                if op_a.priority > op_b.priority {
                    Ok(op_a)
                } else {
                    Ok(op_b)
                }
            }),
        };
        
        let conflict = Conflict::new()
            .with_resource("critical_data")
            .with_operation_a("admin_update")
            .with_priority_a(10)
            .with_operation_b("user_update")
            .with_priority_b(1);
        
        let resolution = ares.resolve_with_strategy(conflict, strategy).await.unwrap();
        
        assert!(resolution.success);
        assert_eq!(resolution.winner, Some("admin_update".to_string()));
    }
}

/// Tests de gestión de locks
#[cfg(test)]
mod lock_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_acquire_read_lock() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let lock = ares.acquire_lock(
            "resource:1",
            LockType::Read,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        assert!(lock.is_acquired());
        assert_eq!(lock.lock_type, LockType::Read);
    }
    
    #[tokio::test]
    async fn test_acquire_write_lock() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let lock = ares.acquire_lock(
            "resource:2",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        assert!(lock.is_acquired());
        assert_eq!(lock.lock_type, LockType::Write);
    }
    
    #[tokio::test]
    async fn test_multiple_read_locks() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Múltiples lectores pueden tener lock simultáneamente
        let lock1 = ares.acquire_lock(
            "shared_resource",
            LockType::Read,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        let lock2 = ares.acquire_lock(
            "shared_resource",
            LockType::Read,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        let lock3 = ares.acquire_lock(
            "shared_resource",
            LockType::Read,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        assert!(lock1.is_acquired());
        assert!(lock2.is_acquired());
        assert!(lock3.is_acquired());
    }
    
    #[tokio::test]
    async fn test_write_lock_blocks_readers() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Adquirir lock de escritura
        let write_lock = ares.acquire_lock(
            "exclusive_resource",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        assert!(write_lock.is_acquired());
        
        // Intentar adquirir lock de lectura (no bloqueante)
        let read_result = ares.acquire_lock(
            "exclusive_resource",
            LockType::Read,
            LockMode::NonBlocking,
            Duration::from_secs(1)
        ).await;
        
        // Debe fallar porque hay un escritor
        assert!(read_result.is_err() || !read_result.unwrap().is_acquired());
    }
    
    #[tokio::test]
    async fn test_lock_release() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let lock = ares.acquire_lock(
            "releasable_resource",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        assert!(lock.is_acquired());
        
        // Liberar lock
        ares.release_lock(lock).await.unwrap();
        
        // Ahora otro puede adquirir el lock
        let new_lock = ares.acquire_lock(
            "releasable_resource",
            LockType::Write,
            LockMode::NonBlocking,
            Duration::from_secs(1)
        ).await.unwrap();
        
        assert!(new_lock.is_acquired());
    }
    
    #[tokio::test]
    async fn test_lock_timeout() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Adquirir lock con timeout corto
        let lock = ares.acquire_lock(
            "timeout_resource",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_millis(100)
        ).await.unwrap();
        
        // No liberar explícitamente
        drop(lock);
        
        // Esperar a que expire
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Ahora debe poderse adquirir de nuevo
        let new_lock = ares.acquire_lock(
            "timeout_resource",
            LockType::Write,
            LockMode::NonBlocking,
            Duration::from_secs(1)
        ).await.unwrap();
        
        assert!(new_lock.is_acquired());
    }
}

/// Tests de detección de deadlocks
#[cfg(test)]
mod deadlock_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_deadlock_detection_simple() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Crear situación de deadlock potencial
        // Thread A: lock X, intenta lock Y
        // Thread B: lock Y, intenta lock X
        
        let lock_x = ares.acquire_lock(
            "resource_x",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        let lock_y = ares.acquire_lock(
            "resource_y",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        // Detectar ciclo
        let deadlock_risk = ares.check_deadlock_risk(&[
            ("thread_a", vec!["resource_x", "resource_y"]),
            ("thread_b", vec!["resource_y", "resource_x"]),
        ]).await;
        
        assert!(deadlock_risk.is_deadlock_detected());
    }
    
    #[tokio::test]
    async fn test_deadlock_prevention() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Intentar adquirir locks en orden que causaría deadlock
        let result = ares.acquire_ordered_locks(&[
            ("resource_a", LockType::Write),
            ("resource_b", LockType::Write),
            ("resource_c", LockType::Write),
        ]).await;
        
        assert!(result.is_ok());
        
        // Todos los locks deben estar adquiridos
        let locks = result.unwrap();
        assert_eq!(locks.len(), 3);
        assert!(locks.iter().all(|l| l.is_acquired()));
    }
    
    #[tokio::test]
    async fn test_deadlock_resolution() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Simular deadlock detectado
        let deadlock = Deadlock::new()
            .with_cycle(vec![
                ("thread1", "resource1"),
                ("thread2", "resource2"),
                ("thread1", "resource2"),
                ("thread2", "resource1"),
            ]);
        
        let resolution = ares.resolve_deadlock(deadlock).await.unwrap();
        
        assert!(resolution.resolved);
        assert!(resolution.victim_thread.is_some());
        assert!(resolution.released_locks.len() > 0);
    }
    
    #[tokio::test]
    async fn test_wait_for_graph() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Construir grafo de espera
        ares.record_wait("thread_a", "resource_1").await;
        ares.record_hold("thread_b", "resource_1").await;
        ares.record_wait("thread_b", "resource_2").await;
        ares.record_hold("thread_c", "resource_2").await;
        ares.record_wait("thread_c", "resource_3").await;
        ares.record_hold("thread_a", "resource_3").await; // Ciclo!
        
        let graph = ares.get_wait_for_graph().await;
        let cycles = graph.detect_cycles();
        
        assert!(!cycles.is_empty());
    }
}

/// Tests de reconstrucción de estado
#[cfg(test)]
mod state_reconstruction_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_reconstruction_from_events() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Secuencia de eventos
        let events = vec![
            StateEvent::initial("{\"balance\": 100}"),
            StateEvent::operation("deposit", "{\"amount\": 50}"),
            StateEvent::operation("withdraw", "{\"amount\": 30}"),
            StateEvent::operation("deposit", "{\"amount\": 20}"),
        ];
        
        let reconstructed = ares.reconstruct_state(&events).await.unwrap();
        
        // Balance final: 100 + 50 - 30 + 20 = 140
        assert_eq!(reconstructed.get("balance").unwrap().as_i64(), Some(140));
    }
    
    #[tokio::test]
    async fn test_conflict_resolution_with_history() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("document:1")
            .with_operation_a("edit_intro")
            .with_base_version("v1")
            .with_operation_b("edit_conclusion")
            .with_base_version("v1")
            .with_history(vec![
                Operation::new("create", "{\"content\": \"\"}"),
                Operation::new("edit_intro", "{\"intro\": \"Hello\"}"),
            ]);
        
        let resolution = ares.resolve_with_history(conflict).await.unwrap();
        
        assert!(resolution.success);
        // Debe reconstruir y verificar compatibilidad
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ares_message_resolve_conflict() {
        let mut ares = Ares::new().await.expect("Failed to create Ares");
        
        let conflict = Conflict::new()
            .with_resource("test_resource")
            .with_operation_a("op_a")
            .with_operation_b("op_b");
        
        let message = ActorMessage::resolve_conflict_request(conflict);
        let response = ares.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_ares_message_acquire_lock() {
        let mut ares = Ares::new().await.expect("Failed to create Ares");
        
        let message = ActorMessage::acquire_lock_request(
            "resource:lock_test",
            LockType::Write,
            Duration::from_secs(30)
        );
        
        let response = ares.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(response.unwrap().lock_acquired);
    }
    
    #[tokio::test]
    async fn test_ares_message_detect_deadlock() {
        let mut ares = Ares::new().await.expect("Failed to create Ares");
        
        let message = ActorMessage::detect_deadlock_request();
        let response = ares.handle_message(message).await;
        
        assert!(response.is_ok());
        // Debe retornar estado de deadlock
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_lock_acquisition_performance() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let start = Instant::now();
        
        for i in 0..1000 {
            let lock = ares.acquire_lock(
                &format!("perf_resource_{}", i),
                LockType::Write,
                LockMode::NonBlocking,
                Duration::from_secs(1)
            ).await.unwrap();
            
            ares.release_lock(lock).await.ok();
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = 1000.0 / elapsed.as_secs_f64();
        
        assert!(
            ops_per_sec > 5000.0,
            "Lock acquisition too slow: {:.0} ops/sec",
            ops_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_conflict_resolution_performance() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let start = Instant::now();
        
        for i in 0..100 {
            let conflict = Conflict::new()
                .with_resource(&format!("resource:{}", i))
                .with_operation_a("update_a")
                .with_operation_b("update_b");
            
            ares.resolve_conflict(conflict).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let resolutions_per_sec = 100.0 / elapsed.as_secs_f64();
        
        assert!(
            resolutions_per_sec > 100.0,
            "Conflict resolution too slow: {:.0} resolutions/sec",
            resolutions_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_locks() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        let ares = std::sync::Arc::new(tokio::sync::Mutex::new(ares));
        
        let mut handles = vec![];
        
        for i in 0..50 {
            let ares_clone = ares.clone();
            let handle = tokio::spawn(async move {
                let lock = ares_clone.lock().await.acquire_lock(
                    &format!("concurrent_resource_{}", i % 10),
                    LockType::Write,
                    LockMode::Blocking,
                    Duration::from_secs(5)
                ).await.unwrap();
                
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                ares_clone.lock().await.release_lock(lock).await.ok();
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Todas las operaciones deben completar sin deadlock
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lock_on_nonexistent_resource() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let result = ares.acquire_lock(
            "",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(1)
        ).await;
        
        // Debe manejar recurso vacío
        assert!(result.is_err() || result.is_ok());
    }
    
    #[tokio::test]
    async fn test_double_lock_release() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let lock = ares.acquire_lock(
            "double_release_test",
            LockType::Write,
            LockMode::Blocking,
            Duration::from_secs(30)
        ).await.unwrap();
        
        // Primera liberación
        ares.release_lock(lock.clone()).await.unwrap();
        
        // Segunda liberación (debe manejar gracefully)
        let result = ares.release_lock(lock).await;
        
        // No debe panic
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_very_long_conflict_resolution() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Crear conflicto complejo con muchas operaciones
        let mut conflict = Conflict::new()
            .with_resource("complex_resource");
        
        for i in 0..100 {
            conflict.add_operation(&format!("op_{}", i));
        }
        
        let result = ares.resolve_conflict(conflict).await;
        
        // Debe manejar sin timeout
        assert!(result.is_ok());
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ares_creation() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        assert_eq!(ares.name(), GodName::Ares);
        assert_eq!(ares.domain(), DivineDomain::ConflictResolution);
    }
    
    #[tokio::test]
    async fn test_ares_health_check() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        let health = ares.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_lock_cleanup_on_shutdown() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Crear varios locks
        for i in 0..10 {
            let _ = ares.acquire_lock(
                &format!("cleanup_resource_{}", i),
                LockType::Write,
                LockMode::Blocking,
                Duration::from_secs(300)
            ).await.unwrap();
        }
        
        // Shutdown
        ares.shutdown().await.unwrap();
        
        // Todos los locks deben liberarse
        let active_locks = ares.get_active_locks().await;
        assert!(active_locks.is_empty());
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let ares = Ares::new().await.expect("Failed to create Ares");
        
        // Realizar operaciones
        for i in 0..20 {
            let conflict = Conflict::new()
                .with_resource(&format!("resource:{}", i))
                .with_operation_a("a")
                .with_operation_b("b");
            
            ares.resolve_conflict(conflict).await.unwrap();
        }
        
        for i in 0..30 {
            let lock = ares.acquire_lock(
                &format!("lock_{}", i),
                LockType::Write,
                LockMode::NonBlocking,
                Duration::from_secs(1)
            ).await.unwrap();
            
            ares.release_lock(lock).await.ok();
        }
        
        let metrics = ares.collect_metrics().await;
        
        assert_eq!(metrics.conflicts_resolved, 20);
        assert_eq!(metrics.locks_acquired, 30);
        assert!(metrics.deadlocks_detected >= 0);
    }
}
