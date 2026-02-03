// src/actors/poseidon/websocket.rs
// OLYMPUS v13 - Poseidon WebSocket Manager
// Gestión de conexiones WebSocket con auto-reconexión

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub url: String,
    pub domain: super::DivineDomain,
    pub status: ConnectionStatus,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub message_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
    Failed,
}

#[derive(Debug, Clone)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    pending: Arc<RwLock<Vec<PendingConnection>>>,
}

#[derive(Debug, Clone)]
struct PendingConnection {
    url: String,
    domain: super::DivineDomain,
    attempt: u32,
    max_attempts: u32,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn connect(&self, url: &str, domain: super::DivineDomain) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        
        // Add to pending
        let mut pending = self.pending.write().await;
        pending.push(PendingConnection {
            url: url.to_string(),
            domain,
            attempt: 0,
            max_attempts: 10,
        });
        
        Ok(id)
    }
    
    pub async fn disconnect(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.status = ConnectionStatus::Disconnected;
        }
    }
    
    pub async fn close_all(&self) {
        let mut connections = self.connections.write().await;
        for conn in connections.values_mut() {
            conn.status = ConnectionStatus::Disconnected;
        }
    }
    
    pub fn connection_count(&self) -> usize {
        self.connections.blocking_read().len()
    }
    
    pub async fn get_status(&self) -> WebSocketStatus {
        let connections = self.connections.read().await;
        let connected = connections.values().filter(|c| c.status == ConnectionStatus::Connected).count();
        let disconnected = connections.values().filter(|c| c.status == ConnectionStatus::Disconnected).count();
        
        WebSocketStatus {
            total: connections.len(),
            connected,
            disconnected,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketStatus {
    pub total: usize,
    pub connected: usize,
    pub disconnected: usize,
}
