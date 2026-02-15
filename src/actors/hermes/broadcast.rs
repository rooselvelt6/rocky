// src/actors/hermes/broadcast.rs
// OLYMPUS v13 - Hermes Broadcaster
// Envío de mensajes a múltiples destinatarios

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use super::GodName;
use crate::traits::message::ActorMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub message: ActorMessage,
    pub exclude: Vec<GodName>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct Broadcaster {
    sender: broadcast::Sender<BroadcastEvent>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, message: ActorMessage, exclude: Vec<GodName>) {
        let event = BroadcastEvent {
            message,
            exclude,
            timestamp: chrono::Utc::now(),
        };
        let _ = self.sender.send(event);
    }

    pub fn send_to(&self, god: GodName, message: ActorMessage, exclude: Vec<GodName>) {
        if !exclude.contains(&god) {
            self.broadcast(message, exclude);
        }
    }

    pub fn get_sender(&self) -> broadcast::Sender<BroadcastEvent> {
        self.sender.clone()
    }
}
