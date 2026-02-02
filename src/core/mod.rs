/// Core system module for Olympus v11
/// This is the entry point for the OTP-inspired system

pub mod otp;
pub mod messaging;

// Re-export commonly used items
pub use otp::*;
pub use messaging::*;

/// System initialization and management
pub mod system {
    use super::*;
    use std::sync::Arc;
    use tracing::{info, error, debug};
    
    /// Global system manager
    pub struct OlympusSystem {
        registry: Arc<GlobalRegistry>,
        node_id: String,
        started_at: std::time::Instant,
    }
    
    impl OlympusSystem {
        /// Create a new system instance
        pub fn new(node_id: impl Into<String>) -> Self {
            let node_id = node_id.into();
            let registry = Arc::new(GlobalRegistry::new(node_id.clone()));
            
            Self {
                registry,
                node_id,
                started_at: std::time::Instant::now(),
            }
        }
        
        /// Start the system
        pub async fn start(&mut self) -> OtpResult<()> {
            info!("Starting Olympus v11 system on node: {}", self.node_id);
            
            // Initialize core services
            self.init_core_services().await?;
            
            info!("Olympus v11 system started successfully");
            Ok(())
        }
        
        /// Stop the system gracefully
        pub async fn stop(&self) -> OtpResult<()> {
            info!("Stopping Olympus v11 system on node: {}", self.node_id);
            
            // Unregister all actors
            let actors = self.registry.list_actors().await?;
            for actor_name in actors {
                if let Err(e) = self.registry.unregister_actor(&actor_name).await {
                    error!("Failed to unregister actor '{}': {:?}", actor_name, e);
                }
            }
            
            info!("Olympus v11 system stopped");
            Ok(())
        }
        
        /// Get the global registry
        pub fn registry(&self) -> Arc<GlobalRegistry> {
            self.registry.clone()
        }
        
        /// Get system information
        pub fn system_info(&self) -> SystemInfo {
            SystemInfo {
                node_id: self.node_id.clone(),
                uptime: self.started_at.elapsed(),
                status: SystemStatus::Running,
            }
        }
        
        async fn init_core_services(&self) -> OtpResult<()> {
            // Initialize core system services here
            debug!("Initializing core services");
            Ok(())
        }
    }
    
    /// System information
    #[derive(Debug, Clone)]
    pub struct SystemInfo {
        pub node_id: String,
        pub uptime: std::time::Duration,
        pub status: SystemStatus,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum SystemStatus {
        Starting,
        Running,
        Stopping,
        Stopped,
        Error,
    }
}

/// Utility functions for system management
pub mod utils {
    use super::*;
    
    /// Get system node ID
    pub fn get_node_id() -> String {
        std::env::var("OLYMPUS_NODE_ID")
            .unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4()))
    }
    
    /// Check if we're in development mode
    pub fn is_development() -> bool {
        std::env::var("OLYMPUS_ENV")
            .unwrap_or_else(|_| "development".to_string())
            == "development"
    }
    
    /// Get default configuration values
    pub fn default_config() -> SystemConfig {
        SystemConfig {
            node_id: get_node_id(),
            mailbox_size: 1000,
            restart_max_restarts: 3,
            restart_time_window_secs: 5,
            backpressure_threshold: 0.8,
            enable_distributed: false,
            enable_monitoring: true,
        }
    }
    
    /// System configuration
    #[derive(Debug, Clone)]
    pub struct SystemConfig {
        pub node_id: String,
        pub mailbox_size: usize,
        pub restart_max_restarts: u32,
        pub restart_time_window_secs: u64,
        pub backpressure_threshold: f32,
        pub enable_distributed: bool,
        pub enable_monitoring: bool,
    }
    
    impl Default for SystemConfig {
        fn default() -> Self {
            default_config()
        }
    }
}