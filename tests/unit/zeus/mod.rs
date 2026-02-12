// tests/unit/zeus/mod.rs
// Tests unitarios para Zeus - Supervisión y Gobernanza

use olympus::actors::zeus::{Zeus, ZeusConfig, SupervisionStrategy};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_zeus_config() {
        let config = ZeusConfig::default();
        assert_eq!(config.supervision_strategy, SupervisionStrategy::OneForOne);
        assert_eq!(config.max_restarts, 5);
        assert_eq!(config.restart_window_secs, 60);
        assert!(config.metrics_enabled);
    }
    
    #[test]
    fn test_zeus_config_builder() {
        let config = ZeusConfig::new()
            .with_strategy(SupervisionStrategy::OneForAll)
            .with_max_restarts(10)
            .with_restart_window(120)
            .with_metrics(false);
            
        assert_eq!(config.supervision_strategy, SupervisionStrategy::OneForAll);
        assert_eq!(config.max_restarts, 10);
        assert_eq!(config.restart_window_secs, 120);
        assert!(!config.metrics_enabled);
    }
}

/// Tests de creación y ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zeus_creation() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        assert_eq!(zeus.name(), GodName::Zeus);
        assert_eq!(zeus.domain(), DivineDomain::Governance);
    }
    
    #[tokio::test]
    async fn test_zeus_with_config() {
        let config = ZeusConfig::new()
            .with_max_restarts(3);
            
        let zeus = Zeus::with_config(config).await.expect("Failed to create Zeus with config");
        
        assert_eq!(zeus.name(), GodName::Zeus);
    }
    
    #[tokio::test]
    async fn test_zeus_health_check() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let health = zeus.health_check().await;
        
        assert!(health.is_healthy());
        assert!(health.uptime_secs > 0);
    }
}

/// Tests de supervisión
#[cfg(test)]
mod supervision_tests {
    use super::*;
    use olympus::actors::zeus::{Supervisor, RestartPolicy};
    
    #[tokio::test]
    async fn test_supervisor_children_list() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let children = zeus.children().await;
        
        // Zeus debe tener al menos algunos actores hijos
        assert!(!children.is_empty(), "Zeus should have children actors");
    }
    
    #[tokio::test]
    async fn test_restart_policy_one_for_one() {
        let policy = RestartPolicy::OneForOne {
            max_restarts: 3,
            within_secs: 60,
        };
        
        // Simular un fallo
        let should_restart = policy.should_restart(2, 30);
        assert!(should_restart);
        
        // Después de 3 fallos, no debe reiniciar
        let should_restart = policy.should_restart(3, 30);
        assert!(!should_restart);
    }
    
    #[tokio::test]
    async fn test_restart_policy_too_many_failures() {
        let policy = RestartPolicy::OneForOne {
            max_restarts: 5,
            within_secs: 60,
        };
        
        // 5 fallos en 30 segundos: debe reiniciar
        let should_restart = policy.should_restart(5, 30);
        assert!(should_restart);
        
        // 6 fallos: no debe reiniciar
        let should_restart = policy.should_restart(6, 30);
        assert!(!should_restart);
    }
}

/// Tests de métricas
#[cfg(test)]
mod metrics_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zeus_metrics_collection() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let metrics = zeus.collect_metrics().await;
        
        assert!(metrics.actor_count > 0);
        assert!(metrics.uptime_secs >= 0);
        assert!(metrics.memory_usage_bytes >= 0);
    }
    
    #[tokio::test]
    async fn test_system_metrics() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let sys_metrics = zeus.system_metrics().await;
        
        assert!(sys_metrics.cpu_usage_percent >= 0.0 && sys_metrics.cpu_usage_percent <= 100.0);
        assert!(sys_metrics.memory_usage_bytes >= 0);
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zeus_message_ping() {
        let mut zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let message = ActorMessage::ping();
        let response = zeus.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_zeus_message_status() {
        let mut zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let message = ActorMessage::status_request();
        let response = zeus.handle_message(message).await;
        
        assert!(response.is_ok());
        let payload = response.unwrap();
        assert!(payload.contains_status_info());
    }
    
    #[tokio::test]
    async fn test_zeus_message_restart_actor() {
        let mut zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let message = ActorMessage::restart_actor(GodName::Hermes);
        let response = zeus.handle_message(message).await;
        
        // Puede fallar si Hermes no está corriendo, pero no debe panic
        // En tests unitarios, simplemente verificamos que no haya panic
        let _ = response;
    }
}

/// Tests de estrategias de supervisión
#[cfg(test)]
mod strategy_tests {
    use super::*;
    
    #[test]
    fn test_supervision_strategy_variants() {
        let strategies = vec![
            SupervisionStrategy::OneForOne,
            SupervisionStrategy::OneForAll,
            SupervisionStrategy::RestForOne,
        ];
        
        for strategy in strategies {
            let desc = format!("{:?}", strategy);
            assert!(!desc.is_empty());
        }
    }
    
    #[test]
    fn test_strategy_restart_behavior() {
        // OneForOne: solo reinicia el actor fallido
        assert!(SupervisionStrategy::OneForOne.restarts_only_failed());
        
        // OneForAll: reinicia todos
        assert!(!SupervisionStrategy::OneForAll.restarts_only_failed());
        
        // RestForOne: reinicia el fallido y los que iniciaron después
        assert!(!SupervisionStrategy::RestForOne.restarts_only_failed());
    }
}

/// Tests de rendimiento
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_zeus_creation_performance() {
        let start = Instant::now();
        
        let _zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "Zeus creation took too long: {:?}",
            elapsed
        );
    }
    
    #[tokio::test]
    async fn test_health_check_performance() {
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let start = Instant::now();
        
        for _ in 0..100 {
            let _ = zeus.health_check().await;
        }
        
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "100 health checks took too long: {:?}",
            elapsed
        );
    }
    
    #[tokio::test]
    async fn test_message_throughput() {
        let mut zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        let start = Instant::now();
        let message_count = 1000;
        
        for _ in 0..message_count {
            let message = ActorMessage::ping();
            let _ = zeus.handle_message(message).await;
        }
        
        let elapsed = start.elapsed();
        let msgs_per_sec = message_count as f64 / elapsed.as_secs_f64();
        
        assert!(
            msgs_per_sec > 1000.0,
            "Message throughput too low: {:.0} msgs/sec",
            msgs_per_sec
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zeus_concurrent_access() {
        use tokio::task;
        
        let zeus = Zeus::new().await.expect("Failed to create Zeus");
        let zeus = std::sync::Arc::new(tokio::sync::Mutex::new(zeus));
        
        let mut handles = vec![];
        
        for _ in 0..10 {
            let zeus_clone = zeus.clone();
            let handle = task::spawn(async move {
                let _ = zeus_clone.lock().await.health_check().await;
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.expect("Task failed");
        }
    }
    
    #[tokio::test]
    async fn test_zeus_recovery_from_error() {
        let mut zeus = Zeus::new().await.expect("Failed to create Zeus");
        
        // Enviar mensaje inválido
        let invalid_message = ActorMessage::invalid();
        let result = zeus.handle_message(invalid_message).await;
        
        // Debe manejar el error gracefully
        assert!(result.is_err());
        
        // Zeus debe seguir funcionando
        let health = zeus.health_check().await;
        assert!(health.is_healthy());
    }
}
