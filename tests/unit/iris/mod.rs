// tests/unit/iris/mod.rs
// Tests unitarios para Iris - Service Mesh y Comunicación Inter-servicio

use olympus::actors::iris::{Iris, IrisConfig, ServiceRegistry, LoadBalancer};
use olympus::actors::iris::types::{ServiceInstance, RoutingRule, HealthStatus};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_iris_config() {
        let config = IrisConfig::default();
        assert!(config.service_discovery_enabled);
        assert!(config.load_balancing_enabled);
        assert_eq!(config.health_check_interval_secs, 30);
    }
}

#[cfg(test)]
mod service_registry_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_service() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service = ServiceInstance::new()
            .with_name("auth-service")
            .with_host("10.0.0.1")
            .with_port(8080)
            .with_version("v1.2.0");
        
        let result = iris.register_service(service).await;
        assert!(result.is_ok());
        assert!(iris.service_exists("auth-service").await);
    }
    
    #[tokio::test]
    async fn test_discover_service() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service = ServiceInstance::new()
            .with_name("payment-service")
            .with_host("10.0.0.2")
            .with_port(8081);
        
        iris.register_service(service).await.unwrap();
        
        let discovered = iris.discover_service("payment-service").await;
        assert!(discovered.is_some());
        assert_eq!(discovered.unwrap().host, "10.0.0.2");
    }
    
    #[tokio::test]
    async fn test_deregister_service() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service = ServiceInstance::new()
            .with_name("temp-service")
            .with_host("10.0.0.3")
            .with_port(8082);
        
        iris.register_service(service).await.unwrap();
        assert!(iris.service_exists("temp-service").await);
        
        iris.deregister_service("temp-service").await.unwrap();
        assert!(!iris.service_exists("temp-service").await);
    }
    
    #[tokio::test]
    async fn test_multiple_service_instances() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        // Registrar múltiples instancias del mismo servicio
        for i in 0..3 {
            let service = ServiceInstance::new()
                .with_name("api-gateway")
                .with_host(&format!("10.0.0.{}", i + 10))
                .with_port(8080);
            
            iris.register_service(service).await.unwrap();
        }
        
        let instances = iris.get_service_instances("api-gateway").await;
        assert_eq!(instances.len(), 3);
    }
}

#[cfg(test)]
mod load_balancing_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_round_robin_load_balancing() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        // Registrar 3 instancias
        for i in 0..3 {
            let service = ServiceInstance::new()
                .with_name("backend")
                .with_host(&format!("10.0.0.{}", i))
                .with_port(8080);
            
            iris.register_service(service).await.unwrap();
        }
        
        // Obtener instancias en round-robin
        let mut selected = vec![];
        for _ in 0..6 {
            let instance = iris.get_next_instance("backend", LoadBalancingStrategy::RoundRobin).await;
            selected.push(instance.host.clone());
        }
        
        // Debe distribuir equitativamente
        assert_eq!(selected[0], selected[3]); // Primera repetición
        assert_eq!(selected[1], selected[4]); // Segunda repetición
    }
    
    #[tokio::test]
    async fn test_least_connections_balancing() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service1 = ServiceInstance::new()
            .with_name("web-server")
            .with_host("10.0.0.1")
            .with_port(80)
            .with_active_connections(10);
        
        let service2 = ServiceInstance::new()
            .with_name("web-server")
            .with_host("10.0.0.2")
            .with_port(80)
            .with_active_connections(2);
        
        iris.register_service(service1).await.unwrap();
        iris.register_service(service2).await.unwrap();
        
        let selected = iris.get_next_instance(
            "web-server",
            LoadBalancingStrategy::LeastConnections
        ).await;
        
        // Debe seleccionar el que tiene menos conexiones
        assert_eq!(selected.host, "10.0.0.2");
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_route_by_path() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let rule = RoutingRule::new()
            .with_path_pattern("/api/v1/users/*")
            .with_target_service("user-service");
        
        iris.add_routing_rule(rule).await.unwrap();
        
        let target = iris.route_request("/api/v1/users/123").await;
        assert_eq!(target, Some("user-service".to_string()));
    }
    
    #[tokio::test]
    async fn test_route_by_header() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let rule = RoutingRule::new()
            .with_header_condition("X-Version", "v2")
            .with_target_service("api-v2");
        
        iris.add_routing_rule(rule).await.unwrap();
        
        let headers = vec![("X-Version".to_string(), "v2".to_string())];
        let target = iris.route_request_with_headers("/api/test", &headers).await;
        
        assert_eq!(target, Some("api-v2".to_string()));
    }
}

#[cfg(test)]
mod health_check_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_health_check() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service = ServiceInstance::new()
            .with_name("healthy-service")
            .with_host("10.0.0.1")
            .with_port(8080)
            .with_health_endpoint("/health");
        
        iris.register_service(service).await.unwrap();
        
        let health = iris.check_service_health("healthy-service").await;
        
        assert!(health.status == HealthStatus::Healthy || 
                health.status == HealthStatus::Unhealthy);
    }
    
    #[tokio::test]
    async fn test_unhealthy_service_removal() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let service = ServiceInstance::new()
            .with_name("unhealthy-service")
            .with_host("10.0.0.99")
            .with_port(8080);
        
        iris.register_service(service).await.unwrap();
        
        // Simular múltiples health check fallidos
        for _ in 0..3 {
            iris.record_health_check_failure("unhealthy-service").await;
        }
        
        // El servicio debe marcarse como no saludable
        let health = iris.get_service_health("unhealthy-service").await;
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_iris_creation() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        assert_eq!(iris.name(), GodName::Iris);
        assert_eq!(iris.domain(), DivineDomain::Communication);
    }
    
    #[tokio::test]
    async fn test_iris_health_check() {
        let iris = Iris::new().await.expect("Failed to create Iris");
        
        let health = iris.health_check().await;
        assert!(health.is_healthy());
    }
}
