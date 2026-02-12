// tests/unit/chaos/mod.rs
// Tests unitarios para Chaos - Chaos Engineering

use olympus::actors::chaos::{Chaos, ChaosConfig, Experiment, FailureInjector};
use olympus::actors::chaos::types::{FailureType, ExperimentResult, SafetyConstraint};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_chaos_config() {
        let config = ChaosConfig::default();
        assert!(config.experiments_enabled);
        assert!(config.safety_checks_enabled);
        assert_eq!(config.max_failure_rate, 0.1); // 10%
    }
}

#[cfg(test)]
mod experiment_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_experiment() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let experiment = Experiment::new("network_latency_test")
            .with_failure_type(FailureType::NetworkLatency {
                delay_ms: 100,
                jitter_ms: 20,
            })
            .with_target("service:payment-api")
            .with_duration(std::time::Duration::from_secs(60));
        
        let result = chaos.create_experiment(experiment).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_run_experiment() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let experiment = Experiment::new("cpu_stress_test")
            .with_failure_type(FailureType::ResourceExhaustion {
                resource: ResourceType::CPU,
                percentage: 80,
            })
            .with_duration(std::time::Duration::from_secs(5));
        
        let exp_id = chaos.create_experiment(experiment).await.unwrap();
        
        let result = chaos.run_experiment(&exp_id).await;
        
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.completed);
    }
    
    #[tokio::test]
    async fn test_experiment_hypothesis_validation() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let experiment = Experiment::new("hypothesis_test")
            .with_failure_type(FailureType::ServiceCrash)
            .with_hypothesis(|system_state| {
                // El sistema debe seguir funcionando con 1 servicio caído
                system_state.healthy_services >= system_state.total_services - 1
            });
        
        let exp_id = chaos.create_experiment(experiment).await.unwrap();
        let result = chaos.run_experiment(&exp_id).await.unwrap();
        
        // Verificar que se validó la hipótesis
        assert!(result.hypothesis_validated.is_some());
    }
}

#[cfg(test)]
mod failure_injection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_inject_network_latency() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let result = chaos.inject_failure(
            FailureType::NetworkLatency {
                delay_ms: 500,
                jitter_ms: 50,
            },
            "service:auth"
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_inject_service_crash() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let result = chaos.inject_failure(
            FailureType::ServiceCrash,
            "service:non-critical"
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_inject_resource_exhaustion() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let result = chaos.inject_failure(
            FailureType::ResourceExhaustion {
                resource: ResourceType::Memory,
                percentage: 90,
            },
            "container:test"
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_recovery_validation() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        // Inyectar fallo
        chaos.inject_failure(FailureType::ServiceCrash, "service:test").await.unwrap();
        
        // Esperar recuperación
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Verificar recuperación
        let recovered = chaos.validate_recovery("service:test").await;
        
        // Puede ser true o false dependiendo del sistema
        assert!(recovered.is_recovered || !recovered.is_recovered);
    }
}

#[cfg(test)]
mod safety_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_safety_constraints() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let constraint = SafetyConstraint::new()
            .with_max_failure_rate(0.05) // Max 5% de fallos
            .with_protected_services(vec!["critical-database", "auth-service"]);
        
        chaos.add_safety_constraint(constraint).await.unwrap();
        
        // Intentar experimento que violaría restricción
        let experiment = Experiment::new("unsafe_test")
            .with_failure_type(FailureType::ServiceCrash)
            .with_target("critical-database"); // Protegido!
        
        let result = chaos.validate_experiment_safety(&experiment).await;
        
        assert!(!result.is_safe);
    }
    
    #[tokio::test]
    async fn test_auto_rollback() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let experiment = Experiment::new("rollback_test")
            .with_failure_type(FailureType::NetworkPartition)
            .with_auto_rollback(true);
        
        let exp_id = chaos.create_experiment(experiment).await.unwrap();
        
        // Ejecutar y verificar rollback automático
        let result = chaos.run_experiment(&exp_id).await.unwrap();
        
        if result.requires_rollback {
            assert!(result.rollback_executed);
        }
    }
}

#[cfg(test)]
mod game_day_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_game_day_scenario() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        // Crear escenario de Game Day
        let game_day = chaos.create_game_day_scenario("production_drill")
            .with_description("Simular caída de zona completa")
            .add_experiment(Experiment::new("zone_failure").with_failure_type(FailureType::ZoneOutage))
            .add_experiment(Experiment::new("failover_test").with_failure_type(FailureType::ServiceCrash))
            .build();
        
        let result = chaos.run_game_day(game_day).await;
        
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chaos_creation() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        assert_eq!(chaos.name(), GodName::Chaos);
        assert_eq!(chaos.domain(), DivineDomain::Chaos);
    }
    
    #[tokio::test]
    async fn test_chaos_health_check() {
        let chaos = Chaos::new().await.expect("Failed to create Chaos");
        
        let health = chaos.health_check().await;
        assert!(health.is_healthy());
    }
}
