/// GenServer implementation core
/// Provides the actual runtime behavior for GenServer actors

use super::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};

/// Runtime implementation of a GenServer actor
pub struct GenServerRuntime<G: GenServer> {
    actor_id: ActorId,
    gen_server: G,
    state: Option<G::State>,
    mailbox_rx: mpsc::Receiver<InternalMessage<G>>,
    shutdown_tx: Option<oneshot::Sender<TerminationReason>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

/// Internal messages for GenServer communication
enum InternalMessage<G: GenServer> {
    Call {
        message: G::Message,
        from: ActorId,
        response_tx: oneshot::Sender<OtpResult<G::CallResponse>>,
    },
    Cast {
        message: G::Message,
        from: ActorId,
        response_tx: oneshot::Sender<OtpResult<G::CastResponse>>,
    },
    Info(G::Info),
    CodeChange {
        old_vsn: String,
        extra: String,
        response_tx: oneshot::Sender<OtpResult<()>>,
    },
    Stop(TerminationReason),
}

/// Address for communicating with a GenServer
pub struct GenServerAddr<G: GenServer> {
    actor_id: ActorId,
    mailbox_tx: mpsc::Sender<InternalMessage<G>>,
    shutdown_tx: oneshot::Sender<TerminationReason>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl<G: GenServer> Clone for GenServerAddr<G> {
    fn clone(&self) -> Self {
        Self {
            actor_id: self.actor_id.clone(),
            mailbox_tx: self.mailbox_tx.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

impl<G: GenServer> GenServerRuntime<G> {
    pub fn new(
        actor_id: ActorId,
        gen_server: G,
        mailbox_rx: mpsc::Receiver<InternalMessage<G>>,
        mailbox_tx: mpsc::Sender<InternalMessage<G>>,
    ) -> (Self, GenServerAddr<G>) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        let addr = GenServerAddr {
            actor_id: actor_id.clone(),
            mailbox_tx,
            shutdown_tx,
            is_running: is_running.clone(),
        };
        
        let runtime = Self {
            actor_id,
            gen_server,
            state: None,
            mailbox_rx,
            shutdown_tx: Some(shutdown_rx),
            is_running,
        };
        
        (runtime, addr)
    }
    
    pub async fn start(&mut self) -> OtpResult<()> {
        info!("Starting GenServer: {}", self.actor_id.name);
        
        // Initialize the GenServer
        match self.gen_server.init().await {
            Ok(state) => {
                self.state = Some(state);
                self.is_running.store(true, std::sync::atomic::Ordering::Relaxed);
                debug!("GenServer {} initialized successfully", self.actor_id.name);
            }
            Err(e) => {
                error!("Failed to initialize GenServer {}: {:?}", self.actor_id.name, e);
                return Err(e);
            }
        }
        
        // Start the message processing loop
        self.message_loop().await
    }
    
    async fn message_loop(&mut self) -> OtpResult<()> {
        let state = self.state.as_mut().expect("State must be initialized");
        let shutdown_rx = self.shutdown_rx.take().expect("Shutdown receiver must exist");
        
        loop {
            tokio::select! {
                // Handle incoming messages
                internal_msg = self.mailbox_rx.recv() => {
                    match internal_msg {
                        Some(msg) => {
                            if let Err(e) = self.handle_internal_message(msg, state).await {
                                error!("Error handling message in {}: {:?}", self.actor_id.name, e);
                            }
                        }
                        None => {
                            debug!("Mailbox closed for {}", self.actor_id.name);
                            break;
                        }
                    }
                }
                
                // Handle shutdown signal
                reason = &mut shutdown_rx => {
                    match reason {
                        Ok(reason) => {
                            info!("GenServer {} shutting down: {:?}", self.actor_id.name, reason);
                            self.gen_server.terminate(reason.clone(), state).await;
                            self.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                            return Ok(());
                        }
                        Err(_) => {
                            debug!("Shutdown signal lost for {}", self.actor_id.name);
                            break;
                        }
                    }
                }
            }
        }
        
        // Normal termination
        self.gen_server.terminate(TerminationReason::Normal, state).await;
        self.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    async fn handle_internal_message(
        &mut self,
        msg: InternalMessage<G>,
        state: &mut G::State,
    ) -> OtpResult<()> {
        match msg {
            InternalMessage::Call { message, from, response_tx } => {
                debug!("Handling call in {} from {}", self.actor_id.name, from.name);
                
                match self.gen_server.handle_call(message, from, state).await {
                    Ok(response) => {
                        let _ = response_tx.send(Ok(response));
                    }
                    Err(e) => {
                        let _ = response_tx.send(Err(e));
                    }
                }
            }
            
            InternalMessage::Cast { message, from, response_tx } => {
                debug!("Handling cast in {} from {}", self.actor_id.name, from.name);
                
                match self.gen_server.handle_cast(message, from, state).await {
                    Ok(response) => {
                        let _ = response_tx.send(Ok(response));
                    }
                    Err(e) => {
                        let _ = response_tx.send(Err(e));
                    }
                }
            }
            
            InternalMessage::Info(info) => {
                debug!("Handling info in {}: {:?}", self.actor_id.name, info);
                
                if let Err(e) = self.gen_server.handle_info(info, state).await {
                    error!("Error handling info in {}: {:?}", self.actor_id.name, e);
                }
            }
            
            InternalMessage::CodeChange { old_vsn, extra, response_tx } => {
                debug!("Handling code change in {} from version {}", self.actor_id.name, old_vsn);
                
                let result = self.gen_server.code_change(&old_vsn, state, &extra).await;
                let _ = response_tx.send(result);
            }
            
            InternalMessage::Stop(reason) => {
                info!("GenServer {} stopping: {:?}", self.actor_id.name, reason);
                self.gen_server.terminate(reason, state).await;
                self.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
        
        Ok(())
    }
}

impl<G: GenServer> GenServerAddr<G> {
    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }
    
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    pub async fn call(&self, message: G::Message, from: ActorId) -> OtpResult<G::CallResponse> {
        if !self.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: self.actor_id.name.clone() 
            });
        }
        
        let (response_tx, response_rx) = oneshot::channel();
        
        let msg = InternalMessage::Call {
            message,
            from,
            response_tx,
        };
        
        self.mailbox_tx
            .send(msg)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: self.actor_id.name.clone() 
            })?;
        
        response_rx
            .await
            .map_err(|_| OtpError::MessageTimeout { 
                message_type: std::any::type_name::<G::Message>().to_string() 
            })?
    }
    
    pub async fn cast(&self, message: G::Message, from: ActorId) -> OtpResult<G::CastResponse> {
        if !self.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: self.actor_id.name.clone() 
            });
        }
        
        let (response_tx, response_rx) = oneshot::channel();
        
        let msg = InternalMessage::Cast {
            message,
            from,
            response_tx,
        };
        
        self.mailbox_tx
            .send(msg)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: self.actor_id.name.clone() 
            })?;
        
        response_rx
            .await
            .map_err(|_| OtpError::MessageTimeout { 
                message_type: std::any::type_name::<G::Message>().to_string() 
            })?
    }
    
    pub async fn send_info(&self, info: G::Info) -> OtpResult<()> {
        if !self.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: self.actor_id.name.clone() 
            });
        }
        
        let msg = InternalMessage::Info(info);
        
        self.mailbox_tx
            .send(msg)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: self.actor_id.name.clone() 
            })?;
        
        Ok(())
    }
    
    pub async fn stop(&self, reason: TerminationReason) -> OtpResult<()> {
        debug!("Sending stop signal to {}: {:?}", self.actor_id.name, reason);
        
        if let Err(e) = self.shutdown_tx.send(reason) {
            error!("Failed to send stop signal to {}: {:?}", self.actor_id.name, e);
            return Err(OtpError::ActorNotFound { 
                actor_id: self.actor_id.name.clone() 
            });
        }
        
        Ok(())
    }
    
    pub async fn code_change(&self, old_vsn: String, extra: String) -> OtpResult<()> {
        if !self.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: self.actor_id.name.clone() 
            });
        }
        
        let (response_tx, response_rx) = oneshot::channel();
        
        let msg = InternalMessage::CodeChange { old_vsn, extra, response_tx };
        
        self.mailbox_tx
            .send(msg)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: self.actor_id.name.clone() 
            })?;
        
        response_rx
            .await
            .map_err(|_| OtpError::MessageTimeout { 
                message_type: "code_change".to_string() 
            })?
    }
}

/// Spawner utility for GenServer instances
pub struct GenServerSpawner;

impl GenServerSpawner {
    pub fn spawn<G: GenServer + 'static>(
        name: impl Into<String>,
        gen_server: G,
        mailbox_size: usize,
    ) -> OtpResult<GenServerAddr<G>> {
        let actor_id = ActorId::local(name);
        let (mailbox_tx, mailbox_rx) = mpsc::channel(mailbox_size);
        
        let (mut runtime, addr) = GenServerRuntime::new(
            actor_id.clone(),
            gen_server,
            mailbox_rx,
            mailbox_tx,
        );
        
        // Spawn the runtime in a separate task
        tokio::spawn(async move {
            if let Err(e) = runtime.start().await {
                error!("GenServer {} failed: {:?}", actor_id.name, e);
            }
        });
        
        // Wait a brief moment for initialization
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        if !addr.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: actor_id.name 
            });
        }
        
        Ok(addr)
    }
    
    pub fn spawn_with_config<G: GenServer + 'static>(
        actor_id: ActorId,
        gen_server: G,
        config: GenServerConfig,
    ) -> OtpResult<GenServerAddr<G>> {
        let (mailbox_tx, mailbox_rx) = mpsc::channel(config.mailbox_size);
        
        let (mut runtime, addr) = GenServerRuntime::new(
            actor_id.clone(),
            gen_server,
            mailbox_rx,
            mailbox_tx,
        );
        
        // Spawn the runtime in a separate task
        tokio::spawn(async move {
            if let Err(e) = runtime.start().await {
                error!("GenServer {} failed: {:?}", actor_id.name, e);
            }
        });
        
        // Wait for initialization
        tokio::time::sleep(std::time::Duration::from_millis(config.init_timeout)).await;
        
        if !addr.is_running() {
            return Err(OtpError::ActorNotFound { 
                actor_id: actor_id.name 
            });
        }
        
        Ok(addr)
    }
}

/// Configuration for GenServer spawning
#[derive(Debug, Clone)]
pub struct GenServerConfig {
    pub mailbox_size: usize,
    pub init_timeout: u64,
}

impl Default for GenServerConfig {
    fn default() -> Self {
        Self {
            mailbox_size: 1000,
            init_timeout: 1000,
        }
    }
}