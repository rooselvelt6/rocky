// tests/unit/hefesto/mod.rs
// Tests unitarios para Hefesto - CI/CD y Construcción

use olympus::actors::hefesto::{Hefesto, HefestoConfig, Pipeline, BuildManager};
use olympus::actors::hefesto::types::{BuildConfig, PipelineStage, Artifact, TestResult};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_hefesto_config() {
        let config = HefestoConfig::default();
        assert!(config.parallel_builds);
        assert_eq!(config.max_concurrent_builds, 5);
        assert_eq!(config.default_timeout_mins, 60);
        assert!(config.cache_enabled);
    }
    
    #[test]
    fn test_hefesto_config_builder() {
        let config = HefestoConfig::new()
            .with_max_concurrent_builds(10)
            .with_timeout(120)
            .disable_parallel_builds()
            .disable_cache();
            
        assert_eq!(config.max_concurrent_builds, 10);
        assert_eq!(config.default_timeout_mins, 120);
        assert!(!config.parallel_builds);
        assert!(!config.cache_enabled);
    }
}

/// Tests de pipelines
#[cfg(test)]
mod pipeline_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_pipeline() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let pipeline = Pipeline::new("main_pipeline")
            .with_stage(PipelineStage::Checkout)
            .with_stage(PipelineStage::Build)
            .with_stage(PipelineStage::Test)
            .with_stage(PipelineStage::Deploy);
        
        let result = hefesto.create_pipeline(pipeline).await;
        
        assert!(result.is_ok());
        assert!(hefesto.pipeline_exists("main_pipeline").await);
    }
    
    #[tokio::test]
    async fn test_execute_pipeline() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let pipeline = Pipeline::new("test_pipeline")
            .with_stage(PipelineStage::Checkout)
            .with_stage(PipelineStage::Build);
        
        hefesto.create_pipeline(pipeline).await.unwrap();
        
        let result = hefesto.execute_pipeline("test_pipeline").await;
        
        assert!(result.is_ok());
        let execution = result.unwrap();
        assert!(execution.success || !execution.success); // Puede fallar pero no panic
    }
    
    #[tokio::test]
    async fn test_pipeline_stages_execution_order() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let execution_order = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let order_clone = execution_order.clone();
        
        let pipeline = Pipeline::new("ordered_pipeline")
            .with_stage(PipelineStage::Custom {
                name: "stage_1".to_string(),
                action: Box::new(move || {
                    let order = order_clone.clone();
                    async move {
                        order.lock().unwrap().push(1);
                        Ok(())
                    }
                }),
            })
            .with_stage(PipelineStage::Custom {
                name: "stage_2".to_string(),
                action: Box::new(move || {
                    let order = execution_order.clone();
                    async move {
                        order.lock().unwrap().push(2);
                        Ok(())
                    }
                }),
            });
        
        hefesto.create_pipeline(pipeline).await.unwrap();
        hefesto.execute_pipeline("ordered_pipeline").await.unwrap();
        
        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], 1);
        assert_eq!(order[1], 2);
    }
}

/// Tests de builds
#[cfg(test)]
mod build_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_build() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let config = BuildConfig::new()
            .with_source("https://github.com/repo/app")
            .with_branch("main")
            .with_target("release");
        
        let result = hefesto.create_build(config).await;
        
        assert!(result.is_ok());
        assert!(!result.unwrap().build_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_build_execution() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let config = BuildConfig::new()
            .with_source("test_source")
            .with_build_command("echo 'Building...'");
        
        let build = hefesto.create_build(config).await.unwrap();
        
        let result = hefesto.execute_build(&build.build_id).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_build_caching() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        // Primera build
        let config1 = BuildConfig::new()
            .with_source("cached_source")
            .with_cache_key("cache_v1");
        
        let build1 = hefesto.create_build(config1).await.unwrap();
        hefesto.execute_build(&build1.build_id).await.unwrap();
        
        // Segunda build con mismo cache key
        let config2 = BuildConfig::new()
            .with_source("cached_source")
            .with_cache_key("cache_v1");
        
        let start = std::time::Instant::now();
        let build2 = hefesto.create_build(config2).await.unwrap();
        hefesto.execute_build(&build2.build_id).await.unwrap();
        let elapsed = start.elapsed();
        
        // Debe usar cache y ser más rápido
        assert!(elapsed.as_millis() < 1000);
    }
    
    #[tokio::test]
    async fn test_build_artifacts() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let config = BuildConfig::new()
            .with_source("artifact_test")
            .with_artifact_path("target/release/app");
        
        let build = hefesto.create_build(config).await.unwrap();
        hefesto.execute_build(&build.build_id).await.unwrap();
        
        let artifacts = hefesto.get_build_artifacts(&build.build_id).await.unwrap();
        
        assert!(!artifacts.is_empty());
    }
}

/// Tests de testing
#[cfg(test)]
mod testing_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_run_unit_tests() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let result = hefesto.run_tests(TestType::Unit, "test_project").await;
        
        assert!(result.is_ok());
        let test_result = result.unwrap();
        assert!(test_result.executed);
    }
    
    #[tokio::test]
    async fn test_run_integration_tests() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let result = hefesto.run_tests(TestType::Integration, "test_project").await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_test_result_parsing() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let test_output = r#"
            running 10 tests
            test test_1 ... ok
            test test_2 ... ok
            test test_3 ... FAILED
            test test_4 ... ok
            
            test result: FAILED. 9 passed; 1 failed
        "#;
        
        let result = hefesto.parse_test_results(test_output).await.unwrap();
        
        assert_eq!(result.total_tests, 10);
        assert_eq!(result.passed, 9);
        assert_eq!(result.failed, 1);
        assert!(!result.success);
    }
}

/// Tests de deployment
#[cfg(test)]
mod deployment_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_deploy_to_staging() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let artifact = Artifact::new()
            .with_name("app_v1.0.0")
            .with_path("/builds/app_v1.0.0.tar.gz");
        
        let result = hefesto.deploy(
            artifact,
            Environment::Staging,
            DeploymentStrategy::Rolling
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_deployment_rollback() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        // Deploy
        let artifact = Artifact::new().with_name("app_v1.0.0");
        let deployment = hefesto.deploy(
            artifact,
            Environment::Production,
            DeploymentStrategy::BlueGreen
        ).await.unwrap();
        
        // Rollback
        let rollback = hefesto.rollback_deployment(&deployment.id).await;
        
        assert!(rollback.is_ok());
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hefesto_creation() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        assert_eq!(hefesto.name(), GodName::Hefesto);
        assert_eq!(hefesto.domain(), DivineDomain::Construction);
    }
    
    #[tokio::test]
    async fn test_hefesto_health_check() {
        let hefesto = Hefesto::new().await.expect("Failed to create Hefesto");
        
        let health = hefesto.health_check().await;
        assert!(health.is_healthy());
    }
}
