use tokio::sync::{mpsc, broadcast};
use async_trait::async_trait;
use std::sync::Arc;

pub mod zeus;
pub mod moirai;
pub mod erinyes;
pub mod iris;
pub mod chaos;
pub mod hera;
pub mod athena;
pub mod chronos;
pub mod hestia;
pub mod poseidon;
pub mod hades;
pub mod hephaestus;
pub mod artemis;
pub mod hermes;
pub mod apollo;
pub mod demeter;
pub mod ares;
pub mod dionysus;
pub mod aphrodite;

/// Mensajes universales del bus Iris
#[derive(Debug, Clone)]
pub enum IrisSignal {
    Panic(String),
    Heartbeat(String),
    SystemReady,
    SystemShutdown,
}

/// Mensajes de comando para los Actores
#[derive(Debug)]
pub enum GodCommand {
    Start,
    Stop,
    Status,
    Recover(String),
}

/// Trait base para toda deidad (Actor)
#[async_trait]
pub trait GodActor: Send + Sync {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_command(&mut self, cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>>;
}
