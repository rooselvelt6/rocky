// tests/unit/demeter/mod.rs
// Tests unitarios para Demeter - Gestión de Recursos

use olympus::actors::demeter::{Demeter, DemeterConfig, ResourceMonitor, QuotaManager};
use olympus::actors::demeter::types::{ResourceUsage, ResourceType, Quota};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_demeter_config() {
        let config = DemeterConfig::default();
        assert!(config.monitoring_enabled);
        assert!(config.auto_scaling_enabled);
        assert_eq!(config.cpu_threshold, 80.0);
        assert_eq!(config.memory_threshold, 85.0);
    }
}

#[cfg(test)]
mod resource_monitoring_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_monitor_cpu_usage() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let usage = demeter.get_resource_usage(ResourceType::CPU).await;
        
        assert!(usage.percentage >= 0.0 && usage.percentage <= 100.0);
    }
    
    #[tokio::test]
    async fn test_monitor_memory_usage() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let usage = demeter.get_resource_usage(ResourceType::Memory).await;
        
        assert!(usage.used_bytes >= 0);
        assert!(usage.total_bytes > 0);
    }
    
    #[tokio::test]
    async fn test_monitor_disk_usage() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let usage = demeter.get_resource_usage(ResourceType::Disk).await;
        
        assert!(usage.used_bytes >= 0);
        assert!(usage.available_bytes >= 0);
    }
    
    #[tokio::test]
    async fn test_resource_threshold_alert() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        // Configurar umbral bajo para testing
        demeter.set_threshold(ResourceType::CPU, 5.0).await.unwrap();
        
        // Simular uso alto
        demeter.simulate_high_cpu(10.0).await;
        
        // Verificar alerta
        let alerts = demeter.get_active_alerts().await;
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.resource_type == ResourceType::CPU));
    }
}

#[cfg(test)]
mod quota_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_set_quota() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let quota = Quota::new()
            .with_cpu_cores(2.0)
            .with_memory_mb(1024)
            .with_disk_gb(10);
        
        let result = demeter.set_quota("service:api", quota).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_enforce_quota() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let quota = Quota::new()
            .with_memory_mb(100);
        
        demeter.set_quota("limited-service", quota).await.unwrap();
        
        // Intentar exceder quota
        let result = demeter.allocate_memory("limited-service", 200).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_quota_violation_detection() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let quota = Quota::new()
            .with_cpu_cores(1.0);
        
        demeter.set_quota("monitored-service", quota).await.unwrap();
        
        // Simular uso que viola quota
        demeter.simulate_quota_violation("monitored-service").await;
        
        let violations = demeter.get_quota_violations().await;
        assert!(!violations.is_empty());
    }
}

#[cfg(test)]
mod auto_scaling_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scale_up_trigger() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        // Configurar auto-scaling
        demeter.configure_auto_scaling(
            "web-service",
            ScalingPolicy::new()
                .scale_up_when(ScalingMetric::CPU, Comparison::GreaterThan, 70.0)
                .scale_up_by(2)
        ).await.unwrap();
        
        // Simular alta carga
        demeter.simulate_high_load("web-service").await;
        
        // Verificar que se disparó scale up
        let scaling_events = demeter.get_scaling_events("web-service").await;
        assert!(scaling_events.iter().any(|e| e.action == ScalingAction::ScaleUp));
    }
    
    #[tokio::test]
    async fn test_scale_down_trigger() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        demeter.configure_auto_scaling(
            "background-service",
            ScalingPolicy::new()
                .scale_down_when(ScalingMetric::CPU, Comparison::LessThan, 20.0)
                .scale_down_by(1)
        ).await.unwrap();
        
        // Simular baja carga
        demeter.simulate_low_load("background-service").await;
        
        let scaling_events = demeter.get_scaling_events("background-service").await;
        assert!(scaling_events.iter().any(|e| e.action == ScalingAction::ScaleDown));
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_demeter_creation() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        assert_eq!(demeter.name(), GodName::Demeter);
        assert_eq!(demeter.domain(), DivineDomain::Resources);
    }
    
    #[tokio::test]
    async fn test_demeter_health_check() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let health = demeter.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let demeter = Demeter::new().await.expect("Failed to create Demeter");
        
        let metrics = demeter.collect_metrics().await;
        
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0);
        assert!(metrics.active_services >= 0);
    }
}
