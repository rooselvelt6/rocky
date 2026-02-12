// tests/unit/moirai/mod.rs
// Tests unitarios para Moirai - Gestión de Lifecycle y Orquestación

use olympus::actors::moirai::{Moirai, MoiraiConfig, ContainerOrchestrator, ThreadManager};
use olympus::actors::moirai::types::{Container, ContainerConfig, ResourceLimits, LifecycleHook};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_moirai_config() {
        let config = MoiraiConfig::default();
        assert_eq!(config.max_containers, 100);
        assert_eq!(config.max_threads, 1000);
        assert!(config.auto_restart_enabled);
    }
}

#[cfg(test)]
mod container_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_container() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let config = ContainerConfig::new()
            .with_image("olympus/service:latest")
            .with_name("test-container")
            .with_port_mapping(8080, 80);
        
        let result = moirai.create_container(config).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().id.is_empty());
    }
    
    #[tokio::test]
    async fn test_start_container() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let config = ContainerConfig::new()
            .with_image("test-image")
            .with_name("start-test");
        
        let container = moirai.create_container(config).await.unwrap();
        
        let result = moirai.start_container(&container.id).await;
        assert!(result.is_ok());
        
        let status = moirai.get_container_status(&container.id).await;
        assert!(status.is_running());
    }
    
    #[tokio::test]
    async fn test_stop_container() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let config = ContainerConfig::new()
            .with_image("test-image")
            .with_name("stop-test");
        
        let container = moirai.create_container(config).await.unwrap();
        moirai.start_container(&container.id).await.unwrap();
        
        moirai.stop_container(&container.id).await.unwrap();
        
        let status = moirai.get_container_status(&container.id).await;
        assert!(!status.is_running());
    }
    
    #[tokio::test]
    async fn test_container_resource_limits() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let config = ContainerConfig::new()
            .with_image("limited-image")
            .with_name("limited-container")
            .with_resource_limits(ResourceLimits {
                cpu_cores: 2.0,
                memory_mb: 512,
                disk_mb: 1024,
            });
        
        let container = moirai.create_container(config).await.unwrap();
        
        let limits = moirai.get_container_resource_limits(&container.id).await.unwrap();
        assert_eq!(limits.cpu_cores, 2.0);
        assert_eq!(limits.memory_mb, 512);
    }
    
    #[tokio::test]
    async fn test_container_lifecycle_hooks() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let hook_executed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let hook_clone = hook_executed.clone();
        
        let config = ContainerConfig::new()
            .with_image("hook-image")
            .with_name("hook-test")
            .with_lifecycle_hook(LifecycleHook::PreStart, move || {
                let flag = hook_clone.clone();
                async move {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                }
            });
        
        let container = moirai.create_container(config).await.unwrap();
        moirai.start_container(&container.id).await.unwrap();
        
        assert!(hook_executed.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[cfg(test)]
mod thread_management_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spawn_thread() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let result = moirai.spawn_thread(|| async {
            // Thread work
            Ok(())
        }).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_thread_pool() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        // Crear pool de 5 threads
        let pool = moirai.create_thread_pool(5).await.unwrap();
        
        // Ejecutar 10 tareas
        let mut handles = vec![];
        for i in 0..10 {
            let handle = pool.execute(move || async move {
                // Task i
                i
            }).await;
            handles.push(handle);
        }
        
        // Todas deben completar
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result >= 0 && result < 10);
        }
    }
    
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        // Crear contenedores
        for i in 0..5 {
            let config = ContainerConfig::new()
                .with_image("test")
                .with_name(&format!("container-{}", i));
            
            let container = moirai.create_container(config).await.unwrap();
            moirai.start_container(&container.id).await.unwrap();
        }
        
        // Graceful shutdown
        moirai.shutdown().await.unwrap();
        
        // Todos los contenedores deben estar detenidos
        let running = moirai.get_running_containers().await;
        assert!(running.is_empty());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_moirai_creation() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        assert_eq!(moirai.name(), GodName::Moirai);
        assert_eq!(moirai.domain(), DivineDomain::Lifecycle);
    }
    
    #[tokio::test]
    async fn test_moirai_health_check() {
        let moirai = Moirai::new().await.expect("Failed to create Moirai");
        
        let health = moirai.health_check().await;
        assert!(health.is_healthy());
    }
}
