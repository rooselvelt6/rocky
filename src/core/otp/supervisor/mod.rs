/// Supervisor implementation with restart strategies
/// Provides fault tolerance and process supervision

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, oneshot};
use tracing::{debug, error, info, warn};

/// Runtime implementation of a Supervisor
pub struct SupervisorRuntime<S: Supervisor> {
    supervisor: S,
    actor_id: ActorId,
    children: Arc<RwLock<HashMap<ActorId, ChildProcess>>>,
    restart_counts: Arc<RwLock<HashMap<ActorId, RestartCounter>>>,
    restart_strategy: RestartStrategy,
    config: Option<S::Config>,
}

/// Information about a supervised child process
#[derive(Debug)]
pub struct ChildProcess {
    pub id: ActorId,
    pub spec: ChildSpec,
    pub restart_count: u32,
    pub last_restart: std::time::Instant,
    pub is_running: bool,
    pub stop_handle: Option<oneshot::Sender<TerminationReason>>,
    pub start_time: std::time::Instant,
}

/// Counter for tracking restarts within time window
#[derive(Debug, Clone)]
pub struct RestartCounter {
    pub count: u32,
    pub first_restart: std::time::Instant,
}

/// Address for communicating with the supervisor
pub struct SupervisorAddr<S: Supervisor> {
    actor_id: ActorId,
    children: Arc<RwLock<HashMap<ActorId, ChildProcess>>>,
    restart_counts: Arc<RwLock<HashMap<ActorId, RestartCounter>>>,
}

impl<S: Supervisor> Clone for SupervisorAddr<S> {
    fn clone(&self) -> Self {
        Self {
            actor_id: self.actor_id.clone(),
            children: self.children.clone(),
            restart_counts: self.restart_counts.clone(),
        }
    }
}

impl<S: Supervisor> SupervisorRuntime<S> {
    pub fn new(actor_id: ActorId, supervisor: S, restart_strategy: RestartStrategy) -> Self {
        Self {
            supervisor,
            actor_id,
            children: Arc::new(RwLock::new(HashMap::new())),
            restart_counts: Arc::new(RwLock::new(HashMap::new())),
            restart_strategy,
            config: None,
        }
    }
    
    pub async fn start(&mut self) -> OtpResult<()> {
        info!("Starting Supervisor: {}", self.actor_id.name);
        
        // Initialize supervisor
        match self.supervisor.init().await {
            Ok(config) => {
                self.config = Some(config);
                debug!("Supervisor {} initialized successfully", self.actor_id.name);
                
                // Start monitoring for child failures
                self.start_failure_monitoring().await?;
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize Supervisor {}: {:?}", self.actor_id.name, e);
                Err(OtpError::SupervisorError { 
                    reason: format!("Initialization failed: {}", e) 
                })
            }
        }
    }
    
    pub async fn start_child(&mut self, child_spec: ChildSpec) -> OtpResult<ActorId> {
        let child_id = child_spec.id.clone();
        
        info!("Starting child: {}", child_id.name);
        
        // Start the child process
        let child_process = match self.start_single_child(child_spec.clone()).await {
            Ok(process) => process,
            Err(e) => {
                error!("Failed to start child {}: {:?}", child_id.name, e);
                return Err(e);
            }
        };
        
        // Register the child
        {
            let mut children = self.children.write().await;
            children.insert(child_id.clone(), child_process);
        }
        
        // Initialize restart counter
        {
            let mut restart_counts = self.restart_counts.write().await;
            restart_counts.insert(child_id.clone(), RestartCounter {
                count: 0,
                first_restart: std::time::Instant::now(),
            });
        }
        
        info!("Child {} started successfully", child_id.name);
        Ok(child_id)
    }
    
    pub async fn terminate_child(&mut self, child_id: ActorId) -> OtpResult<()> {
        info!("Terminating child: {}", child_id.name);
        
        // Remove from supervision tree
        let child_process = {
            let mut children = self.children.write().await;
            children.remove(&child_id)
        };
        
        if let Some(mut process) = child_process {
            // Stop the child process
            if let Some(stop_handle) = process.stop_handle.take() {
                let _ = stop_handle.send(TerminationReason::Normal);
            }
            
            // Wait for graceful shutdown
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            // Clean up restart counter
            let mut restart_counts = self.restart_counts.write().await;
            restart_counts.remove(&child_id);
            
            info!("Child {} terminated successfully", child_id.name);
            Ok(())
        } else {
            warn!("Child {} not found for termination", child_id.name);
            Err(OtpError::ActorNotFound { 
                actor_id: child_id.name 
            })
        }
    }
    
    pub async fn restart_child(&mut self, failed_child_id: ActorId) -> OtpResult<()> {
        info!("Restarting child: {}", failed_child_id.name);
        
        // Check restart limits
        if self.should_restart(&failed_child_id).await? {
            // Get child spec
            let child_spec = {
                let children = self.children.read().await;
                children.get(&failed_child_id)
                    .map(|p| p.spec.clone())
                    .ok_or_else(|| OtpError::ActorNotFound { 
                        actor_id: failed_child_id.name.clone() 
                    })?
            };
            
            // Terminate the failed child
            self.terminate_child(failed_child_id.clone()).await?;
            
            // Start new instance
            self.start_child(child_spec).await?;
            
            info!("Child {} restarted successfully", failed_child_id.name);
            Ok(())
        } else {
            error!("Restart limit exceeded for child {}", failed_child_id.name);
            Err(OtpError::RestartLimitExceeded { 
                actor_id: failed_child_id.name 
            })
        }
    }
    
    pub async fn which_children(&self) -> Vec<ChildInfo> {
        let children = self.children.read().await;
        let restart_counts = self.restart_counts.read().await;
        
        children
            .values()
            .map(|p| {
                let restart_count = restart_counts
                    .get(&p.id)
                    .map(|rc| rc.count)
                    .unwrap_or(0);
                
                ChildInfo {
                    id: p.id.clone(),
                    pid: None, // Could store process IDs if needed
                    type_: p.spec.type_.clone(),
                    restarts: restart_count,
                    running: p.is_running,
                }
            })
            .collect()
    }
    
    async fn start_single_child(&self, spec: ChildSpec) -> OtpResult<ChildProcess> {
        // Start the child using the start function
        let mut wrapper = (spec.start.start)().await?;
        let actor_id = spec.id.clone();
        
        // Start the wrapper
        wrapper.start().await?;
        
        let (stop_tx, _stop_rx) = oneshot::channel::<TerminationReason>();
        
        Ok(ChildProcess {
            id: actor_id.clone(),
            spec,
            restart_count: 0,
            last_restart: std::time::Instant::now(),
            is_running: true,
            stop_handle: Some(stop_tx),
            start_time: std::time::Instant::now(),
        })
    }
    
    async fn should_restart(&self, child_id: &ActorId) -> OtpResult<bool> {
        let restart_limit = match &self.restart_strategy {
            RestartStrategy::OneForOne { max_restarts, time_window }
            | RestartStrategy::OneForAll { max_restarts, time_window }
            | RestartStrategy::RestForOne { max_restarts, time_window }
            | RestartStrategy::SimpleOneForOne { max_restarts, time_window } => (*max_restarts, *time_window),
        };
        
        let mut restart_counts = self.restart_counts.write().await;
        let restart_counter = restart_counts
            .entry(child_id.clone())
            .or_insert_with(|| RestartCounter {
                count: 0,
                first_restart: std::time::Instant::now(),
            });
        
        // Check if we're within the time window
        if restart_counter.first_restart.elapsed() > restart_limit.1 {
            // Reset counter if window has expired
            restart_counter.count = 0;
            restart_counter.first_restart = std::time::Instant::now();
        }
        
        // Check if we've exceeded the restart limit
        if restart_counter.count >= restart_limit.0 {
            return Ok(false);
        }
        
        // Increment restart count
        restart_counter.count += 1;
        restart_counter.first_restart = std::time::Instant::now();
        
        Ok(true)
    }
    
    async fn start_failure_monitoring(&mut self) -> OtpResult<()> {
        // This would monitor child processes for failures
        // In a real implementation, this would use OS signals or other mechanisms
        info!("Failure monitoring started for supervisor {}", self.actor_id.name);
        Ok(())
    }
}

impl<S: Supervisor> SupervisorAddr<S> {
    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }
    
    pub async fn start_child(&self, spec: ChildSpec) -> OtpResult<ActorId> {
        // This would need to be implemented to send commands to the supervisor
        // For now, we'll return an error as it requires more complex messaging
        Err(OtpError::SupervisorError { 
            reason: "Not implemented - use direct supervisor instance".to_string() 
        })
    }
    
    pub async fn get_child_info(&self, child_id: &str) -> Option<ChildInfo> {
        let children = self.children.read().await;
        let restart_counts = self.restart_counts.read().await;
        
        children.values()
            .find(|p| p.id.name == child_id)
            .map(|p| {
                let restart_count = restart_counts
                    .get(&p.id)
                    .map(|rc| rc.count)
                    .unwrap_or(0);
                
                ChildInfo {
                    id: p.id.clone(),
                    pid: None,
                    type_: p.spec.type_.clone(),
                    restarts: restart_count,
                    running: p.is_running,
                }
            })
    }
    
    pub async fn list_children(&self) -> Vec<ChildInfo> {
        let children = self.children.read().await;
        let restart_counts = self.restart_counts.read().await;
        
        children
            .values()
            .map(|p| {
                let restart_count = restart_counts
                    .get(&p.id)
                    .map(|rc| rc.count)
                    .unwrap_or(0);
                
                ChildInfo {
                    id: p.id.clone(),
                    pid: None,
                    type_: p.spec.type_.clone(),
                    restarts: restart_count,
                    running: p.is_running,
                }
            })
            .collect()
    }
}

/// Utility for creating supervisors
pub struct SupervisorBuilder;

impl SupervisorBuilder {
    pub fn spawn<S: Supervisor + 'static>(
        name: impl Into<String>,
        supervisor: S,
        restart_strategy: RestartStrategy,
    ) -> OtpResult<SupervisorAddr<S>> {
        let actor_id = ActorId::local(name);
        let mut runtime = SupervisorRuntime::new(
            actor_id.clone(),
            supervisor,
            restart_strategy,
        );
        
        // Clone for the address
        let addr = SupervisorAddr {
            actor_id: actor_id.clone(),
            children: runtime.children.clone(),
            restart_counts: runtime.restart_counts.clone(),
        };
        
        // Start the supervisor in a separate task
        tokio::spawn(async move {
            if let Err(e) = runtime.start().await {
                error!("Supervisor {} failed: {:?}", actor_id.name, e);
            }
        });
        
        // Wait a brief moment for initialization
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        Ok(addr)
    }
}

/// Implementation of StartFunction for common actor types
pub struct GenServerStartFunction<G: GenServer> {
    pub gen_server: G,
    pub config: GenServerConfig,
}

impl<G: GenServer + 'static> GenServerStartFunction<G> {
    pub fn new(gen_server: G) -> Self {
        Self {
            gen_server,
            config: GenServerConfig::default(),
        }
    }
    
    pub fn with_config(gen_server: G, config: GenServerConfig) -> Self {
        Self { gen_server, config }
    }
}

#[async_trait]
impl<G: GenServer + 'static> StartFunction for GenServerStartFunction<G> {
    async fn start(&self) -> OtpResult<Box<dyn GenServerWrapper>> {
        let actor_id = ActorId::local(std::any::type_name::<G>());
        let addr = GenServerSpawner::spawn_with_config(
            actor_id,
            self.gen_server,
            self.config.clone(),
        )?;
        
        let wrapper = GenServerWrapperImpl {
            addr: Some(addr),
        };
        
        Ok(Box::new(wrapper))
    }
}

/// Wrapper implementation for GenServer addresses
pub struct GenServerWrapperImpl<G: GenServer> {
    addr: Option<GenServerAddr<G>>,
}

#[async_trait]
impl<G: GenServer> GenServerWrapper for GenServerWrapperImpl<G> {
    async fn start(&mut self) -> OtpResult<()> {
        // Already started in constructor
        Ok(())
    }
    
    async fn stop(&mut self, reason: TerminationReason) -> OtpResult<()> {
        if let Some(addr) = self.addr.take() {
            addr.stop(reason).await?;
        }
        Ok(())
    }
    
    fn actor_id(&self) -> &ActorId {
        match &self.addr {
            Some(addr) => addr.actor_id(),
            None => panic!("Actor not started"),
        }
    }
    
    fn is_running(&self) -> bool {
        match &self.addr {
            Some(addr) => addr.is_running(),
            None => false,
        }
    }
}