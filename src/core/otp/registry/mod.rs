/// Registry system for actor discovery and naming
/// Provides distributed service discovery and addressing

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, error, info, warn};

/// Global registry for managing all actors in the system
pub struct GlobalRegistry {
    local_registry: Arc<RwLock<HashMap<String, RegisteredActor>>>,
    distributed_registry: Arc<DistributedRegistry>,
    event_tx: broadcast::Sender<RegistryEvent>,
    _event_rx: broadcast::Receiver<RegistryEvent>,
}

/// Information about a registered actor
#[derive(Debug, Clone)]
pub struct RegisteredActor {
    pub actor_id: ActorId,
    pub address_type: AddressType,
    pub registered_at: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum AddressType {
    Local(tokio::sync::mpsc::Sender<RegistryMessage>),
    Remote(RemoteAddress),
}

#[derive(Debug, Clone)]
pub struct RemoteAddress {
    pub node_id: String,
    pub endpoint: String,
    pub protocol: String,
}

/// Events emitted by the registry
#[derive(Debug, Clone)]
pub enum RegistryEvent {
    ActorRegistered { name: String, actor_id: ActorId },
    ActorUnregistered { name: String },
    ActorDied { name: String, reason: String },
    NodeJoined { node_id: String },
    NodeLeft { node_id: String },
}

/// Messages to/from registry
#[derive(Debug)]
pub enum RegistryMessage {
    Register {
        name: String,
        actor_id: ActorId,
        response_tx: tokio::sync::oneshot::Sender<OtpResult<()>>,
    },
    Unregister {
        name: String,
        response_tx: tokio::sync::oneshot::Sender<OtpResult<()>>,
    },
    Lookup {
        name: String,
        response_tx: tokio::sync::oneshot::Sender<OtpResult<Option<RegisteredActor>>>,
    },
    List {
        response_tx: tokio::sync::oneshot::Sender<OtpResult<Vec<String>>>,
    },
    Send {
        name: String,
        message: Vec<u8>, // Serialized message
        from: ActorId,
        response_tx: tokio::sync::oneshot::Sender<OtpResult<Vec<u8>>>,
    },
}

/// Distributed registry for multi-node environments
pub struct DistributedRegistry {
    local_node_id: String,
    known_nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    replication_factor: usize,
}

/// Information about a node in the cluster
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub last_heartbeat: std::time::Instant,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Active,
    Suspicious,
    Failed,
    Left,
}

impl GlobalRegistry {
    pub fn new(local_node_id: String) -> Self {
        let (event_tx, event_rx) = broadcast::channel(1000);
        
        Self {
            local_registry: Arc::new(RwLock::new(HashMap::new())),
            distributed_registry: Arc::new(DistributedRegistry::new(local_node_id)),
            event_tx,
            _event_rx: event_rx,
        }
    }
    
    pub async fn register_actor<M: Message>(
        &self,
        name: &str,
        addr: impl Into<AddressType>,
        metadata: HashMap<String, String>,
    ) -> OtpResult<()> {
        let address_type = addr.into();
        let actor_id = match &address_type {
            AddressType::Local(_) => ActorId::local(name),
            AddressType::Remote(remote) => ActorId::distributed(name, &remote.node_id),
        };
        
        let registered_actor = RegisteredActor {
            actor_id: actor_id.clone(),
            address_type,
            registered_at: std::time::SystemTime::now(),
            metadata,
        };
        
        // Register locally
        {
            let mut registry = self.local_registry.write().await;
            registry.insert(name.to_string(), registered_actor);
        }
        
        // Replicate to other nodes
        self.distributed_registry.replicate_registration(name, &actor_id).await?;
        
        // Emit event
        let _ = self.event_tx.send(RegistryEvent::ActorRegistered {
            name: name.to_string(),
            actor_id,
        });
        
        info!("Actor '{}' registered successfully", name);
        Ok(())
    }
    
    pub async fn unregister_actor(&self, name: &str) -> OtpResult<()> {
        // Remove from local registry
        {
            let mut registry = self.local_registry.write().await;
            registry.remove(name);
        }
        
        // Replicate to other nodes
        self.distributed_registry.replicate_unregistration(name).await?;
        
        // Emit event
        let _ = self.event_tx.send(RegistryEvent::ActorUnregistered {
            name: name.to_string(),
        });
        
        info!("Actor '{}' unregistered successfully", name);
        Ok(())
    }
    
    pub async fn lookup_actor(&self, name: &str) -> OtpResult<Option<RegisteredActor>> {
        // Check local registry first
        {
            let registry = self.local_registry.read().await;
            if let Some(actor) = registry.get(name) {
                return Ok(Some(actor.clone()));
            }
        }
        
        // Check distributed registry
        self.distributed_registry.lookup_distributed(name).await
    }
    
    pub async fn list_actors(&self) -> OtpResult<Vec<String>> {
        let local_names: Vec<String> = {
            let registry = self.local_registry.read().await;
            registry.keys().cloned().collect()
        };
        
        let distributed_names = self.distributed_registry.list_distributed().await?;
        
        // Merge and deduplicate
        let mut all_names = local_names;
        all_names.extend(distributed_names);
        all_names.sort();
        all_names.dedup();
        
        Ok(all_names)
    }
    
    pub async fn send_message<M: Message>(
        &self,
        name: &str,
        message: M,
        from: ActorId,
    ) -> OtpResult<M::Response> {
        // Look up target actor
        let actor = self.lookup_actor(name).await?
            .ok_or_else(|| OtpError::ActorNotFound { 
                actor_id: name.to_string() 
            })?;
        
        match actor.address_type {
            AddressType::Local(sender) => {
                // Send to local actor
                self.send_local_message(sender, message, from).await
            }
            AddressType::Remote(remote) => {
                // Send to remote actor
                self.send_remote_message(remote, name, message, from).await
            }
        }
    }
    
    pub async fn cast_message<M: Message>(
        &self,
        name: &str,
        message: M,
        from: ActorId,
    ) -> OtpResult<()> {
        // Look up target actor
        let actor = self.lookup_actor(name).await?
            .ok_or_else(|| OtpError::ActorNotFound { 
                actor_id: name.to_string() 
            })?;
        
        match actor.address_type {
            AddressType::Local(sender) => {
                // Send to local actor
                self.cast_local_message(sender, message, from).await
            }
            AddressType::Remote(remote) => {
                // Send to remote actor
                self.cast_remote_message(remote, name, message, from).await
            }
        }
    }
    
    pub fn subscribe_events(&self) -> broadcast::Receiver<RegistryEvent> {
        self.event_tx.subscribe()
    }
    
    async fn send_local_message<M: Message>(
        &self,
        sender: tokio::sync::mpsc::Sender<RegistryMessage>,
        message: M,
        from: ActorId,
    ) -> OtpResult<M::Response> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        let serialized_message = serde_json::to_vec(&message)
            .map_err(|e| OtpError::RegistryError { 
                reason: format!("Serialization failed: {}", e) 
            })?;
        
        let registry_message = RegistryMessage::Send {
            name: std::any::type_name::<M>(),
            message: serialized_message,
            from,
            response_tx,
        };
        
        sender
            .send(registry_message)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: "registry".to_string() 
            })?;
        
        let response_bytes = response_rx
            .await
            .map_err(|_| OtpError::MessageTimeout { 
                message_type: std::any::type_name::<M>().to_string() 
            })??;
        
        serde_json::from_slice(&response_bytes)
            .map_err(|e| OtpError::RegistryError { 
                reason: format!("Deserialization failed: {}", e) 
            })
    }
    
    async fn cast_local_message<M: Message>(
        &self,
        sender: tokio::sync::mpsc::Sender<RegistryMessage>,
        message: M,
        from: ActorId,
    ) -> OtpResult<()> {
        let serialized_message = serde_json::to_vec(&message)
            .map_err(|e| OtpError::RegistryError { 
                reason: format!("Serialization failed: {}", e) 
            })?;
        
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        
        let registry_message = RegistryMessage::Send {
            name: std::any::type_name::<M>(),
            message: serialized_message,
            from,
            response_tx,
        };
        
        sender
            .send(registry_message)
            .await
            .map_err(|_| OtpError::MailboxFull { 
                actor_id: "registry".to_string() 
            })?;
        
        Ok(())
    }
    
    async fn send_remote_message<M: Message>(
        &self,
        remote: RemoteAddress,
        name: &str,
        message: M,
        from: ActorId,
    ) -> OtpResult<M::Response> {
        // This would implement actual remote messaging
        // For now, return an error indicating not implemented
        Err(OtpError::RegistryError { 
            reason: format!("Remote messaging to {} not yet implemented", remote.node_id) 
        })
    }
    
    async fn cast_remote_message<M: Message>(
        &self,
        remote: RemoteAddress,
        name: &str,
        message: M,
        from: ActorId,
    ) -> OtpResult<()> {
        // This would implement actual remote casting
        // For now, return an error indicating not implemented
        Err(OtpError::RegistryError { 
            reason: format!("Remote casting to {} not yet implemented", remote.node_id) 
        })
    }
}

impl DistributedRegistry {
    pub fn new(local_node_id: String) -> Self {
        Self {
            local_node_id,
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            replication_factor: 3,
        }
    }
    
    pub async fn replicate_registration(
        &self,
        name: &str,
        actor_id: &ActorId,
    ) -> OtpResult<()> {
        // This would implement replication to other nodes
        // For now, just log the action
        debug!("Replicating registration of '{}' to distributed registry", name);
        Ok(())
    }
    
    pub async fn replicate_unregistration(&self, name: &str) -> OtpResult<()> {
        // This would implement replication to other nodes
        debug!("Replicating unregistration of '{}' to distributed registry", name);
        Ok(())
    }
    
    pub async fn lookup_distributed(&self, name: &str) -> OtpResult<Option<RegisteredActor>> {
        // This would implement distributed lookup
        // For now, return None
        Ok(None)
    }
    
    pub async fn list_distributed(&self) -> OtpResult<Vec<String>> {
        // This would implement distributed listing
        // For now, return empty list
        Ok(Vec::new())
    }
    
    pub async fn add_node(&self, node_info: NodeInfo) -> OtpResult<()> {
        let mut nodes = self.known_nodes.write().await;
        nodes.insert(node_info.node_id.clone(), node_info.clone());
        
        info!("Added node {} to cluster", node_info.node_id);
        Ok(())
    }
    
    pub async fn remove_node(&self, node_id: &str) -> OtpResult<()> {
        let mut nodes = self.known_nodes.write().await;
        nodes.remove(node_id);
        
        info!("Removed node {} from cluster", node_id);
        Ok(())
    }
    
    pub async fn get_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.known_nodes.read().await;
        nodes.values().cloned().collect()
    }
}

#[async_trait]
impl Registry for GlobalRegistry {
    async fn register<M: Message>(&self, name: &str, addr: ActorAddr<M>) -> OtpResult<()> {
        // Convert ActorAddr to AddressType
        let address_type = AddressType::Local(tokio::sync::mpsc::channel(1000).0); // Placeholder
        let metadata = HashMap::new();
        
        self.register_actor(name, address_type, metadata).await
    }
    
    async fn unregister(&self, name: &str) -> OtpResult<()> {
        self.unregister_actor(name).await
    }
    
    async fn lookup<M: Message>(&self, name: &str) -> OtpResult<Option<ActorAddr<M>>> {
        // This would need to convert RegisteredActor back to ActorAddr
        // For now, return None as the conversion is complex
        Ok(None)
    }
    
    async fn list_registered(&self) -> OtpResult<Vec<String>> {
        self.list_actors().await
    }
    
    async fn send<M: Message>(&self, name: &str, message: M, from: ActorId) -> OtpResult<M::Response> {
        self.send_message(name, message, from).await
    }
    
    async fn cast<M: Message>(&self, name: &str, message: M, from: ActorId) -> OtpResult<()> {
        self.cast_message(name, message, from).await
    }
}

/// Utility functions for working with the registry
pub mod utils {
    use super::*;
    
    pub fn create_global_registry(node_id: impl Into<String>) -> Arc<GlobalRegistry> {
        Arc::new(GlobalRegistry::new(node_id.into()))
    }
    
    pub fn generate_local_node_id() -> String {
        format!("node-{}", uuid::Uuid::new_v4())
    }
    
    pub fn validate_actor_name(name: &str) -> OtpResult<()> {
        if name.is_empty() {
            return Err(OtpError::RegistryError { 
                reason: "Actor name cannot be empty".to_string() 
            });
        }
        
        if name.len() > 255 {
            return Err(OtpError::RegistryError { 
                reason: "Actor name too long (max 255 characters)".to_string() 
            });
        }
        
        // Check for invalid characters
        if name.contains('/') || name.contains('@') || name.contains(' ') {
            return Err(OtpError::RegistryError { 
                reason: "Actor name contains invalid characters".to_string() 
            });
        }
        
        Ok(())
    }
}