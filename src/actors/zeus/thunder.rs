// src/actors/zeus/thunder.rs
// OLYMPUS v13 - Zeus Thunderbolt
// Sistema de broadcast instantáneo a velocidad de la luz

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Thunderbolt - Canal de broadcast de Zeus
#[derive(Debug, Clone)]
pub struct Thunderbolt {
    sender: broadcast::Sender<ThunderEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThunderEvent {
    // Actor lifecycle
    ActorStarted {
        actor: super::GodName,
    },
    ActorStopped {
        actor: super::GodName,
        reason: String,
    },
    ActorRecovered {
        actor: super::GodName,
    },

    // System events
    SystemHealthy,
    SystemDegraded {
        reason: String,
    },
    SystemCritical {
        reason: String,
    },

    // Data events
    DataBroadcast {
        source: super::GodName,
        data_type: String,
    },

    // Emergency
    Emergency {
        reason: String,
        severity: ThunderSeverity,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThunderSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Thunderbolt {
    pub fn new(sender: broadcast::Sender<ThunderEvent>) -> Self {
        Self { sender }
    }

    pub fn new_broadcast() -> (Self, broadcast::Receiver<ThunderEvent>) {
        let (sender, receiver) = broadcast::channel(100);
        (Self { sender }, receiver)
    }

    pub fn broadcast(
        &self,
        event: ThunderEvent,
    ) -> Result<usize, broadcast::error::SendError<ThunderEvent>> {
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ThunderEvent> {
        self.sender.subscribe()
    }

    pub fn send_actor_started(&self, actor: super::GodName) {
        let _ = self.broadcast(ThunderEvent::ActorStarted { 
            actor, 
        });
    }

    pub fn send_actor_stopped(&self, actor: super::GodName, reason: String) {
        let _ = self.broadcast(ThunderEvent::ActorStopped { 
            actor, 
            reason, 
        });
    }

    pub fn send_emergency(&self, reason: String, _severity: ThunderSeverity) {
        let _ = self.broadcast(ThunderEvent::Emergency { 
            reason, 
            severity: ThunderSeverity::Critical,
        });
    }
}
