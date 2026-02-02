/// Iris v12 - Message Bus y Communication
/// Diosa del Arcoíris y mensajería divina

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisMessage {
    pub id: String,
    pub message_type: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub priority: MessagePriority,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct IrisV12 {
    broadcast_sender: broadcast::Sender<IrisMessage>,
    direct_channels: HashMap<String, mpsc::UnboundedSender<IrisMessage>>,
}

impl IrisV12 {
    pub fn new() -> Self {
        let (broadcast_sender, _) = broadcast::channel(1000);
        
        Self {
            broadcast_sender,
            direct_channels: HashMap::new(),
        }
    }

    pub async fn broadcast(&self, message: IrisMessage) -> Result<(), String> {
        match self.broadcast_sender.send(message.clone()) {
            Ok(_) => {
                tracing::info!("🕊️ Iris: Mensaje broadcast enviado a {} receptores", 1);
                Ok(())
            }
            Err(_) => Err("No hay receptores disponibles".to_string()),
        }
    }

    pub async fn send_to(&mut self, recipient: &str, message: IrisMessage) -> Result<(), String> {
        if !self.direct_channels.contains_key(recipient) {
            let (sender, receiver) = mpsc::unbounded_channel();
            self.direct_channels.insert(recipient.to_string(), sender);
            tracing::info!("🕊️ Iris: Canal directo creado para {}", recipient);
        }
        
        if let Some(sender) = self.direct_channels.get(recipient) {
            match sender.send(message) {
                Ok(_) => Ok(()),
                Err(_) => Err(format!("No se puede enviar mensaje a {}", recipient)),
            }
        } else {
            Err(format!("Receptor {} no encontrado", recipient))
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IrisMessage> {
        self.broadcast_sender.subscribe()
    }

    pub fn subscribe_direct(&mut self, actor_name: &str) -> mpsc::UnboundedReceiver<IrisMessage> {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.direct_channels.insert(actor_name.to_string(), sender);
        receiver
    }

    pub fn create_message(&self, message_type: String, payload: serde_json::Value, priority: MessagePriority) -> IrisMessage {
        IrisMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            sender: "iris".to_string(),
            recipient: None,
            priority,
            payload,
            timestamp: Utc::now(),
        }
    }
}

impl Default for IrisV12 {
    fn default() -> Self {
        Self::new()
    }
}