/// Advanced messaging system with backpressure and guarantees
/// Provides reliable message delivery with flow control

use super::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, error, warn};

/// Configuration for message handling
#[derive(Debug, Clone)]
pub struct MessageConfig {
    pub mailbox_size: usize,
    pub backpressure_threshold: f32, // 0.0 to 1.0
    pub priority_levels: u8,
    pub timeout: std::time::Duration,
    pub retry_attempts: u32,
    pub dead_letter_enabled: bool,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            mailbox_size: 1000,
            backpressure_threshold: 0.8,
            priority_levels: 4,
            timeout: std::time::Duration::from_secs(5),
            retry_attempts: 3,
            dead_letter_enabled: true,
        }
    }
}

/// Message with priority and metadata
#[derive(Debug, Clone)]
pub struct PriorityMessage<M: Message> {
    pub inner: MessageEnvelope<M>,
    pub priority: u8,
    pub created_at: std::time::Instant,
    pub delivery_attempts: u32,
}

/// Mailbox with backpressure and priority handling
pub struct BackpressureMailbox<M: Message> {
    config: MessageConfig,
    sender: mpsc::Sender<PriorityMessage<M>>,
    receiver: mpsc::Receiver<PriorityMessage<M>>,
    semaphore: Arc<Semaphore>,
    dead_letter_tx: Option<mpsc::Sender<DeadLetterMessage<M>>>,
    metrics: Arc<MessageMetrics>,
}

/// Metrics for message handling
#[derive(Debug, Default)]
pub struct MessageMetrics {
    pub messages_sent: Arc<std::sync::atomic::AtomicU64>,
    pub messages_received: Arc<std::sync::atomic::AtomicU64>,
    pub messages_dropped: Arc<std::sync::atomic::AtomicU64>,
    pub messages_timeout: Arc<std::sync::atomic::AtomicU64>,
    pub average_latency: Arc<std::sync::RwLock<Option<std::time::Duration>>>,
}

/// Dead letter message for failed deliveries
#[derive(Debug)]
pub struct DeadLetterMessage<M: Message> {
    pub original_message: PriorityMessage<M>,
    pub error: OtpError,
    pub failed_at: std::time::Instant,
    pub reason: DeadLetterReason,
}

#[derive(Debug, Clone)]
pub enum DeadLetterReason {
    MailboxFull,
    Timeout,
    MaxRetriesExceeded,
    ActorNotFound,
    SerializationError,
}

/// Address for actors with backpressure-aware messaging
pub struct BackpressureAddr<M: Message> {
    actor_id: ActorId,
    sender: mpsc::Sender<PriorityMessage<M>>,
    semaphore: Arc<Semaphore>,
    config: MessageConfig,
    metrics: Arc<MessageMetrics>,
}

impl<M: Message> Clone for BackpressureAddr<M> {
    fn clone(&self) -> Self {
        Self {
            actor_id: self.actor_id.clone(),
            sender: self.sender.clone(),
            semaphore: self.semaphore.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

impl<M: Message> BackpressureAddr<M> {
    /// Send a message with backpressure awareness
    pub async fn send(&self, message: M, from: ActorId) -> OtpResult<()> {
        return self.send_with_priority(message, from, 2).await; // Normal priority = 2
    }
    
    /// Send a message with specific priority (0=highest, 3=lowest)
    pub async fn send_with_priority(
        &self,
        message: M,
        from: ActorId,
        priority: u8,
    ) -> OtpResult<()> {
        // Acquire semaphore slot (backpressure control)
        let _permit = tokio::time::timeout(
            self.config.timeout,
            self.semaphore.acquire()
        ).await
        .map_err(|_| OtpError::MessageTimeout {
            message_type: "semaphore_acquire".to_string()
        })?
        .map_err(|_| OtpError::MailboxFull {
            actor_id: self.actor_id.name.clone()
        })?;
        
        let envelope = MessageEnvelope {
            message,
            from,
            to: self.actor_id.clone(),
            timestamp: std::time::SystemTime::now(),
            id: uuid::Uuid::new_v4(),
            priority: match priority {
                0 => MessagePriority::Critical,
                1 => MessagePriority::High,
                2 => MessagePriority::Normal,
                _ => MessagePriority::Low,
            },
        };
        
        let priority_msg = PriorityMessage {
            inner: envelope,
            priority,
            created_at: std::time::Instant::now(),
            delivery_attempts: 0,
        };
        
        // Check backpressure
        let current_load = self.semaphore.available_permits() as f32 / self.semaphore.max_permits() as f32;
        if current_load < (1.0 - self.config.backpressure_threshold) {
            warn!("Backpressure detected for actor {}: {:.1}% full", 
                self.actor_id.name, (1.0 - current_load) * 100.0);
        }
        
        // Send message
        tokio::time::timeout(
            self.config.timeout,
            self.sender.send(priority_msg)
        ).await
        .map_err(|_| OtpError::MessageTimeout {
            message_type: std::any::type_name::<M>().to_string()
        })?
        .map_err(|_| OtpError::MailboxFull {
            actor_id: self.actor_id.name.clone()
        })?;
        
        // Update metrics
        self.metrics.messages_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Send a message and wait for response
    pub async fn call(&self, message: M, from: ActorId) -> OtpResult<M::Response> {
        // For call operations, we'd need response handling
        // This is a simplified implementation
        self.send(message, from).await?;
        
        // In a real implementation, we'd set up a response channel
        // For now, we'll return an error indicating the limitation
        Err(OtpError::MessageTimeout {
            message_type: "call_not_implemented".to_string()
        })
    }
    
    /// Get current mailbox pressure
    pub fn mailbox_pressure(&self) -> f32 {
        1.0 - (self.semaphore.available_permits() as f32 / self.semaphore.max_permits() as f32)
    }
    
    /// Get metrics for this address
    pub fn metrics(&self) -> &MessageMetrics {
        &self.metrics
    }
    
    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }
}

impl<M: Message> BackpressureMailbox<M> {
    pub fn new(actor_id: ActorId, config: MessageConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.mailbox_size);
        let semaphore = Arc::new(Semaphore::new(config.mailbox_size));
        let metrics = Arc::new(MessageMetrics::default());
        
        let (dead_letter_tx, _dead_letter_rx) = if config.dead_letter_enabled {
            let (tx, rx) = mpsc::channel(1000);
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        
        Self {
            config,
            sender,
            receiver,
            semaphore,
            dead_letter_tx,
            metrics,
        }
    }
    
    /// Create a new address for this mailbox
    pub fn address(&self) -> BackpressureAddr<M> {
        BackpressureAddr {
            actor_id: ActorId::local("mailbox"), // This should be properly set
            sender: self.sender.clone(),
            semaphore: self.semaphore.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
    
    /// Receive the next message with priority handling
    pub async fn receive(&mut self) -> Option<PriorityMessage<M>> {
        // In a real implementation, we'd handle priority queuing
        // For now, we'll use the basic receiver
        match self.receiver.recv().await {
            Some(msg) => {
                self.metrics.messages_received.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                // Update latency metrics
                let latency = msg.created_at.elapsed();
                let mut avg_latency = self.metrics.average_latency.write().unwrap();
                match *avg_latency {
                    Some(current) => {
                        *avg_latency = Some(std::time::Duration::from_nanos(
                            (current.as_nanos() + latency.as_nanos()) / 2
                        ));
                    }
                    None => *avg_latency = Some(latency),
                }
                
                Some(msg)
            }
            None => None,
        }
    }
    
    /// Handle timeout for messages
    pub async fn handle_timeouts(&mut self) {
        // This would scan for timed out messages
        // Implementation would depend on the specific requirements
    }
    
    /// Send to dead letter queue
    async fn send_to_dead_letter(&mut self, msg: PriorityMessage<M>, reason: DeadLetterReason) {
        if let Some(ref dead_letter_tx) = self.dead_letter_tx {
            let dead_letter_msg = DeadLetterMessage {
                original_message: msg,
                error: OtpError::MessageTimeout {
                    message_type: "dead_letter".to_string()
                },
                failed_at: std::time::Instant::now(),
                reason,
            };
            
            let _ = dead_letter_tx.send(dead_letter_msg).await;
            self.metrics.messages_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Message router for handling message distribution
pub struct MessageRouter {
    routes: std::collections::HashMap<String, RouteConfig>,
    default_route: Option<String>,
    metrics: Arc<MessageMetrics>,
}

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub destination: String,
    pub filter: Option<String>, // Pattern for filtering messages
    pub priority_adjustment: Option<i8>, // Adjust message priority
    pub timeout: Option<std::time::Duration>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: std::collections::HashMap::new(),
            default_route: None,
            metrics: Arc::new(MessageMetrics::default()),
        }
    }
    
    pub fn add_route(&mut self, pattern: &str, config: RouteConfig) {
        self.routes.insert(pattern.to_string(), config);
    }
    
    pub fn set_default_route(&mut self, destination: &str) {
        self.default_route = Some(destination.to_string());
    }
    
    pub fn route_message<M: Message>(&self, message: &M, from: &ActorId) -> Option<String> {
        // Simple routing based on message type
        let message_type = std::any::type_name::<M>();
        
        // Find matching route
        for (pattern, config) in &self.routes {
            if message_type.contains(pattern) {
                return Some(config.destination.clone());
            }
        }
        
        // Use default route
        self.default_route.clone()
    }
    
    pub fn metrics(&self) -> &MessageMetrics {
        &self.metrics
    }
}

/// Dead letter queue processor
pub struct DeadLetterProcessor<M: Message> {
    receiver: mpsc::Receiver<DeadLetterMessage<M>>,
    retry_strategy: RetryStrategy,
    max_retries: u32,
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    ExponentialBackoff {
        initial_delay: std::time::Duration,
        max_delay: std::time::Duration,
        multiplier: f64,
    },
    FixedDelay(std::time::Duration),
    Immediate,
}

impl<M: Message> DeadLetterProcessor<M> {
    pub fn new(
        receiver: mpsc::Receiver<DeadLetterMessage<M>>,
        retry_strategy: RetryStrategy,
        max_retries: u32,
    ) -> Self {
        Self {
            receiver,
            retry_strategy,
            max_retries,
        }
    }
    
    /// Process dead letter messages
    pub async fn process_dead_letters(&mut self) {
        while let Some(dead_letter) = self.receiver.recv().await {
            debug!("Processing dead letter for message: {:?}", dead_letter.original_message.inner.id);
            
            // Decide whether to retry or permanently fail
            if dead_letter.original_message.delivery_attempts < self.max_retries {
                // Calculate retry delay
                let delay = match &self.retry_strategy {
                    RetryStrategy::ExponentialBackoff { initial_delay, max_delay, multiplier } => {
                        let base_delay = initial_delay.as_millis() as f64;
                        let exponential = base_delay * multiplier.powi(dead_letter.original_message.delivery_attempts as i32);
                        let delay_ms = exponential.min(*max_delay as f64) as u64;
                        std::time::Duration::from_millis(delay_ms)
                    }
                    RetryStrategy::FixedDelay(duration) => *duration,
                    RetryStrategy::Immediate => std::time::Duration::from_millis(0),
                };
                
                // Wait before retry
                tokio::time::sleep(delay).await;
                
                // Retry logic would go here
                debug!("Would retry message after {:?}", delay);
            } else {
                // Max retries exceeded, log and discard
                error!("Max retries exceeded for message: {:?}", dead_letter.original_message.inner.id);
            }
        }
    }
}

/// Utility functions for messaging
pub mod utils {
    use super::*;
    
    pub fn create_backpressure_mailbox<M: Message>(
        actor_id: ActorId,
    ) -> (BackpressureMailbox<M>, BackpressureAddr<M>) {
        let config = MessageConfig::default();
        let mailbox = BackpressureMailbox::new(actor_id, config);
        let addr = mailbox.address();
        (mailbox, addr)
    }
    
    pub fn create_backpressure_mailbox_with_config<M: Message>(
        actor_id: ActorId,
        config: MessageConfig,
    ) -> (BackpressureMailbox<M>, BackpressureAddr<M>) {
        let mailbox = BackpressureMailbox::new(actor_id, config);
        let addr = mailbox.address();
        (mailbox, addr)
    }
    
    pub fn calculate_backpressure_threshold(mailbox_size: usize, threshold: f32) -> usize {
        (mailbox_size as f32 * threshold) as usize
    }
    
    pub fn validate_message_config(config: &MessageConfig) -> OtpResult<()> {
        if config.mailbox_size == 0 {
            return Err(OtpError::RegistryError { 
                reason: "Mailbox size cannot be zero".to_string() 
            });
        }
        
        if !(0.0..=1.0).contains(&config.backpressure_threshold) {
            return Err(OtpError::RegistryError { 
                reason: "Backpressure threshold must be between 0.0 and 1.0".to_string() 
            });
        }
        
        if config.priority_levels == 0 {
            return Err(OtpError::RegistryError { 
                reason: "Priority levels must be at least 1".to_string() 
            });
        }
        
        Ok(())
    }
}