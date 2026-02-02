/// Core OTP traits and behaviors for the Olympus v11 system
/// This module provides the fundamental building blocks for implementing
/// Erlang/OTP-like patterns in Rust

use async_trait::async_trait;
use std::fmt::Debug;
use std::time::Duration;

// Core Error Types
#[derive(Debug, thiserror::Error)]
pub enum OtpError {
    #[error("Actor not found: {actor_id}")]
    ActorNotFound { actor_id: String },
    
    #[error("Mailbox full: {actor_id}")]
    MailboxFull { actor_id: String },
    
    #[error("Supervisor error: {reason}")]
    SupervisorError { reason: String },
    
    #[error("Restart limit exceeded: {actor_id}")]
    RestartLimitExceeded { actor_id: String },
    
    #[error("Message timeout: {message_type}")]
    MessageTimeout { message_type: String },
    
    #[error("Registry error: {reason}")]
    RegistryError { reason: String },
}

pub type OtpResult<T> = Result<T, OtpError>;

// Core trait for all messages in the system
pub trait Message: Send + Sync + Debug + 'static {
    type Response: Send + Sync + Debug + 'static;
}

// Generic envelope for all messages with metadata
#[derive(Debug, Clone)]
pub struct MessageEnvelope<M: Message> {
    pub message: M,
    pub from: ActorId,
    pub to: ActorId,
    pub timestamp: std::time::SystemTime,
    pub id: uuid::Uuid,
    pub priority: MessagePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriority {
    Critical,
    High,
    Normal,
    Low,
}

// Unique identifier for actors in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ActorId {
    pub name: String,
    pub node: Option<String>,
    pub process_id: Option<u64>,
}

impl ActorId {
    pub fn local(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            node: None,
            process_id: None,
        }
    }
    
    pub fn distributed(name: impl Into<String>, node: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            node: Some(node.into()),
            process_id: None,
        }
    }
    
    pub fn with_process(mut self, process_id: u64) -> Self {
        self.process_id = Some(process_id);
        self
    }
}

// Actor address for sending messages
#[derive(Debug, Clone)]
pub struct ActorAddr<M: Message> {
    pub actor_id: ActorId,
    pub sender: tokio::sync::mpsc::Sender<MessageEnvelope<M>>,
    pub node_id: Option<String>,
}

impl<M: Message> ActorAddr<M> {
    pub async fn send(&self, message: M, from: ActorId) -> OtpResult<()> {
        let envelope = MessageEnvelope {
            message,
            from,
            to: self.actor_id.clone(),
            timestamp: std::time::SystemTime::now(),
            id: uuid::Uuid::new_v4(),
            priority: MessagePriority::Normal,
        };
        
        self.sender
            .send(envelope)
            .await
            .map_err(|_| OtpError::ActorNotFound { actor_id: self.actor_id.name.clone() })
    }
    
    pub async fn call(&self, message: M, from: ActorId) -> OtpResult<M::Response> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        let envelope = MessageEnvelope {
            message,
            from,
            to: self.actor_id.clone(),
            timestamp: std::time::SystemTime::now(),
            id: uuid::Uuid::new_v4(),
            priority: MessagePriority::Normal,
        };
        
        self.sender
            .send(envelope)
            .await
            .map_err(|_| OtpError::ActorNotFound { actor_id: self.actor_id.name.clone() })?;
        
        response_rx
            .await
            .map_err(|_| OtpError::MessageTimeout { message_type: std::any::type_name::<M>().to_string() })
    }
}

// GenServer trait - core behavior for all actors
#[async_trait]
pub trait GenServer: Send + Sync + 'static {
    type State: Send + Sync;
    type Message: Message;
    type CallResponse: Send + Sync;
    type CastResponse: Send + Sync;
    type Info: Send + Sync + Debug;

    /// Initialize the GenServer with its initial state
    async fn init(&mut self) -> OtpResult<Self::State>;
    
    /// Handle synchronous calls (request-response)
    async fn handle_call(
        &mut self,
        message: Self::Message,
        from: ActorId,
        state: &mut Self::State,
    ) -> OtpResult<Self::CallResponse>;
    
    /// Handle asynchronous casts (fire-and-forget)
    async fn handle_cast(
        &mut self,
        message: Self::Message,
        from: ActorId,
        state: &mut Self::State,
    ) -> OtpResult<Self::CastResponse>;
    
    /// Handle internal messages (system events, timeouts, etc.)
    async fn handle_info(
        &mut self,
        info: Self::Info,
        state: &mut Self::State,
    ) -> OtpResult<()>;
    
    /// Called when the GenServer is terminating
    async fn terminate(
        &mut self,
        reason: TerminationReason,
        state: &mut Self::State,
    );
    
    /// Called for code hot-swapping (advanced feature)
    async fn code_change(
        &mut self,
        old_vsn: &str,
        state: &mut Self::State,
        extra: &str,
    ) -> OtpResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TerminationReason {
    Normal,
    Shutdown,
    MaxRestartsExceeded,
    CriticalError(String),
    SupervisorShutdown,
}

// Restart strategies for supervisors
#[derive(Debug, Clone)]
pub enum RestartStrategy {
    /// Restart only the failed child
    OneForOne {
        max_restarts: u32,
        time_window: Duration,
    },
    /// Terminate all children and restart all
    OneForAll {
        max_restarts: u32,
        time_window: Duration,
    },
    /// Terminate and restart the failed child and all subsequent children
    RestForOne {
        max_restarts: u32,
        time_window: Duration,
    },
    /// Dynamic supervisor with unlimited children
    SimpleOneForOne {
        max_restarts: u32,
        time_window: Duration,
    },
}

// Child specification for supervisors
#[derive(Debug, Clone)]
pub struct ChildSpec {
    pub id: ActorId,
    pub start: Box<dyn StartFunction>,
    pub restart: RestartPolicy,
    pub shutdown: ShutdownPolicy,
    pub type_: ChildType,
}

#[derive(Debug, Clone)]
pub enum RestartPolicy {
    Permanent,      // Always restart
    Transient,      // Restart only if termination is abnormal
    Temporary,      // Never restart
}

#[derive(Debug, Clone)]
pub enum ShutdownPolicy {
    Immediate,      // Immediate termination
    Timeout(Duration), // Wait for graceful shutdown
    Infinity,       // Wait indefinitely
}

#[derive(Debug, Clone)]
pub enum ChildType {
    Worker,         // Regular worker process
    Supervisor,     // Supervisor process
}

// Function trait for starting children
#[async_trait]
pub trait StartFunction: Send + Sync {
    async fn start(&self) -> OtpResult<Box<dyn GenServerWrapper>>;
}

// Wrapper trait for GenServer instances
#[async_trait]
pub trait GenServerWrapper: Send + Sync {
    async fn start(&mut self) -> OtpResult<()>;
    async fn stop(&mut self, reason: TerminationReason) -> OtpResult<()>;
    fn actor_id(&self) -> &ActorId;
    fn is_running(&self) -> bool;
}

// Supervisor trait
#[async_trait]
pub trait Supervisor: Send + Sync + 'static {
    type Config: Send + Sync;
    type ChildSpec: Send + Sync;

    async fn init(&mut self) -> OtpResult<Self::Config>;
    async fn start_child(&mut self, spec: Self::ChildSpec) -> OtpResult<ActorId>;
    async fn terminate_child(&mut self, id: ActorId) -> OtpResult<()>;
    async fn restart_child(&mut self, id: ActorId) -> OtpResult<()>;
    fn restart_strategy(&self) -> RestartStrategy;
    async fn which_children(&self) -> Vec<ChildInfo>;
}

#[derive(Debug, Clone)]
pub struct ChildInfo {
    pub id: ActorId,
    pub pid: Option<u64>,
    pub type_: ChildType,
    pub restarts: u32,
    pub running: bool,
}

// Registry trait for actor discovery
#[async_trait]
pub trait Registry: Send + Sync + 'static {
    async fn register<M: Message>(&self, name: &str, addr: ActorAddr<M>) -> OtpResult<()>;
    async fn unregister(&self, name: &str) -> OtpResult<()>;
    async fn lookup<M: Message>(&self, name: &str) -> OtpResult<Option<ActorAddr<M>>>;
    async fn list_registered(&self) -> OtpResult<Vec<String>>;
    async fn send<M: Message>(&self, name: &str, message: M, from: ActorId) -> OtpResult<M::Response>;
    async fn cast<M: Message>(&self, name: &str, message: M, from: ActorId) -> OtpResult<()>;
}

// Application behavior for managing entire applications
#[async_trait]
pub trait Application: Send + Sync + 'static {
    type Config: Send + Sync;
    
    async fn start(&mut self, config: Self::Config) -> OtpResult<()>;
    async fn stop(&mut self) -> OtpResult<()>;
    async fn config_change(&mut self, config: Self::Config) -> OtpResult<()>;
}

// Export submodules
pub mod genserver;
pub mod supervisor;
pub mod registry;
pub mod strategies;

// Re-export commonly used items
pub use genserver::*;
pub use supervisor::*;
pub use registry::*;
pub use strategies::*;

// Utility functions
pub mod utils {
    use super::*;
    
    pub fn generate_actor_id(name: impl Into<String>) -> ActorId {
        ActorId::local(name)
    }
    
    pub fn generate_distributed_actor_id(name: impl Into<String>, node: impl Into<String>) -> ActorId {
        ActorId::distributed(name, node)
    }
    
    pub fn format_termination_reason(reason: &TerminationReason) -> String {
        match reason {
            TerminationReason::Normal => "Normal termination".to_string(),
            TerminationReason::Shutdown => "System shutdown".to_string(),
            TerminationReason::MaxRestartsExceeded => "Maximum restarts exceeded".to_string(),
            TerminationReason::CriticalError(msg) => format!("Critical error: {}", msg),
            TerminationReason::SupervisorShutdown => "Supervisor shutdown".to_string(),
        }
    }
}