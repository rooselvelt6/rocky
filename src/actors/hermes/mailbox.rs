// src/actors/hermes/mailbox.rs
// OLYMPUS v13 - Hermes Mailbox
// Cola de mensajes por dios

use tokio::sync::mpsc;
use std::time::Instant;
use serde::{Deserialize, Serialize};

use super::GodName;
use crate::traits::message::ActorMessage;

#[derive(Debug, Clone)]
pub struct Mailbox {
    god: GodName,
    receiver: mpsc::Receiver<ActorMessage>,
    queue: Vec<ActorMessage>,
    max_size: usize,
}

impl Mailbox {
    pub fn new(god: GodName, max_size: usize) -> (Self, mpsc::Sender<ActorMessage>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                god,
                receiver: rx,
                queue: Vec::new(),
                max_size,
            },
            tx,
        )
    }
    
    pub async fn deliver(&self, message: ActorMessage) {
        // In real implementation, would send to the actor's mailbox channel
    }
    
    pub async fn receive(&mut self) -> Option<ActorMessage> {
        self.receiver.recv().await
    }
    
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxStats {
    pub god: GodName,
    pub message_count: u64,
    pub oldest_message: Option<Instant>,
}
