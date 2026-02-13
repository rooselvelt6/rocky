// src/actors/hermes/mailbox.rs
// OLYMPUS v15 - Hermes Mailbox
// Cola de mensajes por dios con procesamiento real

use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::{warn, debug};

use super::GodName;
use crate::traits::message::ActorMessage;
use crate::errors::ActorError;

#[derive(Debug, Clone)]
pub struct Mailbox {
    god: GodName,
    sender: mpsc::Sender<ActorMessage>,
    receiver: Arc<RwLock<mpsc::Receiver<ActorMessage>>>,
    internal_queue: Arc<RwLock<VecDeque<ActorMessage>>>,
    max_size: usize,
    delivered_count: Arc<RwLock<u64>>,
    failed_count: Arc<RwLock<u64>>,
    last_delivery: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl Mailbox {
    pub fn new(god: GodName, max_size: usize) -> (Self, mpsc::Sender<ActorMessage>) {
        let (tx, rx) = mpsc::channel(max_size);
        
        let mailbox = Self {
            god: god.clone(),
            sender: tx.clone(),
            receiver: Arc::new(RwLock::new(rx)),
            internal_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
            delivered_count: Arc::new(RwLock::new(0)),
            failed_count: Arc::new(RwLock::new(0)),
            last_delivery: Arc::new(RwLock::new(None)),
        };
        
        (mailbox, tx)
    }
    
    pub async fn deliver(&self, message: ActorMessage) -> Result<(), ActorError> {
        // First, try to send directly through the channel
        match self.sender.try_send(message.clone()) {
            Ok(()) => {
                debug!(
                    message_id = %message.id,
                    to = %self.god,
                    "Message delivered to mailbox channel"
                );
                
                let mut count = self.delivered_count.write().await;
                *count += 1;
                
                let mut last = self.last_delivery.write().await;
                *last = Some(chrono::Utc::now());
                
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(msg)) => {
                // Channel full, queue internally
                warn!(
                    message_id = %msg.id,
                    to = %self.god,
                    "Mailbox channel full, queuing internally"
                );
                
                let mut queue = self.internal_queue.write().await;
                if queue.len() >= self.max_size {
                    let mut count = self.failed_count.write().await;
                    *count += 1;
                    return Err(ActorError::MailboxFull { 
                        god: self.god.clone(),
                        max_size: self.max_size 
                    });
                }
                
                queue.push_back(msg);
                
                let mut count = self.delivered_count.write().await;
                *count += 1;
                
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(ActorError::ActorNotRunning { god: self.god.clone() })
            }
        }
    }
    
    pub async fn receive(&self) -> Option<ActorMessage> {
        // First check internal queue
        let mut queue = self.internal_queue.write().await;
        if let Some(msg) = queue.pop_front() {
            return Some(msg);
        }
        drop(queue);
        
        // Then check the channel
        let mut receiver = self.receiver.write().await;
        receiver.recv().await
    }
    
    pub async fn try_receive(&self) -> Option<ActorMessage> {
        // First check internal queue
        let mut queue = self.internal_queue.write().await;
        if let Some(msg) = queue.pop_front() {
            return Some(msg);
        }
        drop(queue);
        
        // Then try the channel (non-blocking)
        let mut receiver = self.receiver.write().await;
        receiver.try_recv().ok()
    }
    
    pub async fn len(&self) -> usize {
        let queue = self.internal_queue.read().await;
        queue.len()
    }
    
    pub async fn is_empty(&self) -> bool {
        let queue = self.internal_queue.read().await;
        queue.is_empty()
    }
    
    pub async fn stats(&self) -> MailboxStats {
        let delivered = *self.delivered_count.read().await;
        let failed = *self.failed_count.read().await;
        let last = *self.last_delivery.read().await;
        let queued = self.len().await;
        
        MailboxStats {
            god: self.god.clone(),
            delivered_count: delivered,
            failed_count: failed,
            queued_count: queued as u64,
            last_delivery: last,
            max_size: self.max_size,
        }
    }
    
    pub async fn clear(&self) {
        let mut queue = self.internal_queue.write().await;
        queue.clear();
        
        // Drain the channel
        let mut receiver = self.receiver.write().await;
        while receiver.try_recv().is_ok() {}
    }
    
    pub fn get_sender(&self) -> mpsc::Sender<ActorMessage> {
        self.sender.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxStats {
    pub god: GodName,
    pub delivered_count: u64,
    pub failed_count: u64,
    pub queued_count: u64,
    pub last_delivery: Option<chrono::DateTime<chrono::Utc>>,
    pub max_size: usize,
}

#[derive(Debug)]
pub struct MailboxManager {
    mailboxes: Arc<RwLock<std::collections::HashMap<GodName, Mailbox>>>,
    default_max_size: usize,
}

impl MailboxManager {
    pub fn new(default_max_size: usize) -> Self {
        Self {
            mailboxes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_max_size,
        }
    }
    
    pub async fn create_mailbox(&self, god: GodName) -> mpsc::Sender<ActorMessage> {
        let (mailbox, sender) = Mailbox::new(god.clone(), self.default_max_size);
        
        let mut mailboxes = self.mailboxes.write().await;
        mailboxes.insert(god, mailbox);
        
        sender
    }
    
    pub async fn get_mailbox(&self, god: &GodName) -> Option<Mailbox> {
        let mailboxes = self.mailboxes.read().await;
        mailboxes.get(god).cloned()
    }
    
    pub async fn deliver_to(&self, god: &GodName, message: ActorMessage) -> Result<(), ActorError> {
        let mailboxes = self.mailboxes.read().await;
        
        if let Some(mailbox) = mailboxes.get(god) {
            mailbox.deliver(message).await
        } else {
            Err(ActorError::NotFound { god: god.clone() })
        }
    }
    
    pub async fn remove_mailbox(&self, god: &GodName) {
        let mut mailboxes = self.mailboxes.write().await;
        mailboxes.remove(god);
    }
    
    pub async fn get_all_stats(&self) -> Vec<MailboxStats> {
        let mailboxes = self.mailboxes.read().await;
        let mut stats = Vec::new();
        
        for (_, mailbox) in mailboxes.iter() {
            stats.push(mailbox.stats().await);
        }
        
        stats
    }
    
    pub async fn total_messages(&self) -> u64 {
        let mailboxes = self.mailboxes.read().await;
        let mut total = 0u64;
        
        for (_, mailbox) in mailboxes.iter() {
            let stats = mailbox.stats().await;
            total += stats.delivered_count;
        }
        
        total
    }
}
